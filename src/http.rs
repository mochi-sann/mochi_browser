use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[cfg(not(target_arch = "wasm32"))]
mod fetch {
    use super::HttpResponse;

    /// Fetches a URL and returns the HTTP response.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the response body cannot be read,
    /// or header values are not valid UTF-8.
    pub fn fetch_url(url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let response = reqwest::blocking::get(url)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_owned()))
            .collect();
        let body = response.text()?;
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use fetch::fetch_url;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response_serialization() {
        let response = HttpResponse {
            status: 200,
            headers: vec![("Content-Type".to_string(), "text/html".to_string())],
            body: "test body".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: HttpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, response);
    }

    #[test]
    fn test_http_response_empty_headers() {
        let response = HttpResponse {
            status: 404,
            headers: vec![],
            body: "not found".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: HttpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.status, 404);
        assert_eq!(deserialized.headers.len(), 0);
        assert_eq!(deserialized.body, "not found");
    }

    #[test]
    fn test_http_response_json_serialization() {
        let response = HttpResponse {
            status: 500,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Server".to_string(), "TestServer".to_string()),
            ],
            body: "{\"error\": \"internal server error\"}".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: HttpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, response);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_fetch_url_success() {
        let result = fetch_url("https://httpbin.org/status/200");

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, 200);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_fetch_url_not_found() {
        let result = fetch_url("https://httpbin.org/status/404");

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, 404);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_fetch_url_invalid_url() {
        let result = fetch_url("not a valid url");

        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_fetch_url_gets_headers() {
        let result = fetch_url("https://httpbin.org/headers");

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.headers.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_fetch_url_gets_body() {
        let result = fetch_url("https://httpbin.org/uuid");

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.body.is_empty());
        assert!(response.body.contains("uuid"));
    }
}
