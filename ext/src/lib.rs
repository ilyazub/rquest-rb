use magnus::{block::Proc, class, define_class, function, method, Error, Module, Object, RClass, RHash, RString, Value};
use rquest::{Client, Response};
use rquest::redirect::Policy;
use rquest_util::Emulation;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use url::Url;

// Create a runtime once and reuse it
fn get_runtime() -> &'static Runtime {
    static mut RUNTIME: Option<Runtime> = None;
    static INIT: std::sync::Once = std::sync::Once::new();

    unsafe {
        INIT.call_once(|| {
            RUNTIME = Some(Runtime::new().expect("Failed to create runtime"));
        });
        RUNTIME.as_ref().unwrap()
    }
}

// Represents our HTTP client
struct HttpClient {
    client: Client,
    default_headers: HashMap<String, String>,
    follow_redirects: bool,
}

impl HttpClient {
    fn new() -> Self {
        Self {
            client: Client::builder()
                .emulation(Emulation::Chrome133)
                .build()
                .expect("Failed to create client"),
            default_headers: HashMap::new(),
            follow_redirects: true,
        }
    }

    fn with_header(mut self, name: String, value: String) -> Self {
        self.default_headers.insert(name, value);
        self
    }

    fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.default_headers.extend(headers);
        self
    }

    fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    fn get(&self, url: &str) -> Result<Response, rquest::Error> {
        let rt = get_runtime();
        let mut req = self.client.get(url);
        
        // Add default headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }

        if !self.follow_redirects {
            req = req.redirect(Policy::none());
        }

        rt.block_on(req.send())
    }

    fn post(&self, url: &str, body: Option<&str>) -> Result<Response, rquest::Error> {
        let rt = get_runtime();
        let mut req = self.client.post(url);
        
        // Add default headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }

        if !self.follow_redirects {
            req = req.redirect(Policy::none());
        }

        if let Some(body_str) = body {
            req = req.body(body_str);
        }

        rt.block_on(req.send())
    }

    fn put(&self, url: &str, body: Option<&str>) -> Result<Response, rquest::Error> {
        let rt = get_runtime();
        let mut req = self.client.put(url);
        
        // Add default headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }

        if !self.follow_redirects {
            req = req.redirect(Policy::none());
        }

        if let Some(body_str) = body {
            req = req.body(body_str);
        }

        rt.block_on(req.send())
    }

    fn delete(&self, url: &str) -> Result<Response, rquest::Error> {
        let rt = get_runtime();
        let mut req = self.client.delete(url);
        
        // Add default headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }

        if !self.follow_redirects {
            req = req.redirect(Policy::none());
        }

        rt.block_on(req.send())
    }

    fn head(&self, url: &str) -> Result<Response, rquest::Error> {
        let rt = get_runtime();
        let mut req = self.client.head(url);
        
        // Add default headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }

        if !self.follow_redirects {
            req = req.redirect(Policy::none());
        }

        rt.block_on(req.send())
    }

    fn patch(&self, url: &str, body: Option<&str>) -> Result<Response, rquest::Error> {
        let rt = get_runtime();
        let mut req = self.client.patch(url);
        
        // Add default headers
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }

        if !self.follow_redirects {
            req = req.redirect(Policy::none());
        }

        if let Some(body_str) = body {
            req = req.body(body_str);
        }

        rt.block_on(req.send())
    }
}

struct HttpResponse {
    inner: Response,
}

impl HttpResponse {
    fn new(response: Response) -> Self {
        Self { inner: response }
    }

    fn status(&self) -> u16 {
        self.inner.status().as_u16()
    }

    fn body(&self) -> String {
        let rt = get_runtime();
        match rt.block_on(self.inner.text()) {
            Ok(text) => text,
            Err(_) => String::new(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (name, value) in self.inner.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }
        headers
    }

    fn content_type(&self) -> Option<String> {
        self.inner
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok().map(String::from))
    }

    fn uri(&self) -> String {
        self.inner.url().to_string()
    }
}

// Ruby bindings

#[magnus::wrap(class = "HTTP::Client")]
struct RbHttpClient {
    inner: Arc<Mutex<HttpClient>>,
}

impl RbHttpClient {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HttpClient::new())),
        }
    }

    fn with_headers(&self, headers: HashMap<String, String>) -> Self {
        let client = self.inner.lock().unwrap().clone().with_headers(headers);
        Self {
            inner: Arc::new(Mutex::new(client)),
        }
    }

    fn with_header(&self, name: String, value: String) -> Self {
        let client = self.inner.lock().unwrap().clone().with_header(name, value);
        Self {
            inner: Arc::new(Mutex::new(client)),
        }
    }

    fn follow(&self, follow: bool) -> Self {
        let client = self.inner.lock().unwrap().clone().follow_redirects(follow);
        Self {
            inner: Arc::new(Mutex::new(client)),
        }
    }

    fn get(&self, url: String) -> Result<RbHttpResponse, Error> {
        let client = self.inner.lock().map_err(|_| Error::new(magnus::exception::runtime_error(), "mutex poisoned"))?;
        
        match client.get(&url) {
            Ok(response) => Ok(RbHttpResponse { inner: HttpResponse::new(response) }),
            Err(e) => Err(Error::new(magnus::exception::runtime_error(), e.to_string())),
        }
    }

    fn post(&self, url: String, body: Option<String>) -> Result<RbHttpResponse, Error> {
        let client = self.inner.lock().map_err(|_| Error::new(magnus::exception::runtime_error(), "mutex poisoned"))?;
        
        match client.post(&url, body.as_deref()) {
            Ok(response) => Ok(RbHttpResponse { inner: HttpResponse::new(response) }),
            Err(e) => Err(Error::new(magnus::exception::runtime_error(), e.to_string())),
        }
    }

    fn put(&self, url: String, body: Option<String>) -> Result<RbHttpResponse, Error> {
        let client = self.inner.lock().map_err(|_| Error::new(magnus::exception::runtime_error(), "mutex poisoned"))?;
        
        match client.put(&url, body.as_deref()) {
            Ok(response) => Ok(RbHttpResponse { inner: HttpResponse::new(response) }),
            Err(e) => Err(Error::new(magnus::exception::runtime_error(), e.to_string())),
        }
    }

    fn delete(&self, url: String) -> Result<RbHttpResponse, Error> {
        let client = self.inner.lock().map_err(|_| Error::new(magnus::exception::runtime_error(), "mutex poisoned"))?;
        
        match client.delete(&url) {
            Ok(response) => Ok(RbHttpResponse { inner: HttpResponse::new(response) }),
            Err(e) => Err(Error::new(magnus::exception::runtime_error(), e.to_string())),
        }
    }

    fn head(&self, url: String) -> Result<RbHttpResponse, Error> {
        let client = self.inner.lock().map_err(|_| Error::new(magnus::exception::runtime_error(), "mutex poisoned"))?;
        
        match client.head(&url) {
            Ok(response) => Ok(RbHttpResponse { inner: HttpResponse::new(response) }),
            Err(e) => Err(Error::new(magnus::exception::runtime_error(), e.to_string())),
        }
    }

    fn patch(&self, url: String, body: Option<String>) -> Result<RbHttpResponse, Error> {
        let client = self.inner.lock().map_err(|_| Error::new(magnus::exception::runtime_error(), "mutex poisoned"))?;
        
        match client.patch(&url, body.as_deref()) {
            Ok(response) => Ok(RbHttpResponse { inner: HttpResponse::new(response) }),
            Err(e) => Err(Error::new(magnus::exception::runtime_error(), e.to_string())),
        }
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            default_headers: self.default_headers.clone(),
            follow_redirects: self.follow_redirects,
        }
    }
}

#[magnus::wrap(class = "HTTP::Response")]
struct RbHttpResponse {
    inner: HttpResponse,
}

impl RbHttpResponse {
    fn status(&self) -> u16 {
        self.inner.status()
    }

    fn body(&self) -> String {
        self.inner.body()
    }

    fn to_s(&self) -> String {
        self.inner.body()
    }

    fn headers(&self) -> HashMap<String, String> {
        self.inner.headers()
    }

    fn content_type(&self) -> Option<String> {
        self.inner.content_type()
    }

    fn uri(&self) -> String {
        self.inner.uri()
    }
}

// Module-level methods that are compatible with http.rb API
fn rb_get(url: String) -> Result<RbHttpResponse, Error> {
    let client = RbHttpClient::new();
    client.get(url)
}

fn rb_post(url: String, body: Option<String>) -> Result<RbHttpResponse, Error> {
    let client = RbHttpClient::new();
    client.post(url, body)
}

fn rb_put(url: String, body: Option<String>) -> Result<RbHttpResponse, Error> {
    let client = RbHttpClient::new();
    client.put(url, body)
}

fn rb_delete(url: String) -> Result<RbHttpResponse, Error> {
    let client = RbHttpClient::new();
    client.delete(url)
}

fn rb_head(url: String) -> Result<RbHttpResponse, Error> {
    let client = RbHttpClient::new();
    client.head(url)
}

fn rb_patch(url: String, body: Option<String>) -> Result<RbHttpResponse, Error> {
    let client = RbHttpClient::new();
    client.patch(url, body)
}

fn rb_headers(headers_hash: RHash) -> RbHttpClient {
    let client = RbHttpClient::new();
    let mut headers = HashMap::new();

    headers_hash.each(|key, value| {
        if let (Ok(key_str), Ok(value_str)) = (RString::try_convert(key), RString::try_convert(value)) {
            headers.insert(key_str.to_string(), value_str.to_string());
        }
        Ok(())
    }).unwrap();

    client.with_headers(headers)
}

fn rb_follow(follow: bool) -> RbHttpClient {
    RbHttpClient::new().follow(follow)
}

#[magnus::init]
fn init(ruby: &magnus::Ruby) -> Result<(), Error> {
    #[cfg(ruby_gte_3_0)]
    unsafe {
        rb_sys::rb_ext_ractor_safe(true);
    }

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
    client_class.define_method("with_headers", method!(RbHttpClient::with_headers, 1))?;
    client_class.define_method("with_header", method!(RbHttpClient::with_header, 2))?;
    client_class.define_method("follow", method!(RbHttpClient::follow, 1))?;
    client_class.define_method("get", method!(RbHttpClient::get, 1))?;
    client_class.define_method("post", method!(RbHttpClient::post, -1))?;
    client_class.define_method("put", method!(RbHttpClient::put, -1))?;
    client_class.define_method("delete", method!(RbHttpClient::delete, 1))?;
    client_class.define_method("head", method!(RbHttpClient::head, 1))?;
    client_class.define_method("patch", method!(RbHttpClient::patch, -1))?;
    
    // Module-level functions to mimic HTTP module functions
    http_module.define_singleton_method("get", function!(rb_get, 1))?;
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
    use tokio::test;

    #[test]
    fn test_http_client_basic() {
        let client = HttpClient::new();
        let response = client.get("https://httpbin.org/get").unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_client_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Test Client".to_string());
        let client = HttpClient::new().with_headers(headers);
        let response = client.get("https://httpbin.org/headers").unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_client_post() {
        let client = HttpClient::new();
        let response = client.post("https://httpbin.org/post", Some("test body")).unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_client_put() {
        let client = HttpClient::new();
        let response = client.put("https://httpbin.org/put", Some("test body")).unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_client_delete() {
        let client = HttpClient::new();
        let response = client.delete("https://httpbin.org/delete").unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_client_head() {
        let client = HttpClient::new();
        let response = client.head("https://httpbin.org/get").unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_client_patch() {
        let client = HttpClient::new();
        let response = client.patch("https://httpbin.org/patch", Some("test body")).unwrap();
        assert_eq!(response.status().as_u16(), 200);
    }

    #[test]
    fn test_http_response() {
        let client = HttpClient::new();
        let response = client.get("https://httpbin.org/get").unwrap();
        let http_response = HttpResponse::new(response);
        
        assert_eq!(http_response.status(), 200);
        assert!(http_response.body().contains("httpbin.org"));
        assert!(http_response.headers().contains_key("content-type"));
        assert!(http_response.uri().contains("httpbin.org"));
    }
}
