//! SSRF protection for outbound downloads (GIF paste).
//! HTTPS-only, host allowlist, DNS resolution checks, no redirects.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use url::Url;

const ALLOWED_HOST_SUFFIXES: &[&str] = &["tenor.com", "giphy.com", "media.tenor.co"];

/// Parse, allowlist and resolve `url`. Rejects private/loopback/link-local/metadata IPs.
pub fn validate_public_https_url(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("Only HTTPS URLs are allowed".into());
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err("URLs with credentials are not allowed".into());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL is missing a host".to_string())?
        .to_ascii_lowercase();

    if host.parse::<IpAddr>().is_ok() {
        return Err("Direct IP downloads are not allowed".into());
    }
    if !is_allowed_host(&host) {
        return Err(format!("Host '{host}' is not on the download allowlist"));
    }

    let port = parsed.port().unwrap_or(443);
    let addrs = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for {host}: {e}"))?;

    let mut resolved = false;
    for addr in addrs {
        resolved = true;
        if is_disallowed_ip(addr.ip()) {
            return Err(format!(
                "Refusing download: {host} resolved to non-public IP {}",
                addr.ip()
            ));
        }
    }
    if !resolved {
        return Err(format!("No DNS records for {host}"));
    }

    Ok(parsed)
}

fn is_allowed_host(host: &str) -> bool {
    ALLOWED_HOST_SUFFIXES.iter().any(|suffix| {
        host == *suffix
            || host.ends_with(&format!(".{suffix}"))
    })
}

pub fn is_disallowed_ip(ip: IpAddr) -> bool {
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
        || (o[0] == 100 && o[1] & 0b1100_0000 == 0b0100_0000) // 100.64.0.0/10 CGNAT
        || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24 IETF
        || (o[0] == 192 && o[1] == 0 && o[2] == 2) // TEST-NET-1
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19)) // benchmark
        || (o[0] == 198 && o[1] == 51 && o[2] == 100)
        || (o[0] == 203 && o[1] == 0 && o[2] == 113)
}

fn is_disallowed_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (s[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        || (s[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || s[0] == 0x2001 && s[1] == 0x0db8 // documentation
}

/// reqwest redirect policy: never follow. Callers must re-validate any new URL.
pub fn no_redirects() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_https_and_ips() {
        assert!(validate_public_https_url("http://media.tenor.com/x.gif").is_err());
        assert!(validate_public_https_url("https://127.0.0.1/x.gif").is_err());
        assert!(validate_public_https_url("https://169.254.169.254/latest").is_err());
        assert!(validate_public_https_url("https://evil.example/x.gif").is_err());
        assert!(validate_public_https_url("https://localhost/x.gif").is_err());
    }

    #[test]
    fn allowlist_recognises_tenor() {
        assert!(is_allowed_host("media.tenor.com"));
        assert!(is_allowed_host("media1.tenor.com"));
        assert!(is_allowed_host("c.tenor.com"));
        assert!(!is_allowed_host("tenor.com.evil.test"));
        assert!(!is_allowed_host("example.com"));
    }

    #[test]
    fn private_ips_are_blocked() {
        assert!(is_disallowed_ip("10.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("192.168.1.1".parse().unwrap()));
        assert!(is_disallowed_ip("127.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("169.254.169.254".parse().unwrap()));
        assert!(is_disallowed_ip("100.64.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("::1".parse().unwrap()));
        assert!(is_disallowed_ip("fc00::1".parse().unwrap()));
        assert!(is_disallowed_ip("fe80::1".parse().unwrap()));
        assert!(!is_disallowed_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_disallowed_ip("1.1.1.1".parse().unwrap()));
    }
}
