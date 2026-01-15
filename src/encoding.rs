//! Content encoding negotiation and compression utilities
//!
//! Handles Accept-Encoding header parsing with quality values
//! and provides compression helpers for ISR cache.

use axum::http::{HeaderMap, HeaderValue, header};
use std::io::Write;

/// Minimum size threshold for compression (bytes)
///
/// NOTE: This value must match MIN_SIZE in web/scripts/compress-assets.ts
/// to ensure runtime and build-time compression use the same threshold.
pub const COMPRESSION_MIN_SIZE: usize = 512;

/// Supported encodings in priority order (best to worst)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentEncoding {
    Zstd,
    Brotli,
    Gzip,
    Identity,
}

impl ContentEncoding {
    /// File extension suffix for this encoding
    #[inline]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Zstd => ".zst",
            Self::Brotli => ".br",
            Self::Gzip => ".gz",
            Self::Identity => "",
        }
    }

    /// Content-Encoding header value
    #[inline]
    pub fn header_value(&self) -> Option<HeaderValue> {
        match self {
            Self::Zstd => Some(HeaderValue::from_static("zstd")),
            Self::Brotli => Some(HeaderValue::from_static("br")),
            Self::Gzip => Some(HeaderValue::from_static("gzip")),
            Self::Identity => None,
        }
    }

    /// Default priority (higher = better)
    #[inline]
    fn default_priority(&self) -> u8 {
        match self {
            Self::Zstd => 4,
            Self::Brotli => 3,
            Self::Gzip => 2,
            Self::Identity => 1,
        }
    }
}

/// Parse Accept-Encoding header and return all supported encodings
///
/// Returns encodings in priority order (best first) with quality > 0.
/// Supports quality values and wildcard (*).
#[inline]
pub fn parse_accepted_encodings(headers: &HeaderMap) -> Vec<ContentEncoding> {
    let Some(accept) = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
    else {
        return vec![ContentEncoding::Identity];
    };

    let mut encodings: Vec<(ContentEncoding, f32)> = Vec::new();

    for part in accept.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Parse quality value, handling additional params (e.g., "br;q=0.8;level=5")
        let (encoding_str, quality) = if let Some((enc, params)) = part.split_once(';') {
            let q = params
                .split(';')
                .find_map(|p| p.trim().strip_prefix("q="))
                .and_then(|q| q.parse::<f32>().ok())
                .unwrap_or(1.0);
            (enc.trim(), q)
        } else {
            (part, 1.0)
        };

        // Skip disabled encodings
        if quality == 0.0 {
            continue;
        }

        let encoding = match encoding_str.to_lowercase().as_str() {
            "zstd" => ContentEncoding::Zstd,
            "br" | "brotli" => ContentEncoding::Brotli,
            "gzip" | "x-gzip" => ContentEncoding::Gzip,
            "*" => ContentEncoding::Gzip, // Wildcard defaults to gzip
            "identity" => ContentEncoding::Identity,
            _ => continue,
        };

        encodings.push((encoding, quality));
    }

    // Sort by quality (desc), then by default priority (desc)
    encodings.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.0.default_priority().cmp(&a.0.default_priority()))
    });

    if encodings.is_empty() {
        vec![ContentEncoding::Identity]
    } else {
        encodings.into_iter().map(|(e, _)| e).collect()
    }
}

/// Parse Accept-Encoding header and return best supported encoding
///
/// Supports quality values: `Accept-Encoding: gzip;q=0.8, br;q=1.0, zstd`
/// Priority when equal quality: zstd > brotli > gzip > identity
#[inline]
pub fn negotiate_encoding(headers: &HeaderMap) -> ContentEncoding {
    parse_accepted_encodings(headers)
        .into_iter()
        .next()
        .unwrap_or(ContentEncoding::Identity)
}

/// Check if content type should be compressed
#[inline]
#[allow(dead_code)]
pub fn is_compressible_content_type(content_type: &str) -> bool {
    let ct = content_type.to_lowercase();

    // Text types
    if ct.starts_with("text/") {
        return true;
    }

    // JSON, XML, SVG
    if ct.contains("json") || ct.contains("xml") || ct.contains("svg") {
        return true;
    }

    // JavaScript
    if ct.contains("javascript") || ct.contains("ecmascript") {
        return true;
    }

    // Font formats (woff/woff2 are already compressed)
    if ct.contains("font") && !ct.contains("woff") {
        return true;
    }

    false
}

/// Compress data with zstd at fast level (level 3)
pub fn compress_zstd(data: &[u8]) -> Option<Vec<u8>> {
    match zstd::encode_all(std::io::Cursor::new(data), 3) {
        Ok(compressed) => Some(compressed),
        Err(e) => {
            tracing::warn!(error = %e, size = data.len(), "zstd compression failed");
            None
        }
    }
}

/// Compress data with brotli at fast level (level 4)
pub fn compress_brotli(data: &[u8]) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    let mut writer = brotli::CompressorWriter::new(&mut output, 4096, 4, 22);
    if let Err(e) = writer.write_all(data) {
        tracing::warn!(error = %e, size = data.len(), "brotli compression failed");
        return None;
    }
    drop(writer);
    Some(output)
}

/// Compress data with gzip at fast level (level 1)
pub fn compress_gzip(data: &[u8]) -> Option<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    if let Err(e) = encoder.write_all(data) {
        tracing::warn!(error = %e, size = data.len(), "gzip write failed");
        return None;
    }
    match encoder.finish() {
        Ok(compressed) => Some(compressed),
        Err(e) => {
            tracing::warn!(error = %e, size = data.len(), "gzip finish failed");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_accepted_encodings() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, "gzip, br, zstd".parse().unwrap());
        let encodings = parse_accepted_encodings(&headers);
        assert_eq!(encodings[0], ContentEncoding::Zstd);
        assert_eq!(encodings[1], ContentEncoding::Brotli);
        assert_eq!(encodings[2], ContentEncoding::Gzip);
    }

    #[test]
    fn test_parse_accepted_encodings_with_quality() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "gzip;q=1.0, br;q=0.5, zstd;q=0.8".parse().unwrap(),
        );
        let encodings = parse_accepted_encodings(&headers);
        assert_eq!(encodings[0], ContentEncoding::Gzip);
        assert_eq!(encodings[1], ContentEncoding::Zstd);
        assert_eq!(encodings[2], ContentEncoding::Brotli);
    }

    #[test]
    fn test_negotiate_simple() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, "gzip, br".parse().unwrap());
        assert_eq!(negotiate_encoding(&headers), ContentEncoding::Brotli);
    }

    #[test]
    fn test_negotiate_with_quality() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "gzip;q=1.0, br;q=0.5".parse().unwrap(),
        );
        assert_eq!(negotiate_encoding(&headers), ContentEncoding::Gzip);
    }

    #[test]
    fn test_negotiate_zstd_priority() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, "gzip, br, zstd".parse().unwrap());
        assert_eq!(negotiate_encoding(&headers), ContentEncoding::Zstd);
    }

    #[test]
    fn test_negotiate_no_header() {
        let headers = HeaderMap::new();
        assert_eq!(negotiate_encoding(&headers), ContentEncoding::Identity);
    }

    #[test]
    fn test_negotiate_disabled_encoding() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "zstd;q=0, br, gzip".parse().unwrap(),
        );
        // zstd is disabled (q=0), so should pick brotli
        assert_eq!(negotiate_encoding(&headers), ContentEncoding::Brotli);
    }

    #[test]
    fn test_negotiate_real_browser() {
        // Chrome's actual header
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );
        assert_eq!(negotiate_encoding(&headers), ContentEncoding::Zstd);
    }

    #[test]
    fn test_compressible_content_types() {
        assert!(is_compressible_content_type("text/html"));
        assert!(is_compressible_content_type("text/css"));
        assert!(is_compressible_content_type("application/json"));
        assert!(is_compressible_content_type("application/javascript"));
        assert!(is_compressible_content_type("image/svg+xml"));
        assert!(is_compressible_content_type("text/xml"));

        // Not compressible
        assert!(!is_compressible_content_type("image/png"));
        assert!(!is_compressible_content_type("image/jpeg"));
        assert!(!is_compressible_content_type("video/mp4"));
        assert!(!is_compressible_content_type("font/woff2"));
        assert!(!is_compressible_content_type("application/octet-stream"));
    }

    #[test]
    fn test_compression_functions() {
        let data = b"Hello, World! This is some test data that should be compressed.";

        // All compression functions should work
        let zstd = compress_zstd(data).unwrap();
        let brotli = compress_brotli(data).unwrap();
        let gzip = compress_gzip(data).unwrap();

        // Compressed should generally be smaller (for reasonable input)
        // Note: very small inputs might not compress well
        assert!(!zstd.is_empty());
        assert!(!brotli.is_empty());
        assert!(!gzip.is_empty());
    }

    #[test]
    fn test_extension() {
        assert_eq!(ContentEncoding::Zstd.extension(), ".zst");
        assert_eq!(ContentEncoding::Brotli.extension(), ".br");
        assert_eq!(ContentEncoding::Gzip.extension(), ".gz");
        assert_eq!(ContentEncoding::Identity.extension(), "");
    }

    #[test]
    fn test_header_value() {
        assert_eq!(
            ContentEncoding::Zstd.header_value().unwrap(),
            HeaderValue::from_static("zstd")
        );
        assert_eq!(
            ContentEncoding::Brotli.header_value().unwrap(),
            HeaderValue::from_static("br")
        );
        assert_eq!(
            ContentEncoding::Gzip.header_value().unwrap(),
            HeaderValue::from_static("gzip")
        );
        assert!(ContentEncoding::Identity.header_value().is_none());
    }
}
