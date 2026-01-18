#[cfg(not(target_arch = "wasm32"))]
use mochi_browser::http::{fetch_url, HttpResponse};

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_http_response_default() {
    let response = HttpResponse {
        status: 0,
        headers: vec![],
        body: String::new(),
    };

    assert_eq!(response.status, 0);
    assert_eq!(response.headers.len(), 0);
    assert_eq!(response.body, "");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_http_response_with_data() {
    let response = HttpResponse {
        status: 200,
        headers: vec![
            ("content-length".to_string(), "100".to_string()),
            ("content-type".to_string(), "text/plain".to_string()),
        ],
        body: "Hello, World!".to_string(),
    };

    assert_eq!(response.status, 200);
    assert_eq!(response.headers.len(), 2);
    assert_eq!(response.headers[0].0, "content-length");
    assert_eq!(response.headers[0].1, "100");
    assert_eq!(response.headers[1].0, "content-type");
    assert_eq!(response.headers[1].1, "text/plain");
    assert_eq!(response.body, "Hello, World!");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_http_response_clone() {
    let response = HttpResponse {
        status: 404,
        headers: vec![("x-custom".to_string(), "value".to_string())],
        body: "Not Found".to_string(),
    };

    let cloned = response.clone();

    assert_eq!(cloned, response);
    assert_eq!(cloned.status, 404);
    assert_eq!(cloned.body, "Not Found");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_fetch_url_simple_get() {
    let result = fetch_url("https://httpbin.org/get");

    assert!(result.is_ok(), "Fetch should succeed");
    let response = result.unwrap();
    assert_eq!(response.status, 200, "Status should be 200");
    assert!(!response.body.is_empty(), "Body should not be empty");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_fetch_url_with_query_params() {
    let result = fetch_url("https://httpbin.org/get?param1=value1&param2=value2");

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.contains("param1"));
    assert!(response.body.contains("value1"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_fetch_url_multiple_requests() {
    let result1 = fetch_url("https://httpbin.org/status/200");
    let result2 = fetch_url("https://httpbin.org/status/201");

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap().status, 200);
    assert_eq!(result2.unwrap().status, 201);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_fetch_url_user_agent() {
    let result = fetch_url("https://httpbin.org/user-agent");

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
}
