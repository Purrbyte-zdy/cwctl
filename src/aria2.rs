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
    use std::{
        io::{ErrorKind, Read, Write},
        net::TcpListener,
        thread,
    };

    fn start_server(body: &'static str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let endpoint = format!(
            "http://{}/jsonrpc",
            listener
                .local_addr()
                .expect("test server should have an address")
        );
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test server should accept");
            stream
                .set_read_timeout(Some(std::time::Duration::from_millis(100)))
                .expect("test server should set timeout");
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                let size = match stream.read(&mut buffer) {
                    Ok(size) => size,
                    Err(error)
                        if error.kind() == ErrorKind::WouldBlock
                            || error.kind() == ErrorKind::TimedOut =>
                    {
                        break
                    }
                    Err(error) => panic!("test server should read request: {error}"),
                };
                if size == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..size]);
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("test server should write response");
            String::from_utf8(request).expect("request should be UTF-8")
        });
        (endpoint, handle)
    }

    #[test]
    fn validates_endpoint() {
        assert!(Aria2Client::new("http://127.0.0.1:6800/jsonrpc", None).is_ok());
        assert!(Aria2Client::new("file:///tmp/aria2", None).is_err());
    }

    #[test]
    fn submits_add_uri_request_and_returns_gid() {
        let (endpoint, server) =
            start_server("{\"jsonrpc\":\"2.0\",\"id\":\"cwctl\",\"result\":\"gid\"}");
        let client = Aria2Client::new(endpoint, Some("secret".to_owned()))
            .expect("client should accept test endpoint");

        assert_eq!(
            client
                .add_uri("https://example.test/plugin.cwplugin")
                .expect("RPC request should succeed"),
            "gid"
        );
        let request = server.join().expect("test server should finish");
        assert!(request.contains("aria2.addUri"));
        assert!(request.contains("token:secret"));
        assert!(request.contains("https://example.test/plugin.cwplugin"));
    }

    #[test]
    fn returns_rpc_errors() {
        let (endpoint, server) = start_server(
            "{\"jsonrpc\":\"2.0\",\"id\":\"cwctl\",\"error\":{\"code\":1,\"message\":\"bad URI\"}}",
        );
        let client = Aria2Client::new(endpoint, None).expect("client should accept test endpoint");

        let error = client
            .add_uri("not-a-url")
            .expect_err("RPC error should be returned");

        assert!(error.contains("aria2 RPC error 1: bad URI"));
        server.join().expect("test server should finish");
    }
}
