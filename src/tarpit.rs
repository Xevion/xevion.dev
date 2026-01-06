use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use futures::stream::{self, Stream};
use rand::Rng;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct TarpitConfig {
    pub enabled: bool,
    pub delay_min_ms: u64,
    pub delay_max_ms: u64,
    pub chunk_size_min: usize,
    pub chunk_size_max: usize,
    pub max_global_connections: usize,
    pub max_connections_per_ip: usize,
}

impl TarpitConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("TARPIT_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            delay_min_ms: std::env::var("TARPIT_DELAY_MIN_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            delay_max_ms: std::env::var("TARPIT_DELAY_MAX_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            chunk_size_min: std::env::var("TARPIT_CHUNK_MIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(64),
            chunk_size_max: std::env::var("TARPIT_CHUNK_MAX")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
            max_global_connections: std::env::var("TARPIT_MAX_GLOBAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            max_connections_per_ip: std::env::var("TARPIT_MAX_PER_IP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        }
    }
}

pub struct TarpitState {
    global_semaphore: Arc<Semaphore>,
    ip_connections: Arc<DashMap<IpAddr, Arc<Semaphore>>>,
    pub config: Arc<TarpitConfig>,
}

impl TarpitState {
    pub fn new(config: TarpitConfig) -> Self {
        let config = Arc::new(config);
        Self {
            global_semaphore: Arc::new(Semaphore::new(config.max_global_connections)),
            ip_connections: Arc::new(DashMap::new()),
            config,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ResponseMode {
    RandomBytes,
    FakeHtml,
    FakeJson,
}

impl ResponseMode {
    fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..3) {
            0 => Self::RandomBytes,
            1 => Self::FakeHtml,
            _ => Self::FakeJson,
        }
    }

    fn content_type(&self) -> &'static str {
        match self {
            Self::RandomBytes => "application/octet-stream",
            Self::FakeHtml => "text/html; charset=utf-8",
            Self::FakeJson => "application/json",
        }
    }
}

pub fn is_malicious_path(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // File extension checks
    if path_lower.ends_with(".php")
        || path_lower.ends_with(".asp")
        || path_lower.ends_with(".aspx")
        || path_lower.ends_with(".sql")
        || path_lower.ends_with(".zip")
        || path_lower.ends_with(".tar")
        || path_lower.ends_with(".tar.gz")
        || path_lower.ends_with(".rar")
        || path_lower.ends_with(".backup")
    {
        return true;
    }

    // WordPress paths
    if path_lower.starts_with("/wp-") || path_lower.starts_with("/wordpress/") {
        return true;
    }

    // Admin panels
    if path_lower.starts_with("/administrator")
        || path_lower.contains("phpmyadmin")
    {
        return true;
    }

    // Config and credential files
    if path_lower.starts_with("/.env")
        || path_lower.contains("/config.")
        || path_lower.contains("/.git/")
        || path_lower.contains("/.svn/")
        || path_lower.contains("/.hg/")
        || path_lower.contains("/.bzr/")
        || path_lower.contains("/credentials")
        || path_lower.contains("service-account.json")
        || path_lower.contains("firebase")
        || path_lower.contains("/.aws/")
        || path_lower.contains("/.kube/")
    {
        return true;
    }

    // CGI and old web tech
    if path_lower.starts_with("/cgi-bin/") {
        return true;
    }

    // Spring Boot actuators
    if path_lower.starts_with("/actuator") {
        return true;
    }

    // API documentation/explorers
    if path_lower.starts_with("/api-docs")
        || path_lower.starts_with("/swagger")
        || path_lower.starts_with("/graphql")
        || path_lower.starts_with("/graphiql")
        || path_lower.starts_with("/playground")
    {
        return true;
    }

    // Infrastructure files
    if path_lower.contains("/terraform.")
        || path_lower.contains("dockerfile")
        || path_lower.contains("docker-compose")
        || path_lower.contains("/backup")
    {
        return true;
    }

    // Package manager files (except those we might legitimately serve)
    if path_lower.contains("composer.json")
        || path_lower.contains("composer.lock")
        || path_lower.contains("gemfile")
        || path_lower.contains("pipfile")
    {
        return true;
    }

    false
}

pub fn extract_client_ip(headers: &HeaderMap, peer_addr: Option<SocketAddr>) -> IpAddr {
    // Check X-Real-IP first (Railway sets this)
    if let Some(real_ip) = headers.get("x-real-ip")
        && let Ok(ip_str) = real_ip.to_str()
        && let Ok(ip) = ip_str.parse()
    {
        return ip;
    }

    // Fallback to X-Forwarded-For (take first IP)
    if let Some(forwarded) = headers.get("x-forwarded-for")
        && let Ok(forwarded_str) = forwarded.to_str()
        && let Some(first_ip) = forwarded_str.split(',').next()
        && let Ok(ip) = first_ip.trim().parse()
    {
        return ip;
    }

    // Fallback to peer address from connection
    peer_addr.map_or_else(
        || {
            tracing::warn!("No peer address available, defaulting to localhost");
            "127.0.0.1".parse().expect("hardcoded IP should parse")
        },
        |addr| addr.ip(),
    )
}

type BoxedByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, std::io::Error>> + Send>>;

fn create_random_bytes_stream(config: Arc<TarpitConfig>) -> BoxedByteStream {
    Box::pin(stream::unfold((), move |()| {
        let config = Arc::clone(&config);
        async move {
            let (delay_ms, chunk) = {
                let mut rng = rand::rng();
                let delay_ms = rng.random_range(config.delay_min_ms..=config.delay_max_ms);
                let chunk_size = rng.random_range(config.chunk_size_min..=config.chunk_size_max);
                let chunk: Vec<u8> = (0..chunk_size).map(|_| rng.random()).collect();
                (delay_ms, chunk)
            };

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            Some((Ok(chunk), ()))
        }
    }))
}

fn create_fake_html_stream(config: Arc<TarpitConfig>) -> BoxedByteStream {
    Box::pin(stream::unfold(0, move |counter| {
        let config = Arc::clone(&config);
        async move {
            let (delay_ms, chunk) = {
                let mut rng = rand::rng();
                let delay_ms = rng.random_range(config.delay_min_ms..=config.delay_max_ms);

                let chunk = if counter == 0 {
                    concat!(
                        "<!DOCTYPE html>\n",
                        "<html>\n",
                        "<head>\n",
                        "  <title>Admin Panel</title>\n",
                        "  <meta charset=\"utf-8\">\n",
                        "</head>\n",
                        "<body>\n",
                        "  <h1>Loading...</h1>\n",
                        "  <div class=\"content\">\n"
                    )
                    .as_bytes()
                    .to_vec()
                } else {
                    let elements = [
                        "    <div class=\"item\">Processing request...</div>\n",
                        "    <span class=\"status\">Initializing...</span>\n",
                        "    <!-- Loading data -->\n",
                        "    <p>Fetching records...</p>\n",
                        "    <div class=\"loader\"></div>\n",
                        "    <script>console.log('Loading...');</script>\n",
                    ];
                    let element = elements[rng.random_range(0..elements.len())];
                    element.as_bytes().to_vec()
                };
                (delay_ms, chunk)
            };

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            Some((Ok(chunk), counter + 1))
        }
    }))
}

fn create_fake_json_stream(config: Arc<TarpitConfig>) -> BoxedByteStream {
    Box::pin(stream::unfold(0, move |counter| {
        let config = Arc::clone(&config);
        async move {
            let (delay_ms, chunk) = {
                let mut rng = rand::rng();
                let delay_ms = rng.random_range(config.delay_min_ms..=config.delay_max_ms);

                let chunk = if counter == 0 {
                    b"{\"status\":\"success\",\"data\":[\n".to_vec()
                } else {
                    let id = counter;
                    let username = format!("user{}", rng.random_range(1000..9999));
                    let email = format!("{username}@example.com");
                    let json = format!(
                        "{{\"id\":{id},\"username\":\"{username}\",\"email\":\"{email}\",\"active\":true}},\n"
                    );
                    json.as_bytes().to_vec()
                };
                (delay_ms, chunk)
            };

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            Some((Ok(chunk), counter + 1))
        }
    }))
}
pub async fn tarpit_handler(
    State(state): State<Arc<TarpitState>>,
    peer: Option<ConnectInfo<SocketAddr>>,
    req: Request,
) -> Response {
    let path = req.uri().path().to_string();
    let headers = req.headers();

    let client_ip = extract_client_ip(headers, peer.map(|ConnectInfo(addr)| addr));

    // Try to acquire global semaphore
    let _global_permit = if let Ok(Ok(permit)) = tokio::time::timeout(
        Duration::from_millis(100),
        state.global_semaphore.clone().acquire_owned(),
    )
    .await
    {
        permit
    } else {
        tracing::debug!(
            client_ip = %client_ip,
            reason = "global_limit",
            "Tarpit connection rejected"
        );
        return (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response();
    };

    // Get or create per-IP semaphore
    let ip_semaphore = state
        .ip_connections
        .entry(client_ip)
        .or_insert_with(|| Arc::new(Semaphore::new(state.config.max_connections_per_ip)))
        .clone();

    // Try to acquire per-IP semaphore
    let _ip_permit = if let Ok(Ok(permit)) = tokio::time::timeout(
        Duration::from_millis(100),
        ip_semaphore.clone().acquire_owned(),
    )
    .await
    {
        permit
    } else {
        tracing::debug!(
            client_ip = %client_ip,
            reason = "ip_limit",
            "Tarpit connection rejected"
        );
        return (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response();
    };

    let mode = ResponseMode::random();
    let start = Instant::now();

    tracing::debug!(
        path = %path,
        client_ip = %client_ip,
        mode = ?mode,
        global_available = state.global_semaphore.available_permits(),
        ip_available = ip_semaphore.available_permits(),
        "Tarpit triggered"
    );

    let stream: BoxedByteStream = match mode {
        ResponseMode::RandomBytes => create_random_bytes_stream(Arc::clone(&state.config)),
        ResponseMode::FakeHtml => create_fake_html_stream(Arc::clone(&state.config)),
        ResponseMode::FakeJson => create_fake_json_stream(Arc::clone(&state.config)),
    };

    // Wrap stream to log on drop and hold permits
    let stream_with_logging = stream::unfold(
        (
            stream,
            start,
            client_ip,
            0usize,
            false,
            _global_permit,
            _ip_permit,
        ),
        |(mut stream, start, client_ip, bytes_sent, logged, global_permit, ip_permit)| async move {
            use futures::StreamExt;

            match stream.next().await {
                Some(Ok(chunk)) => {
                    let new_bytes = bytes_sent + chunk.len();
                    Some((
                        Ok(chunk),
                        (
                            stream,
                            start,
                            client_ip,
                            new_bytes,
                            logged,
                            global_permit,
                            ip_permit,
                        ),
                    ))
                }
                Some(Err(e)) => Some((
                    Err(e),
                    (
                        stream,
                        start,
                        client_ip,
                        bytes_sent,
                        logged,
                        global_permit,
                        ip_permit,
                    ),
                )),
                None => {
                    if !logged {
                        let duration = start.elapsed();
                        tracing::debug!(
                            client_ip = %client_ip,
                            duration_secs = duration.as_secs(),
                            bytes_sent,
                            "Tarpit connection closed"
                        );
                    }
                    None
                }
            }
        },
    );

    let body = Body::from_stream(stream_with_logging);

    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        mode.content_type()
            .parse()
            .expect("content type should be valid header value"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_files() {
        assert!(is_malicious_path("/admin.php"));
        assert!(is_malicious_path("/wp-login.php"));
        assert!(is_malicious_path("/index.php"));
        assert!(is_malicious_path("/INFO.PHP"));
    }

    #[test]
    fn test_wordpress_paths() {
        assert!(is_malicious_path("/wp-admin/"));
        assert!(is_malicious_path("/wp-content/plugins/"));
        assert!(is_malicious_path("/wp-includes/"));
        assert!(is_malicious_path("/wordpress/index.php"));
    }

    #[test]
    fn test_admin_panels() {
        assert!(!is_malicious_path("/admin"));
        assert!(is_malicious_path("/administrator"));
        assert!(is_malicious_path("/phpmyadmin"));
        assert!(is_malicious_path("/phpMyAdmin"));
    }

    #[test]
    fn test_config_files() {
        assert!(is_malicious_path("/.env"));
        assert!(is_malicious_path("/.git/config"));
        assert!(is_malicious_path("/config.php"));
        assert!(is_malicious_path("/.aws/credentials"));
    }

    #[test]
    fn test_actuator_endpoints() {
        assert!(is_malicious_path("/actuator"));
        assert!(is_malicious_path("/actuator/health"));
    }

    #[test]
    fn test_api_docs() {
        assert!(is_malicious_path("/swagger.json"));
        assert!(is_malicious_path("/graphql"));
        assert!(is_malicious_path("/api-docs"));
    }

    #[test]
    fn test_legitimate_paths() {
        assert!(!is_malicious_path("/"));
        assert!(!is_malicious_path("/about"));
        assert!(!is_malicious_path("/api/projects"));
        assert!(!is_malicious_path("/favicon.ico"));
        assert!(!is_malicious_path("/robots.txt"));
        assert!(!is_malicious_path("/sitemap.xml"));
        assert!(!is_malicious_path("/keybase.txt"));
        assert!(!is_malicious_path("/_app/some-asset.js"));
    }

    #[test]
    fn test_ip_extraction() {
        use std::net::SocketAddr;
        let mut headers = HeaderMap::new();
        let peer: SocketAddr = "192.0.2.50:12345".parse().unwrap();

        // Test X-Real-IP
        headers.insert("x-real-ip", "203.0.113.42".parse().unwrap());
        let ip = extract_client_ip(&headers, Some(peer));
        assert_eq!(ip, "203.0.113.42".parse::<IpAddr>().unwrap());

        // Test X-Forwarded-For
        headers.clear();
        headers.insert(
            "x-forwarded-for",
            "198.51.100.1, 192.0.2.1".parse().unwrap(),
        );
        let ip = extract_client_ip(&headers, Some(peer));
        assert_eq!(ip, "198.51.100.1".parse::<IpAddr>().unwrap());

        // Test X-Real-IP takes precedence
        headers.insert("x-real-ip", "203.0.113.100".parse().unwrap());
        let ip = extract_client_ip(&headers, Some(peer));
        assert_eq!(ip, "203.0.113.100".parse::<IpAddr>().unwrap());

        // Test fallback to peer address
        headers.clear();
        let ip = extract_client_ip(&headers, Some(peer));
        assert_eq!(ip, "192.0.2.50".parse::<IpAddr>().unwrap());

        // Test fallback to localhost when no peer
        let ip = extract_client_ip(&headers, None);
        assert_eq!(ip, "127.0.0.1".parse::<IpAddr>().unwrap());
    }
}
