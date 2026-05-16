//! User endpoints: lookup, login, and API key management.

use crate::prelude::*;

/// Manager for user-related API endpoints.
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
    /// Requires a session token (`x-auth-token` + `x-device-id`).
    pub async fn lookup_user(&self, query: &str) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        params.insert("q".to_string(), query.to_string());
        let qs = build_request(&params);
        self.client
            .get_session(API::User(UserEndpoint::Lookup).as_ref(), Some(qs))
            .await
    }

    /// Authenticate with a Bayse account to get a session token.
    ///
    /// Body should contain login credentials (email + password or similar).
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

    pub async fn authenticated(&mut self, email: &str, password: &str) -> Result<Self, BayseError> {
        self.login(email, password).await?;
        Ok(self.clone())
    }

    /// Create a new API key pair for programmatic access.
    ///
    /// Requires session authentication.
    async fn create_api_key(&self, name: &str) -> Result<CreateApiKeyResponse, BayseError> {
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
    pub async fn list_api_keys(&self) -> Result<ListApiKeysResponse, BayseError> {
        self.client
            .get_session(API::User(UserEndpoint::ListApiKeys).as_ref(), None)
            .await
    }

    /// Revoke (permanently deactivate) an API key.
    ///
    /// Requires session authentication.
    pub async fn revoke_api_key(&self, key_id: &str) -> Result<RevokeApiKeyResponse, BayseError> {
        let endpoint = format!("/v1/user/me/api-keys/{key_id}");
        self.client
            .delete_session::<RevokeApiKeyResponse>(&endpoint, None)
            .await
    }

    /// Rotate an API key's secret while keeping the same key ID.
    ///
    /// Requires session authentication.
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
