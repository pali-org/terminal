use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(feature = "cli")]
use crate::logging::{log_http_request, log_http_response};

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
    /// # Features (when http-optimized is enabled):
    /// - Fast DNS resolution using Hickory DNS (prevents EU routing issues)
    /// - Optimized connection pooling and keep-alive
    /// - Rustls TLS for better performance than OpenSSL
    /// - Reduced TLS handshake overhead with connection reuse
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Configuration cannot be loaded from disk
    /// - Configuration file format is invalid
    /// - HTTP client initialization fails
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        
        #[cfg(feature = "http-optimized")]
        let client = Self::build_optimized_client()?;
        
        #[cfg(not(feature = "http-optimized"))]
        let client = Self::build_standard_client()?;
            
        Ok(Self { client, config })
    }

    #[cfg(feature = "http-optimized")]
    fn build_optimized_client() -> Result<Client> {
        // Build an optimized HTTP client focused on reducing latency
        let client = Client::builder()
            // Connection and timeout optimizations to reduce latency
            .timeout(Duration::from_secs(30))              // Total request timeout
            .connect_timeout(Duration::from_secs(5))       // Faster connection timeout
            .tcp_nodelay(true)                             // Disable Nagle's algorithm for faster small requests
            .tcp_keepalive(Duration::from_secs(60))        // Keep TCP connections alive
            
            // Aggressive connection pool optimizations for connection reuse
            .pool_max_idle_per_host(20)                    // Keep more connections alive per host
            .pool_idle_timeout(Duration::from_secs(120))   // Keep connections alive longer to avoid TLS handshakes
            
            // TLS optimizations to reduce handshake overhead 
            .use_rustls_tls()                              // Use rustls (faster than OpenSSL, better geolocation)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)  // Minimum TLS 1.2
            
            // DNS optimization - Use Hickory DNS for better routing (key fix for EU issue)
            .hickory_dns(true)                             // Use Hickory DNS resolver (prevents EU routing issues)
            
            // User agent for debugging/monitoring  
            .user_agent(concat!("pali-terminal/", env!("CARGO_PKG_VERSION"), " (http-optimized)"))
            
            .build()
            .context("Unable to initialize network client")?;
        
        Ok(client)
    }

    #[cfg(not(feature = "http-optimized"))]
    fn build_standard_client() -> Result<Client> {
        // Build a standard HTTP client with default settings
        let client = Client::builder()
            // Basic timeout settings
            .timeout(Duration::from_secs(30))              // Total request timeout
            .connect_timeout(Duration::from_secs(10))      // Standard connection timeout
            
            // User agent for debugging/monitoring  
            .user_agent(concat!("pali-terminal/", env!("CARGO_PKG_VERSION"), " (standard)"))
            
            .build()
            .context("Unable to initialize network client")?;
        
        Ok(client)
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
                response.json().await.context("Unable to process server response")?;

            if api_response.success {
                api_response
                    .data
                    .ok_or_else(|| anyhow::anyhow!("Server returned success status but no data. This indicates a server-side issue - please contact support."))
            } else {
                let error_msg = api_response
                    .error
                    .unwrap_or_else(|| "The server encountered an issue. Please try again.".to_string());
                anyhow::bail!("{}", error_msg)
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to connect to server. Please check your connection.".to_string());
            anyhow::bail!("Server error: {}", if error_text.trim().is_empty() { "Please try again later" } else { &error_text })
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
        let url = self.build_url("/todos");
        
        #[cfg(feature = "cli")]
        log_http_request("POST", &url, true);
        
        let req = self.client.post(&url);
        let req = self.add_auth_header(req);

        let start = std::time::Instant::now();
        let response = req.json(&request).send().await?;
        let elapsed = start.elapsed();
        
        #[cfg(feature = "cli")]
        log_http_response(response.status().as_u16(), elapsed);

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
        let url = self.build_url("/todos");
        
        #[cfg(feature = "cli")]
        log_http_request("GET", &url, false);
        
        let req = self.client.get(&url);
        let mut req = self.add_auth_header(req);

        if let Some(tag) = tag {
            req = req.query(&[("tag", tag)]);
        }

        if let Some(priority) = priority {
            req = req.query(&[("priority", priority)]);
        }

        let start = std::time::Instant::now();
        let response = req.send().await?;
        let elapsed = start.elapsed();
        
        #[cfg(feature = "cli")]
        log_http_response(response.status().as_u16(), elapsed);
        
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
                .unwrap_or_else(|_| "Unable to connect to server. Please check your connection.".to_string());
            anyhow::bail!("Server error: {}", if error_text.trim().is_empty() { "Please try again later" } else { &error_text })
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
                .unwrap_or_else(|_| "Unable to connect to server. Please check your connection.".to_string());
            anyhow::bail!("Server error: {}", if error_text.trim().is_empty() { "Please try again later" } else { &error_text })
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
        use pali_types::ApiKeyResponse;
        
        let req = self.client.post(self.build_url("/initialize"));
        // Note: No auth header for initialize - it's for first-time setup
        
        let response = req.send().await?;
        let result: ApiKeyResponse = Self::handle_response(response).await?;
        Ok(result.api_key)
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
        use pali_types::ApiKeyResponse;
        
        let req = self.client.post(self.build_url("/reinitialize"));
        let req = self.add_auth_header(req);
        
        let response = req.send().await?;
        let result: ApiKeyResponse = Self::handle_response(response).await?;
        Ok(result.api_key)
    }

    /// Resolves a partial ID prefix to a full todo ID (server-side)
    /// 
    /// **NOTE**: This endpoint is not yet implemented by Claude #2.
    /// When available, it will provide efficient server-side prefix resolution
    /// instead of fetching all todos client-side.
    /// 
    /// # Arguments
    /// 
    /// * `prefix` - The partial ID prefix (e.g., "d2fa")
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The full UUID if exactly one match is found
    /// * `Err` - If no matches found, multiple ambiguous matches, or endpoint not available
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails
    /// - No todo found with the given prefix
    /// - Multiple todos match the prefix (ambiguous)
    /// - Server returns an error response
    /// - API key is missing or invalid
    pub async fn resolve_id_prefix(&self, prefix: &str) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct ResolveResponse {
            full_id: String,
        }

        let url = self.build_url(&format!("/todos/resolve/{}", prefix));
        
        #[cfg(feature = "cli")]
        log_http_request("GET", &url, false);
        
        let req = self.client.get(&url);
        let req = self.add_auth_header(req);

        let start = std::time::Instant::now();
        let response = req.send().await?;
        let elapsed = start.elapsed();
        
        #[cfg(feature = "cli")]
        log_http_response(response.status().as_u16(), elapsed);
        
        let result: ResolveResponse = Self::handle_response(response).await?;
        Ok(result.full_id)
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
