//! Safe URL opener used by Smart Actions.
//! Validation lives in Rust so the webview cannot bypass the TypeScript sanitizer.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::{Command, Stdio};
use url::Url;

const MAX_URL_LEN: usize = 2048;

/// Open `raw` with `xdg-open` after an allowlist check. Never passes the URL
/// through a shell.
pub fn open_safe_url(raw: &str) -> Result<String, String> {
    let safe = validate_open_url(raw)?;
    Command::new("xdg-open")
        .arg(&safe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch xdg-open: {e}"))?;
    Ok(safe)
}

pub fn validate_open_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_URL_LEN {
        return Err("URL is empty or too long".into());
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("URL contains control characters".into());
    }

    let parsed = Url::parse(trimmed).map_err(|e| format!("Invalid URL: {e}"))?;
    let scheme = parsed.scheme();
    if scheme != "https" && scheme != "http" && scheme != "mailto" {
        return Err(format!("Protocol '{scheme}' is not allowed"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs with credentials are not allowed".into());
    }
    if scheme == "mailto" {
        return Ok(parsed.to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL is missing a host".to_string())?
        .trim_matches(|c| c == '[' || c == ']')
        .to_ascii_lowercase();

    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".internal") {
        return Err("Local/internal hosts are not allowed".into());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_disallowed_ip(ip) {
            return Err("Private or reserved IP addresses are not allowed".into());
        }
    } else if looks_like_dotted_ipv4(&host) {
        return Err("Private or reserved IP addresses are not allowed".into());
    }

    Ok(parsed.to_string())
}

fn looks_like_dotted_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

fn is_disallowed_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_disallowed_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_disallowed_v4(v4);
            }
            is_disallowed_v6(v6)
        }
    }
}

fn is_disallowed_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_multicast()
        || o[0] == 0
        || (o[0] == 100 && o[1] & 0b1100_0000 == 0b0100_0000)
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        || (o[0] == 192 && o[1] == 0 && o[2] == 2)
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
        || (o[0] == 198 && o[1] == 51 && o[2] == 100)
        || (o[0] == 203 && o[1] == 0 && o[2] == 113)
        || o[0] >= 224
}

fn is_disallowed_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (s[0] & 0xffc0) == 0xfe80
        || (s[0] & 0xfe00) == 0xfc00
        || (s[0] == 0x2001 && s[1] == 0x0db8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_public_https() {
        assert!(validate_open_url("https://example.com/docs").is_ok());
        assert!(validate_open_url("mailto:user@example.com").is_ok());
    }

    #[test]
    fn rejects_dangerous_targets() {
        assert!(validate_open_url("javascript:alert(1)").is_err());
        assert!(validate_open_url("http://127.0.0.1/").is_err());
        assert!(validate_open_url("https://localhost/admin").is_err());
        assert!(validate_open_url("https://169.254.169.254/latest").is_err());
        assert!(validate_open_url("https://user:pass@example.com/").is_err());
        assert!(validate_open_url("file:///etc/passwd").is_err());
    }
}
