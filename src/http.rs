use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[cfg(not(target_arch = "wasm32"))]
mod fetch {
    use super::*;

    pub fn fetch_url(url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let response = reqwest::blocking::get(url)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
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
