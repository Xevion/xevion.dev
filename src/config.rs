use clap::Parser;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::str::FromStr;

/// Server configuration parsed from CLI arguments and environment variables
#[derive(Parser, Debug)]
#[command(name = "api")]
#[command(about = "xevion.dev API server with ISR caching", long_about = None)]
pub struct Args {
    /// Address(es) to listen on. Can be host:port, :port, or Unix socket path.
    /// Can be specified multiple times.
    /// Examples: :8080, 0.0.0.0:8080, [::]:8080, /tmp/api.sock
    #[arg(long, env = "LISTEN_ADDR", value_delimiter = ',', required = true)]
    pub listen: Vec<ListenAddr>,

    /// Downstream Bun SSR server URL or Unix socket path
    /// Examples: http://localhost:5173, /tmp/bun.sock
    #[arg(long, env = "DOWNSTREAM_URL", required = true)]
    pub downstream: String,

    /// Optional header name to trust for request IDs (e.g., X-Railway-Request-Id)
    #[arg(long, env = "TRUST_REQUEST_ID")]
    pub trust_request_id: Option<String>,
}

/// Address to listen on - either TCP or Unix socket
#[derive(Debug, Clone)]
pub enum ListenAddr {
    Tcp(SocketAddr),
    Unix(PathBuf),
}

impl FromStr for ListenAddr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Unix socket: starts with / or ./
        if s.starts_with('/') || s.starts_with("./") {
            return Ok(ListenAddr::Unix(PathBuf::from(s)));
        }

        // Shorthand :port -> 127.0.0.1:port
        if let Some(port_str) = s.strip_prefix(':') {
            let port: u16 = port_str
                .parse()
                .map_err(|_| format!("Invalid port number: {}", port_str))?;
            return Ok(ListenAddr::Tcp(SocketAddr::from(([127, 0, 0, 1], port))));
        }

        // Try parsing as a socket address (handles both IPv4 and IPv6)
        // This supports formats like: 0.0.0.0:8080, [::]:8080, 192.168.1.1:3000
        match s.parse::<SocketAddr>() {
            Ok(addr) => Ok(ListenAddr::Tcp(addr)),
            Err(_) => {
                // Try resolving as hostname:port
                match s.to_socket_addrs() {
                    Ok(mut addrs) => addrs
                        .next()
                        .ok_or_else(|| format!("Could not resolve address: {}", s))
                        .map(ListenAddr::Tcp),
                    Err(_) => Err(format!(
                        "Invalid address '{}'. Expected host:port, :port, or Unix socket path",
                        s
                    )),
                }
            }
        }
    }
}

impl std::fmt::Display for ListenAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListenAddr::Tcp(addr) => write!(f, "{}", addr),
            ListenAddr::Unix(path) => write!(f, "{}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shorthand_port() {
        let addr: ListenAddr = ":8080".parse().unwrap();
        match addr {
            ListenAddr::Tcp(socket) => {
                assert_eq!(socket.port(), 8080);
                assert_eq!(socket.ip().to_string(), "127.0.0.1");
            }
            _ => panic!("Expected TCP address"),
        }
    }

    #[test]
    fn test_parse_ipv4() {
        let addr: ListenAddr = "0.0.0.0:8080".parse().unwrap();
        match addr {
            ListenAddr::Tcp(socket) => {
                assert_eq!(socket.port(), 8080);
                assert_eq!(socket.ip().to_string(), "0.0.0.0");
            }
            _ => panic!("Expected TCP address"),
        }
    }

    #[test]
    fn test_parse_ipv6() {
        let addr: ListenAddr = "[::]:8080".parse().unwrap();
        match addr {
            ListenAddr::Tcp(socket) => {
                assert_eq!(socket.port(), 8080);
                assert_eq!(socket.ip().to_string(), "::");
            }
            _ => panic!("Expected TCP address"),
        }
    }

    #[test]
    fn test_parse_unix_socket() {
        let addr: ListenAddr = "/tmp/api.sock".parse().unwrap();
        match addr {
            ListenAddr::Unix(path) => {
                assert_eq!(path, PathBuf::from("/tmp/api.sock"));
            }
            _ => panic!("Expected Unix socket"),
        }
    }

    #[test]
    fn test_parse_relative_unix_socket() {
        let addr: ListenAddr = "./api.sock".parse().unwrap();
        match addr {
            ListenAddr::Unix(path) => {
                assert_eq!(path, PathBuf::from("./api.sock"));
            }
            _ => panic!("Expected Unix socket"),
        }
    }
}
