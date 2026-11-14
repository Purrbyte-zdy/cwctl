use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct Aria2Client {
    endpoint: String,
    secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<String>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl Aria2Client {
    pub fn from_environment() -> Result<Self, String> {
        let endpoint = std::env::var("CWCTL_ARIA2_RPC")
            .unwrap_or_else(|_| "http://127.0.0.1:6800/jsonrpc".to_owned());
        let secret = std::env::var("CWCTL_ARIA2_SECRET").ok();
        Self::new(endpoint, secret)
    }

    pub fn new(endpoint: impl Into<String>, secret: Option<String>) -> Result<Self, String> {
        let endpoint = endpoint.into();
        if !(endpoint.starts_with("http://") || endpoint.starts_with("https://")) {
            return Err("error: aria2 RPC endpoint must be an HTTP(S) URL".to_owned());
        }
        Ok(Self { endpoint, secret })
    }

    pub fn add_uri(&self, uri: &str) -> Result<String, String> {
        let mut params = Vec::new();
        if let Some(secret) = &self.secret {
            params.push(Value::String(format!("token:{secret}")));
        }
        params.push(json!([uri]));

        let request = json!({
            "jsonrpc": "2.0",
            "id": "cwctl",
            "method": "aria2.addUri",
            "params": params,
        });
        let response = ureq::post(&self.endpoint)
            .send_json(request)
            .map_err(|error| format!("error: aria2 RPC request failed: {error}"))?;
        let rpc: RpcResponse = response
            .into_body()
            .read_json()
            .map_err(|error| format!("error: invalid aria2 RPC response: {error}"))?;

        match (rpc.result, rpc.error) {
            (Some(gid), None) => Ok(gid),
            (_, Some(error)) => Err(format!(
                "error: aria2 RPC error {}: {}",
                error.code, error.message
            )),
            _ => Err("error: aria2 RPC response did not contain a result".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Aria2Client;

    #[test]
    fn validates_endpoint() {
        assert!(Aria2Client::new("http://127.0.0.1:6800/jsonrpc", None).is_ok());
        assert!(Aria2Client::new("file:///tmp/aria2", None).is_err());
    }
}
