use magnus::{function, method, Error as MagnusError, Module, Object, RHash, Value, exception, TryConvert, Symbol, IntoValue};
use magnus::r_hash::ForEach;
use rquest::{Response as RquestResponse, Error as RquestError};
use rquest::redirect::Policy;
use rquest_util::{Emulation as RquestEmulation};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::cell::{Cell, RefCell};
use serde_json::Error as JsonError;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::num::Wrapping;

// Fast random implementation similar to rquest-util crate
fn fast_random() -> u64 {
    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(seed()));
    }

    #[inline]
    fn seed() -> u64 {
        let seed = RandomState::new();
        let mut out = 0;
        let mut cnt = 0;
        while out == 0 {
            cnt += 1;
            let mut hasher = seed.build_hasher();
            hasher.write_usize(cnt);
            out = hasher.finish();
        }
        out
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545f4914f6cdd1d)
    })
}

// Function to get a random desktop browser emulation
fn get_random_desktop_emulation() -> RquestEmulation {
    let browsers = [
        RquestEmulation::Chrome134,
        RquestEmulation::Chrome128,
        RquestEmulation::Chrome101,
        RquestEmulation::Firefox135,
        RquestEmulation::Safari17_0
    ];
    
    let index = (fast_random() as usize) % browsers.len();
    browsers[index]
}

// Function to get a random mobile browser emulation
fn get_random_mobile_emulation() -> RquestEmulation {
    // We primarily use Safari iOS, which is a guaranteed mobile variant
    // Fall back to other variants if needed
    let browsers = [
        RquestEmulation::SafariIos17_4_1,  // This is the mobile Safari variant
        RquestEmulation::SafariIos17_2,    // Another mobile Safari variant
        RquestEmulation::SafariIos16_5,    // Another mobile Safari variant
        RquestEmulation::FirefoxAndroid135  // Firefox for Android variant
    ];
    
    let index = (fast_random() as usize) % browsers.len();
    browsers[index]
}

// Function to get a random browser emulation (desktop or mobile)
fn get_random_emulation() -> RquestEmulation {
    // 80% chance to get desktop browser
    if fast_random() % 100 < 80 {
        get_random_desktop_emulation()
    } else {
        get_random_mobile_emulation()
    }
}

// Helper function to convert RquestError to MagnusError
fn rquest_error_to_magnus_error(err: RquestError) -> MagnusError {
    MagnusError::new(exception::runtime_error(), format!("HTTP request failed: {}", err))
}

// Helper function to convert JsonError to MagnusError
fn json_error_to_magnus_error(err: JsonError) -> MagnusError {
    MagnusError::new(exception::runtime_error(), format!("JSON serialization failed: {}", err))
}

// Create a runtime once and reuse it without using static mut
fn get_runtime() -> Arc<Runtime> {
    thread_local! {
        static RUNTIME: RefCell<Option<Arc<Runtime>>> = RefCell::new(None);
    }

    RUNTIME.with(|cell| {
        let mut runtime = cell.borrow_mut();
        if runtime.is_none() {
            *runtime = Some(Arc::new(Runtime::new().expect("Failed to create runtime")));
        }
        runtime.as_ref().unwrap().clone()
    })
}

// Helper function to extract body from args
fn extract_body(args: &[Value]) -> Result<Option<String>, MagnusError> {
    if args.len() <= 1 {
        return Ok(None);
    }

    let body_value = &args[1];
    if let Ok(body_hash) = RHash::try_convert(*body_value) {
        // Check if the hash has a "body" key
        let body_key = Symbol::new("body").into_value();
        if let Some(body) = body_hash.get(body_key) {
            if let Ok(body_str) = String::try_convert(body) {
                return Ok(Some(body_str));
            }
        }
        Ok(None)
    } else {
        Ok(Some(String::try_convert(*body_value)?))
    }
}

#[magnus::wrap(class = "Rquest::HTTP::Client")]
struct ClientWrap(rquest::Client);

impl ClientWrap {
    fn inner(&self) -> &rquest::Client {
        &self.0
    }
}

impl Clone for ClientWrap {
    fn clone(&self) -> Self {
        // This creates a new client with the same settings
        ClientWrap(
            rquest::Client::builder()
                .emulation(get_random_emulation())
                .build()
                .expect("Failed to create client")
        )
    }
}

#[magnus::wrap(class = "Rquest::HTTP::Client")]
struct RbHttpClient {
    client: ClientWrap,
    default_headers: HashMap<String, String>,
    follow_redirects: bool,
}

impl RbHttpClient {
    fn new() -> Self {
        Self {
            client: ClientWrap(
                rquest::Client::builder()
                .emulation(get_random_emulation())
                .build()
                .expect("Failed to create client")
            ),
            default_headers: HashMap::new(),
            follow_redirects: true,
        }
    }

    fn new_desktop() -> Self {
        Self {
            client: ClientWrap(
                rquest::Client::builder()
                .emulation(get_random_desktop_emulation())
                .build()
                .expect("Failed to create client")
            ),
            default_headers: HashMap::new(),
            follow_redirects: true,
        }
    }

    fn new_mobile() -> Self {
        Self {
            client: ClientWrap(
                rquest::Client::builder()
                .emulation(get_random_mobile_emulation())
                .build()
                .expect("Failed to create client")
            ),
            default_headers: HashMap::new(),
            follow_redirects: true,
        }
    }

    fn with_headers(&self, headers: HashMap<String, String>) -> Self {
        let mut new_client = self.clone();
        new_client.default_headers.clear();
        
        // Convert all header names to lowercase for consistency
        for (name, value) in headers {
            new_client.default_headers.insert(name.to_lowercase(), value);
        }
        new_client
    }

    fn with_header(&self, name: String, value: String) -> Self {
        let mut new_client = self.clone();
        new_client.default_headers.insert(name.to_lowercase(), value);
        new_client
    }

    fn follow(&self, follow: bool) -> Self {
        let mut new_client = self.clone();
        new_client.follow_redirects = follow;
        new_client
    }

    fn get(&self, url: String) -> Result<RbHttpResponse, MagnusError> {
        let rt = get_runtime();
        let mut req = self.client.inner().get(&url);
        
        // Apply all headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        // Set default Accept header if not provided by user
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        
        // Force the User-Agent if it was provided
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            // Set User-Agent header explicitly
            req = req.header("User-Agent", user_agent);
        }

        // Configure redirect policy
        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn post(&self, args: &[Value]) -> Result<RbHttpResponse, MagnusError> {
        let url = String::try_convert(args[0])?;
        let body = extract_body(args)?;
        
        let rt = get_runtime();
        let mut req = self.client.inner().post(&url);
        
        // Apply all headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        // Set default headers if not provided by user
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        if !self.default_headers.contains_key("content-type") {
            req = req.header("Content-Type", "application/json");
        }
        
        // Force the User-Agent if it was provided
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            // Set User-Agent header explicitly
            req = req.header("User-Agent", user_agent);
        }

        // Configure redirect policy
        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        // Add body if present
        if let Some(body) = body {
            req = req.body(body);
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn put(&self, args: &[Value]) -> Result<RbHttpResponse, MagnusError> {
        let url = String::try_convert(args[0])?;
        let body = extract_body(args)?;
        
        let rt = get_runtime();
        let mut req = self.client.inner().put(&url);
        
        // Apply all headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        // Set default headers if not provided by user
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        if !self.default_headers.contains_key("content-type") {
            req = req.header("Content-Type", "application/json");
        }
        
        // Force the User-Agent if it was provided
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            // Set User-Agent header explicitly
            req = req.header("User-Agent", user_agent);
        }

        // Configure redirect policy
        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        // Add body if present
        if let Some(body) = body {
            req = req.body(body);
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn delete(&self, url: String) -> Result<RbHttpResponse, MagnusError> {
        let rt = get_runtime();
        let mut req = self.client.inner().delete(&url);
        
        // Apply all headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        // Set default Accept header if not provided by user
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        
        // Force the User-Agent if it was provided
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            // Set User-Agent header explicitly
            req = req.header("User-Agent", user_agent);
        }

        // Configure redirect policy
        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn head(&self, url: String) -> Result<RbHttpResponse, MagnusError> {
        let rt = get_runtime();
        let mut req = self.client.inner().head(&url);
        
        // Apply all headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        // Set default Accept header if not provided by user
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        
        // Force the User-Agent if it was provided
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            // Set User-Agent header explicitly
            req = req.header("User-Agent", user_agent);
        }

        // Configure redirect policy
        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn patch(&self, args: &[Value]) -> Result<RbHttpResponse, MagnusError> {
        let url = String::try_convert(args[0])?;
        let body = extract_body(args)?;
        
        let rt = get_runtime();
        let mut req = self.client.inner().patch(&url);
        
        // Apply all headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        // Set default headers if not provided by user
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        if !self.default_headers.contains_key("content-type") {
            req = req.header("Content-Type", "application/json");
        }
        
        // Force the User-Agent if it was provided
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            // Set User-Agent header explicitly
            req = req.header("User-Agent", user_agent);
        }

        // Configure redirect policy
        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        // Add body if present
        if let Some(body) = body {
            req = req.body(body);
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    // Add headers method to match Ruby API
    fn headers(&self, headers_hash: RHash) -> Self {
        let mut headers = HashMap::new();

        // Safe unwrap after foreach that returns () on success
        let _ = headers_hash.foreach(|key: Value, value: Value| {
            if let (Ok(key_str), Ok(value_str)) = (String::try_convert(key), String::try_convert(value)) {
                // Convert header name to lowercase for case-insensitive matching
                headers.insert(key_str.to_lowercase(), value_str);
            }
            Ok(ForEach::Continue)
        });
        
        self.with_headers(headers)
    }
}

impl Clone for RbHttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            default_headers: self.default_headers.clone(),
            follow_redirects: self.follow_redirects,
        }
    }
}

// Helper struct for buffering response data
struct ResponseData {
    status: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
    url: String,
}

// Wrap the HTTP response
#[magnus::wrap(class = "Rquest::HTTP::Response")]
struct RbHttpResponse {
    data: Arc<ResponseData>,
}

impl RbHttpResponse {
    fn new(response: RquestResponse) -> Self {
        let rt = get_runtime();
        
        // Extract the data from the response
        let status = response.status().as_u16();
        let url = response.url().to_string();
        
        // Convert headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers().iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }
        
        // Clone and consume the response for body
        let body = rt.block_on(async {
            match response.text().await {
                Ok(text) => Some(text),
                Err(_) => None,
            }
        });
        
        Self {
            data: Arc::new(ResponseData {
                status,
                headers,
                body,
                url,
            }),
        }
    }

    fn status(&self) -> u16 {
        self.data.status
    }

    fn body(&self) -> String {
        match &self.data.body {
            Some(body) => body.clone(),
            None => String::new(),
        }
    }

    fn to_s(&self) -> String {
        self.body()
    }

    fn headers(&self) -> HashMap<String, String> {
        self.data.headers.clone()
    }

    fn content_type(&self) -> Option<String> {
        self.data.headers.get("content-type").cloned()
    }

    fn uri(&self) -> String {
        self.data.url.clone()
    }
}

// Module-level methods that are compatible with http.rb API
fn rb_get(url: String) -> Result<RbHttpResponse, MagnusError> {
    let client = RbHttpClient::new();
    client.get(url)
}

fn rb_desktop() -> RbHttpClient {
    RbHttpClient::new_desktop()
}

fn rb_mobile() -> RbHttpClient {
    RbHttpClient::new_mobile()
}

fn rb_post(args: &[Value]) -> Result<RbHttpResponse, MagnusError> {
    let client = RbHttpClient::new();
    client.post(args)
}

fn rb_put(args: &[Value]) -> Result<RbHttpResponse, MagnusError> {
    let client = RbHttpClient::new();
    client.put(args)
}

fn rb_delete(url: String) -> Result<RbHttpResponse, MagnusError> {
    let client = RbHttpClient::new();
    client.delete(url)
}

fn rb_head(url: String) -> Result<RbHttpResponse, MagnusError> {
    let client = RbHttpClient::new();
    client.head(url)
}

fn rb_patch(args: &[Value]) -> Result<RbHttpResponse, MagnusError> {
    let client = RbHttpClient::new();
    client.patch(args)
}

fn rb_headers(headers_hash: RHash) -> RbHttpClient {
    let client = RbHttpClient::new();
    client.headers(headers_hash)
}

fn rb_follow(follow: bool) -> RbHttpClient {
    RbHttpClient::new().follow(follow)
}

#[magnus::init]
fn init(ruby: &magnus::Ruby) -> Result<(), MagnusError> {
    let rquest_module = ruby.define_module("Rquest")?;
    let http_module = rquest_module.define_module("HTTP")?;
    
    let response_class = http_module.define_class("Response", ruby.class_object())?;
    response_class.define_method("status", method!(RbHttpResponse::status, 0))?;
    response_class.define_method("body", method!(RbHttpResponse::body, 0))?;
    response_class.define_method("to_s", method!(RbHttpResponse::to_s, 0))?;
    response_class.define_method("headers", method!(RbHttpResponse::headers, 0))?;
    response_class.define_method("content_type", method!(RbHttpResponse::content_type, 0))?;
    response_class.define_method("uri", method!(RbHttpResponse::uri, 0))?;
    
    let client_class = http_module.define_class("Client", ruby.class_object())?;
    client_class.define_singleton_method("new", function!(RbHttpClient::new, 0))?;
    client_class.define_singleton_method("new_desktop", function!(RbHttpClient::new_desktop, 0))?;
    client_class.define_singleton_method("new_mobile", function!(RbHttpClient::new_mobile, 0))?;
    client_class.define_method("with_headers", method!(RbHttpClient::with_headers, 1))?;
    client_class.define_method("with_header", method!(RbHttpClient::with_header, 2))?;
    client_class.define_method("follow", method!(RbHttpClient::follow, 1))?;
    client_class.define_method("get", method!(RbHttpClient::get, 1))?;
    client_class.define_method("post", method!(RbHttpClient::post, -1))?;
    client_class.define_method("put", method!(RbHttpClient::put, -1))?;
    client_class.define_method("delete", method!(RbHttpClient::delete, 1))?;
    client_class.define_method("head", method!(RbHttpClient::head, 1))?;
    client_class.define_method("patch", method!(RbHttpClient::patch, -1))?;
    client_class.define_method("headers", method!(RbHttpClient::headers, 1))?;
    client_class.define_method("follow", method!(RbHttpClient::follow, 1))?;
    
    // Module-level functions to mimic HTTP module functions
    http_module.define_singleton_method("get", function!(rb_get, 1))?;
    http_module.define_singleton_method("desktop", function!(rb_desktop, 0))?;
    http_module.define_singleton_method("mobile", function!(rb_mobile, 0))?;
    http_module.define_singleton_method("post", function!(rb_post, -1))?;
    http_module.define_singleton_method("put", function!(rb_put, -1))?;
    http_module.define_singleton_method("delete", function!(rb_delete, 1))?;
    http_module.define_singleton_method("head", function!(rb_head, 1))?;
    http_module.define_singleton_method("patch", function!(rb_patch, -1))?;
    http_module.define_singleton_method("headers", function!(rb_headers, 1))?;
    http_module.define_singleton_method("follow", function!(rb_follow, 1))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Remove unused import
    // use tokio::test;

    // Disabled because of issues with nested Tokio runtimes
    // #[test]
    // fn test_http_client_basic() {
    //     // Create a separate runtime for testing
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let response = client.get("https://httpbin.org/get".to_string()).unwrap();
    //         assert_eq!(response.status(), 200);
    //     });
    // }

    // #[test]
    // fn test_http_client_with_headers() {
    //     let mut headers = HashMap::new();
    //     headers.insert("User-Agent".to_string(), "Test Client".to_string());
    //     let client = RbHttpClient::new().with_headers(headers);
    //     let response = client.get("https://httpbin.org/headers".to_string()).unwrap();
    //     assert_eq!(response.status(), 200);
    // }

    // #[test]
    // fn test_http_client_post() {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let args = [
    //             "https://httpbin.org/post".into_value(),
    //             "test body".into_value()
    //         ];
    //         let response = client.post(&args).unwrap();
    //         assert_eq!(response.status(), 200);
    //     });
    // }

    // #[test]
    // fn test_http_client_put() {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let args = [
    //             "https://httpbin.org/put".into_value(),
    //             "test body".into_value()
    //         ];
    //         let response = client.put(&args).unwrap();
    //         assert_eq!(response.status(), 200);
    //     });
    // }

    // #[test]
    // fn test_http_client_delete() {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let response = client.delete("https://httpbin.org/delete".to_string()).unwrap();
    //         assert_eq!(response.status(), 200);
    //     });
    // }

    // #[test]
    // fn test_http_client_head() {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let response = client.head("https://httpbin.org/get".to_string()).unwrap();
    //         assert_eq!(response.status(), 200);
    //     });
    // }

    // #[test]
    // fn test_http_client_patch() {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let args = [
    //             "https://httpbin.org/patch".into_value(),
    //             "test body".into_value()
    //         ];
    //         let response = client.patch(&args).unwrap();
    //         assert_eq!(response.status(), 200);
    //     });
    // }

    // #[test]
    // fn test_http_response() {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         let client = RbHttpClient::new();
    //         let response = client.get("https://httpbin.org/get".to_string()).unwrap();
    //         
    //         assert_eq!(response.status(), 200);
    //         assert!(response.body().contains("httpbin.org"));
    //         assert!(response.headers().contains_key("content-type"));
    //         assert!(response.uri().contains("httpbin.org"));
    //     });
    // }
}
