use magnus::{function, method, Error as MagnusError, Module, Object, RHash, Value, exception, TryConvert, Symbol, IntoValue};
use magnus::r_hash::ForEach;
use rquest::{Response as RquestResponse, Error as RquestError};
use rquest::redirect::Policy;
use rquest_util::{Emulation as RquestEmulation};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::cell::{Cell, RefCell};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::num::Wrapping;
use std::time::Duration;

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

fn get_random_mobile_emulation() -> RquestEmulation {
    let browsers = [
        RquestEmulation::SafariIos17_4_1,
        RquestEmulation::SafariIos17_2,
        RquestEmulation::SafariIos16_5,
        RquestEmulation::FirefoxAndroid135
    ];
    
    let index = (fast_random() as usize) % browsers.len();
    browsers[index]
}

fn get_random_emulation() -> RquestEmulation {
    if fast_random() % 100 < 50 {
        get_random_desktop_emulation()
    } else {
        get_random_mobile_emulation()
    }
}

fn rquest_error_to_magnus_error(err: RquestError) -> MagnusError {
    MagnusError::new(exception::runtime_error(), format!("HTTP request failed: {}", err))
}

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

fn extract_body(args: &[Value]) -> Result<Option<String>, MagnusError> {
    if args.len() <= 1 {
        return Ok(None);
    }

    let body_value = &args[1];
    if let Ok(body_hash) = RHash::try_convert(*body_value) {
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
    fn clone(&self) -> Self {            // This creates a new client with the same settings
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
    proxy: Option<String>,
    timeout: Option<Duration>,
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
            proxy: None,
            timeout: None,
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
            proxy: None,
            timeout: None,
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
            proxy: None,
            timeout: None,
        }
    }

    fn with_headers(&self, headers: HashMap<String, String>) -> Self {
        let mut new_client = self.clone();
        new_client.default_headers.clear();
        
        for (name, value) in headers {
            new_client.default_headers.insert(name.to_lowercase(), value);
        }
        new_client
    }

    fn with_proxy(&self, proxy: String) -> Self {
        let mut new_client = self.clone();
        new_client.proxy = Some(proxy.clone());
        
        new_client.client = ClientWrap(
            rquest::Client::builder()
                .emulation(get_random_emulation())
                .proxy(proxy)
                .build()
                .expect("Failed to create client with proxy")
        );
        
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
        
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            req = req.header("User-Agent", user_agent);
        }

        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        if let Some(timeout) = self.timeout {
            req = req.timeout(timeout);
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
        
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        if !self.default_headers.contains_key("content-type") {
            req = req.header("Content-Type", "application/json");
        }
        
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            req = req.header("User-Agent", user_agent);
        }

        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        if let Some(timeout) = self.timeout {
            req = req.timeout(timeout);
        }

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
        
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        if !self.default_headers.contains_key("content-type") {
            req = req.header("Content-Type", "application/json");
        }
        
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            req = req.header("User-Agent", user_agent);
        }

        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        if let Some(timeout) = self.timeout {
            req = req.timeout(timeout);
        }

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
        
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            req = req.header("User-Agent", user_agent);
        }

        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        if let Some(timeout) = self.timeout {
            req = req.timeout(timeout);
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn head(&self, url: String) -> Result<RbHttpResponse, MagnusError> {
        let rt = get_runtime();
        let mut req = self.client.inner().head(&url);
        
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            req = req.header("User-Agent", user_agent);
        }

        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }

        if let Some(timeout) = self.timeout {
            req = req.timeout(timeout);
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
        
        for (name, value) in &self.default_headers {
            req = req.header(name, value);
        }
        
        if !self.default_headers.contains_key("accept") {
            req = req.header("Accept", "application/json");
        }
        if !self.default_headers.contains_key("content-type") {
            req = req.header("Content-Type", "application/json");
        }
        
        if let Some(user_agent) = self.default_headers.get("user-agent") {
            req = req.header("User-Agent", user_agent);
        }

        if self.follow_redirects {
            req = req.redirect(Policy::limited(10));
        } else {
            req = req.redirect(Policy::none());
        }
        
        if let Some(timeout) = self.timeout {
            req = req.timeout(timeout);
        }

        if let Some(body) = body {
            req = req.body(body);
        }

        match rt.block_on(req.send()) {
            Ok(response) => Ok(RbHttpResponse::new(response)),
            Err(e) => Err(rquest_error_to_magnus_error(e)),
        }
    }

    fn headers(&self, headers_hash: RHash) -> Self {
        let mut headers = HashMap::new();

        let _ = headers_hash.foreach(|key: Value, value: Value| {
            if let (Ok(key_str), Ok(value_str)) = (String::try_convert(key), String::try_convert(value)) {
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
            proxy: self.proxy.clone(),
            timeout: self.timeout,
        }
    }
}

struct ResponseData {
    status: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
    url: String,
}

#[magnus::wrap(class = "Rquest::HTTP::Response")]
struct RbHttpResponse {
    data: Arc<ResponseData>,
}

impl RbHttpResponse {
    fn new(response: RquestResponse) -> Self {
        let rt = get_runtime();
        
        let status = response.status().as_u16();
        let url = response.url().to_string();
        
        let mut headers = HashMap::new();
        for (name, value) in response.headers().iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }
        
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

    fn code(&self) -> u16 {
        self.status()
    }

    fn charset(&self) -> Option<String> {
        if let Some(content_type) = self.content_type() {
            if let Some(charset_part) = content_type.split(';').skip(1)
                .find(|part| part.trim().to_lowercase().starts_with("charset=")) {
                
                let charset = charset_part.trim()
                    .split('=')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                
                if !charset.is_empty() {
                    return Some(charset);
                }
            }
        }
        None
    }
}

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

fn rb_proxy(proxy: String) -> RbHttpClient {
    RbHttpClient::new().with_proxy(proxy)
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
    response_class.define_method("code", method!(RbHttpResponse::code, 0))?;
    response_class.define_method("charset", method!(RbHttpResponse::charset, 0))?;
    
    let client_class = http_module.define_class("Client", ruby.class_object())?;
    client_class.define_singleton_method("new", function!(RbHttpClient::new, 0))?;
    client_class.define_singleton_method("new_desktop", function!(RbHttpClient::new_desktop, 0))?;
    client_class.define_singleton_method("new_mobile", function!(RbHttpClient::new_mobile, 0))?;
    client_class.define_method("with_headers", method!(RbHttpClient::with_headers, 1))?;
    client_class.define_method("follow", method!(RbHttpClient::follow, 1))?;
    client_class.define_method("with_proxy", method!(RbHttpClient::with_proxy, 1))?;
    client_class.define_method("get", method!(RbHttpClient::get, 1))?;
    client_class.define_method("post", method!(RbHttpClient::post, -1))?;
    client_class.define_method("put", method!(RbHttpClient::put, -1))?;
    client_class.define_method("delete", method!(RbHttpClient::delete, 1))?;
    client_class.define_method("head", method!(RbHttpClient::head, 1))?;
    client_class.define_method("patch", method!(RbHttpClient::patch, -1))?;
    client_class.define_method("headers", method!(RbHttpClient::headers, 1))?;
    
    http_module.define_module_function("get", function!(rb_get, 1))?;
    http_module.define_module_function("desktop", function!(rb_desktop, 0))?;
    http_module.define_module_function("mobile", function!(rb_mobile, 0))?;
    http_module.define_module_function("post", function!(rb_post, -1))?;
    http_module.define_module_function("put", function!(rb_put, -1))?;
    http_module.define_module_function("delete", function!(rb_delete, 1))?;
    http_module.define_module_function("head", function!(rb_head, 1))?;
    http_module.define_module_function("patch", function!(rb_patch, -1))?;
    http_module.define_module_function("headers", function!(rb_headers, 1))?;
    http_module.define_module_function("follow", function!(rb_follow, 1))?;
    http_module.define_module_function("proxy", function!(rb_proxy, 1))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_ruby() {
        INIT.call_once(|| {
            unsafe {
                magnus::embed::init();
            }
        });
    }

    // Helper function to get a random proxy from the list
    fn get_random_proxy() -> Option<String> {
        let rt = get_runtime();
        
        rt.block_on(async {
            // Create a client without proxy to fetch the proxy list
            let client = rquest::Client::new();
            
            match client.get("https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/http.txt")
                .send()
                .await {
                    Ok(response) => {
                        if let Ok(text) = response.text().await {
                            // Split into lines and filter out empty lines
                            let proxies: Vec<&str> = text.lines()
                                .filter(|line| !line.trim().is_empty())
                                .collect();
                            
                            if !proxies.is_empty() {
                                // Get a random proxy from the list
                                let index = (fast_random() as usize) % proxies.len();
                                Some(format!("http://{}", proxies[index].trim()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None
                }
        })
    }

    #[test]
    fn test_http_client_basic() {
        init_ruby();
        let response = RbHttpClient::new().get("https://httpbin.org/get".to_string()).unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_http_client_with_proxy() {
        init_ruby();
        if let Some(proxy) = get_random_proxy() {
            println!("Testing with proxy: {}", proxy);
            
            let client = RbHttpClient::new().with_proxy(proxy);
            match client.get("https://httpbin.org/ip".to_string()) {
                Ok(response) => {
                    assert!(response.status() == 200 || response.status() == 407 || response.status() == 403,
                           "Expected status 200, 407 (proxy auth required), or 403 (forbidden), got {}",
                           response.status());
                    println!("Proxy test response status: {}", response.status());
                    println!("Response body: {}", response.body());
                }
                Err(e) => {
                    println!("Proxy test failed with error: {}", e);
                    // Don't fail the test as the proxy might be unavailable
                }
            }
        } else {
            println!("Skipping proxy test - could not fetch proxy list");
        }
    }

    #[test]
    fn test_http_client_post() {
        init_ruby();
        let client = RbHttpClient::new();
        let args = [
            String::from("https://httpbin.org/post").into_value(),
            String::from("test body").into_value()
        ];
        let response = client.post(&args).unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_http_client_put() {
        init_ruby();
        let client = RbHttpClient::new();
        let args = [
            String::from("https://httpbin.org/put").into_value(),
            String::from("test body").into_value()
        ];
        let response = client.put(&args).unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_http_client_delete() {
        init_ruby();
        let response = RbHttpClient::new().delete("https://httpbin.org/delete".to_string()).unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_http_client_head() {
        init_ruby();
        let response = RbHttpClient::new().head("https://httpbin.org/get".to_string()).unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_http_client_patch() {
        init_ruby();
        let client = RbHttpClient::new();
        let args = [
            String::from("https://httpbin.org/patch").into_value(),
            String::from("test body").into_value()
        ];
        let response = client.patch(&args).unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_http_response() {
        init_ruby();
        let client = RbHttpClient::new();
        let response = client.get("https://httpbin.org/get".to_string()).unwrap();
        
        assert_eq!(response.status(), 200);
        assert!(response.body().contains("httpbin.org"));
        assert!(response.headers().contains_key("content-type"));
        assert!(response.uri().contains("httpbin.org"));
    }
}
