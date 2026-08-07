//! User endpoints: lookup, login, and API key management.

use crate::prelude::*;

/// Manager for user-related API endpoints.
///
/// Provides methods for user lookup, session-based login, and API key
/// lifecycle management (create, list, revoke, rotate).
#[derive(Clone)]
pub struct UserManager {
    pub client: Client,
}

impl Bayse for UserManager {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        let config = if api_key.is_some() || secret_key.is_some() {
            Config::default()
                .with_api_key(api_key.unwrap_or_default(), secret_key.unwrap_or_default())
        } else {
            Config::default()
        };
        Self::new_with_config(config)
    }

    fn new_with_config(config: Config) -> Self {
        let client = Client::new(
            config.api_key,
            config.secret_key,
            config.rest_api_endpoint.to_string(),
            config.session_token,
            config.device_id,
        );
        Self { client }
    }
}

impl UserManager {
    /// Resolve a user tag or ID to their public profile.
    ///
    /// Requires a valid API key (read-level: `X-Public-Key` header).
    /// Provide exactly one of `tag` or `user_id`.
    ///
    /// # Parameters
    ///
    /// * `tag` – The user's tag (username), case-insensitive.
    /// * `user_id` – The user's UUID.
    ///
    /// # Returns
    ///
    /// A JSON object with the user's public profile fields (`id`, `tag`,
    /// `imageUrl`), or an error if the query does not match any known user.
    pub async fn lookup_user(
        &self,
        tag: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(t) = tag {
            params.insert("tag".to_string(), t.to_string());
        }
        if let Some(uid) = user_id {
            params.insert("userId".to_string(), uid.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get_read(API::User(UserEndpoint::Lookup).as_ref(), Some(qs))
            .await
    }

    /// Authenticate with a Bayse account to get a session token.
    ///
    /// On success the session token and device ID are persisted on the
    /// underlying client so that subsequent session-authenticated calls
    /// (e.g. `create_api_key`, `list_api_keys`) work automatically.
    ///
    /// # Parameters
    ///
    /// * `email` – The account email address.
    /// * `password` – The account password.
    ///
    /// # Returns
    ///
    /// A [`LoginResponse`] containing the session `token`, `device_id`,
    /// and `user_id`.
    pub async fn login(
        &mut self,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, BayseError> {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });
        let body_str = body.to_string();
        match self
            .client
            .post::<LoginResponse>(API::User(UserEndpoint::Login).as_ref(), Some(body_str))
            .await
        {
            Ok(response) => {
                // Persist the session on the client so that create_api_key (and any
                // future session-authenticated calls) can use it.
                self.client.session_token = Some(response.token.clone());
                self.client.device_id = Some(response.device_id.clone());
                Ok(response)
            }
            Err(err) => Err(err),
        }
    }

    /// Authenticate and return a cloned `UserManager` with an active session.
    ///
    /// This is a convenience wrapper around [`login`](Self::login) that
    /// calls login and then returns a cloned `self` so you can continue
    /// using the original manager or pass the authenticated clone elsewhere.
    ///
    /// # Parameters
    ///
    /// * `email` – The account email address.
    /// * `password` – The account password.
    ///
    /// # Returns
    ///
    /// A new `UserManager` clone with the session token and device ID
    /// already persisted on its internal client.
    pub async fn authenticated(&mut self, email: &str, password: &str) -> Result<Self, BayseError> {
        self.login(email, password).await?;
        Ok(self.clone())
    }

    /// Create a new API key pair for programmatic access.
    ///
    /// Requires session authentication. The returned response includes
    /// both the `public_key` and `secret_key` — the secret is only
    /// shown once and cannot be retrieved later.
    ///
    /// # Parameters
    ///
    /// * `name` – A human-readable label for the key (e.g. `"Trading bot"`).
    ///
    /// # Returns
    ///
    /// A [`CreateApiKeyResponse`] containing the new key ID, name, timestamps,
    /// public key, secret key, and signing instructions.
    pub async fn create_api_key(&self, name: &str) -> Result<CreateApiKeyResponse, BayseError> {
        let body = serde_json::json!({ "name": name });
        let body_str = body.to_string();
        self.client
            .post_session(
                API::User(UserEndpoint::CreateApiKey).as_ref(),
                Some(body_str),
            )
            .await
    }

    /// List all active API keys for your account.
    ///
    /// Requires session authentication.
    ///
    /// # Returns
    ///
    /// A [`ListApiKeysResponse`] containing the list of active keys and
    /// the total count. Each key includes its `public_key` and a
    /// `secret_key_hint` (last four characters) for identification.
    pub async fn list_api_keys(&self) -> Result<ListApiKeysResponse, BayseError> {
        self.client
            .get_session(API::User(UserEndpoint::ListApiKeys).as_ref(), None)
            .await
    }

    /// Revoke (permanently deactivate) an API key.
    ///
    /// Once revoked the key can no longer be used for API access.
    /// This action is irreversible.
    ///
    /// Requires session authentication.
    ///
    /// # Parameters
    ///
    /// * `key_id` – The ID of the key to revoke (retrieved from
    ///   [`list_api_keys`](Self::list_api_keys)).
    ///
    /// # Returns
    ///
    /// A [`RevokeApiKeyResponse`] with a confirmation message.
    pub async fn revoke_api_key(&self, key_id: &str) -> Result<RevokeApiKeyResponse, BayseError> {
        let endpoint = format!("/v1/user/me/api-keys/{key_id}");
        self.client
            .delete_session::<RevokeApiKeyResponse>(&endpoint, None)
            .await
    }

    /// Rotate an API key's secret while keeping the same key ID.
    ///
    /// Generates a new `secret_key` for the given API key. The key ID
    /// and public key remain unchanged — only the signing secret is
    /// replaced. This is useful when a secret is compromised or as
    /// part of regular credential rotation.
    ///
    /// Requires session authentication.
    ///
    /// # Parameters
    ///
    /// * `key_id` – The ID of the key to rotate.
    ///
    /// # Returns
    ///
    /// A JSON object containing the updated key information, including
    /// the new `secret_key` (shown once).
    pub async fn rotate_api_key(&self, key_id: &str) -> Result<serde_json::Value, BayseError> {
        let endpoint = format!("/v1/user/me/api-keys/{key_id}/rotate");
        self.client.post_session(&endpoint, None).await
    }

    /// Login and immediately create an API key pair for programmatic access.
    ///
    /// This is a convenience method that combines `login` + `create_api_key`
    /// in a single call. The session token is stored on the client so that
    /// subsequent session-authenticated calls work without manual setup.
    ///
    /// * `email` – Account email address.
    /// * `password` – Account password.
    /// * `key_name` – A descriptive label for the new API key
    ///   (e.g. `"Trading bot"`).
    pub async fn login_and_create_api_key(
        &mut self,
        email: &str,
        password: &str,
        name: &str,
    ) -> Result<CreateApiKeyResponse, BayseError> {
        let login_response = self.login(email, password).await?;

        // Persist the session on the client so that create_api_key (and any
        // future session-authenticated calls) can use it.
        self.client.session_token = Some(login_response.token);
        self.client.device_id = Some(login_response.device_id);

        self.create_api_key(name).await
    }
}
