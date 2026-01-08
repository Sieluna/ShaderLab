use core::time::Duration;

use url::Url;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub http_url: Url,
    pub ws_url: Url,
    pub timeout: Duration,
}

impl ClientConfig {
    pub fn new(base_url: Url, tls: bool) -> Self {
        let mut http_url = base_url.clone();
        http_url
            .set_scheme(if tls { "https" } else { "http" })
            .unwrap();
        let mut ws_url = base_url.clone();
        ws_url.set_scheme(if tls { "wss" } else { "ws" }).unwrap();

        Self {
            http_url,
            ws_url,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_http_url(mut self, http_url: Url, tls: bool) -> Self {
        let mut http_url = http_url.clone();
        http_url
            .set_scheme(if tls { "https" } else { "http" })
            .unwrap();
        self.http_url = http_url;
        self
    }

    pub fn with_ws_url(mut self, ws_url: Url, tls: bool) -> Self {
        let mut ws_url = ws_url.clone();
        ws_url.set_scheme(if tls { "wss" } else { "ws" }).unwrap();
        self.ws_url = ws_url;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
