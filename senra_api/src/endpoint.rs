use http::Method;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub path: String,
    pub method: Method,
    pub body: Option<Value>,
    pub params: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
}

impl Endpoint {
    pub fn new(path: &'static str) -> Self {
        Self {
            path: path.to_string(),
            method: Method::GET,
            body: None,
            params: Vec::new(),
            query: Vec::new(),
        }
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn with_param(mut self, key: &str, value: impl ToString) -> Self {
        self.params.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_query(mut self, key: &str, value: impl ToString) -> Self {
        self.query.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_body<T: Serialize>(mut self, body: T) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_value(body)?);
        Ok(self)
    }
}
