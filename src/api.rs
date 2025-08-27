use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

const API_KEY_HEADER: &str = "X-API-Key";

// Re-export shared types
pub use pali_types::*;

// CLI-specific type aliases
pub type ApiKey = ApiKeyInfo;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GenerateKeyResponse {
    pub key: String,
    pub id: String,
}

pub struct ApiClient {
    client: Client,
    config: Config,
}

impl ApiClient {
    /// Creates a new API client with configuration loaded from disk
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Configuration cannot be loaded from disk
    /// - Configuration file format is invalid
    /// - HTTP client initialization fails
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let client = Client::new();
        Ok(Self { client, config })
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_endpoint.trim_end_matches('/'), path)
    }

    fn add_auth_header(&self, mut req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref key) = self.config.api_key {
            req = req.header(API_KEY_HEADER, key);
        }
        req
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let api_response: ApiResponse<T> =
                response.json().await.context("Failed to parse response")?;

            if api_response.success {
                api_response
                    .data
                    .ok_or_else(|| anyhow::anyhow!("Server returned success status but no data. This indicates a server-side issue - please contact support."))
            } else {
                let error_msg = api_response
                    .error
                    .unwrap_or_else(|| "Server returned an error but didn't provide details".to_string());
                anyhow::bail!("API error: {}", error_msg)
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response from server".to_string());
            anyhow::bail!("API error ({}): {}", status, error_text)
        }
    }

    /// Creates a new todo item
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - API key is missing or invalid
    pub async fn create_todo(&self, request: CreateTodoRequest) -> Result<Todo> {
        let req = self.client.post(self.build_url("/todos"));
        let req = self.add_auth_header(req);

        let response = req.json(&request).send().await?;

        Self::handle_response(response).await
    }

    /// Lists todos with optional filtering by tag and priority
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - API key is missing or invalid
    pub async fn list_todos(
        &self,
        tag: Option<String>,
        priority: Option<String>,
    ) -> Result<Vec<Todo>> {
        let req = self.client.get(self.build_url("/todos"));
        let mut req = self.add_auth_header(req);

        if let Some(tag) = tag {
            req = req.query(&[("tag", tag)]);
        }

        if let Some(priority) = priority {
            req = req.query(&[("priority", priority)]);
        }

        let response = req.send().await?;
        Self::handle_response(response).await
    }

    /// Retrieves a specific todo by ID
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Todo with the given ID is not found
    /// - Server returns an error response
    /// - Response parsing fails
    /// - API key is missing or invalid
    pub async fn get_todo(&self, id: &str) -> Result<Todo> {
        let req = self.client.get(self.build_url(&format!("/todos/{id}")));
        let req = self.add_auth_header(req);

        let response = req.send().await?;
        Self::handle_response(response).await
    }

    /// Updates an existing todo item
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Todo with the given ID is not found
    /// - Server returns an error response
    /// - Response parsing fails
    /// - API key is missing or invalid
    pub async fn update_todo(&self, id: &str, request: UpdateTodoRequest) -> Result<Todo> {
        let req = self.client.put(self.build_url(&format!("/todos/{id}")));
        let req = self.add_auth_header(req);

        let response = req.json(&request).send().await?;

        Self::handle_response(response).await
    }

    /// Deletes a todo item by ID
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Todo with the given ID is not found
    /// - Server returns an error response
    /// - API key is missing or invalid
    pub async fn delete_todo(&self, id: &str) -> Result<()> {
        let req = self
            .client
            .delete(self.build_url(&format!("/todos/{id}")));
        let req = self.add_auth_header(req);

        let response = req.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response from server".to_string());
            anyhow::bail!("API error ({}): {}", status, error_text)
        }
    }

    /// Toggles the completion status of a todo item
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Todo with the given ID is not found
    /// - Server returns an error response
    /// - Response parsing fails
    /// - API key is missing or invalid
    pub async fn toggle_todo(&self, id: &str) -> Result<Todo> {
        let req = self
            .client
            .patch(self.build_url(&format!("/todos/{id}/toggle")));
        let req = self.add_auth_header(req);

        let response = req.send().await?;
        Self::handle_response(response).await
    }

    /// Searches todos by query string
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - API key is missing or invalid
    pub async fn search_todos(&self, query: &str) -> Result<Vec<Todo>> {
        let req = self.client.get(self.build_url("/todos/search"));
        let req = self.add_auth_header(req);

        let response = req.query(&[("q", query)]).send().await?;

        Self::handle_response(response).await
    }

    /// Rotates the admin API key, generating a new key
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - Current API key is invalid or lacks admin privileges
    pub async fn rotate_admin_key(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct RotateResponse {
            new_key: String,
        }
        
        let req = self.client.post(self.build_url("/admin/keys/rotate"));
        let req = self.add_auth_header(req);

        let response = req.send().await?;
        let result: RotateResponse = Self::handle_response(response).await?;
        Ok(result.new_key)
    }

    /// Generates a new API key with optional name
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - Current API key lacks admin privileges
    pub async fn generate_api_key(&self, name: Option<&str>) -> Result<GenerateKeyResponse> {
        let req = self.client.post(self.build_url("/admin/keys/generate"));
        let mut req = self.add_auth_header(req);

        if let Some(name) = name {
            #[derive(Serialize)]
            struct GenerateRequest {
                name: String,
            }
            req = req.json(&GenerateRequest {
                name: name.to_string(),
            });
        }

        let response = req.send().await?;
        Self::handle_response(response).await
    }

    /// Lists all API keys (admin only)
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - Current API key lacks admin privileges
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let req = self.client.get(self.build_url("/admin/keys"));
        let req = self.add_auth_header(req);

        let response = req.send().await?;
        Self::handle_response(response).await
    }

    /// Revokes an API key by ID (admin only)
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - API key with the given ID is not found
    /// - Server returns an error response
    /// - Current API key lacks admin privileges
    pub async fn revoke_api_key(&self, id: &str) -> Result<()> {
        let req = self
            .client
            .delete(self.build_url(&format!("/admin/keys/{id}")));
        let req = self.add_auth_header(req);

        let response = req.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response from server".to_string());
            anyhow::bail!("API error ({}): {}", status, error_text)
        }
    }

    /// Initializes the server and returns the first admin API key (one-time setup)
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server is already initialized
    /// - Server returns an error response
    /// - Response parsing fails
    pub async fn initialize(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct InitializeResponse {
            admin_key: String,
        }
        
        let req = self.client.post(self.build_url("/initialize"));
        // Note: No auth header for initialize - it's for first-time setup
        
        let response = req.send().await?;
        let result: InitializeResponse = Self::handle_response(response).await?;
        Ok(result.admin_key)
    }

    /// Reinitializes the server, deactivating ALL admin keys and returning a new one (emergency reset)
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - Server returns an error response
    /// - Response parsing fails
    /// - Current API key lacks admin privileges
    pub async fn reinitialize(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct ReinitializeResponse {
            admin_key: String,
        }
        
        let req = self.client.post(self.build_url("/reinitialize"));
        let req = self.add_auth_header(req);
        
        let response = req.send().await?;
        let result: ReinitializeResponse = Self::handle_response(response).await?;
        Ok(result.admin_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_url() {
        let config = Config {
            api_endpoint: "http://localhost:8787".to_string(),
            api_key: None,
        };
        let client = ApiClient {
            client: Client::new(),
            config,
        };
        
        assert_eq!(client.build_url("/todos"), "http://localhost:8787/todos");
    }
    
    #[test]
    fn test_build_url_strips_trailing_slash() {
        let config = Config {
            api_endpoint: "http://localhost:8787/".to_string(),
            api_key: None,
        };
        let client = ApiClient {
            client: Client::new(),
            config,
        };
        
        assert_eq!(client.build_url("/todos"), "http://localhost:8787/todos");
    }
    
    #[test]
    fn test_api_client_has_correct_fields() {
        let config = Config::default();
        let client = ApiClient {
            client: Client::new(),
            config: config.clone(),
        };
        
        // Verify the client was constructed properly
        assert_eq!(client.config.api_endpoint, config.api_endpoint);
        assert_eq!(client.config.api_key, config.api_key);
    }
    
    #[test]
    fn test_build_url_with_different_paths() {
        let config = Config {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: Some("test-key".to_string()),
        };
        let client = ApiClient {
            client: Client::new(),
            config,
        };
        
        assert_eq!(client.build_url("/todos"), "https://api.example.com/todos");
        assert_eq!(client.build_url("/keys"), "https://api.example.com/keys");
        assert_eq!(client.build_url("/todos/123"), "https://api.example.com/todos/123");
    }
}
