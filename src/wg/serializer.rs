//! Serialise a [`ParsedConfig`] back into WireGuard's INI-like
//! `.conf` format. Intentionally minimal: we only emit keys that
//! are actually set, keeping output stable round-trippable
//! against [`super::parser::parse`].

use std::fmt::Write;

use super::parser::ParsedConfig;

pub fn serialize(config: &ParsedConfig) -> String {
    let mut out = String::new();
    out.push_str("[Interface]\n");

    let iface = &config.interface;
    if !iface.private_key.is_empty() {
        let _ = writeln!(out, "PrivateKey = {}", iface.private_key);
    }
    if !iface.address.is_empty() {
        let _ = writeln!(out, "Address = {}", iface.address.join(", "));
    }
    if !iface.dns.is_empty() {
        let _ = writeln!(out, "DNS = {}", iface.dns.join(", "));
    }
    if let Some(port) = iface.listen_port {
        let _ = writeln!(out, "ListenPort = {port}");
    }
    if let Some(mtu) = iface.mtu {
        let _ = writeln!(out, "MTU = {mtu}");
    }

    for peer in &config.peers {
        out.push_str("\n[Peer]\n");
        if !peer.public_key.is_empty() {
            let _ = writeln!(out, "PublicKey = {}", peer.public_key);
        }
        if let Some(psk) = &peer.preshared_key {
            if !psk.is_empty() {
                let _ = writeln!(out, "PresharedKey = {psk}");
            }
        }
        if !peer.allowed_ips.is_empty() {
            let _ = writeln!(out, "AllowedIPs = {}", peer.allowed_ips.join(", "));
        }
        if let Some(endpoint) = &peer.endpoint {
            if !endpoint.is_empty() {
                let _ = writeln!(out, "Endpoint = {endpoint}");
            }
        }
        if let Some(ka) = peer.persistent_keepalive {
            let _ = writeln!(out, "PersistentKeepalive = {ka}");
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wg::parser::parse;

    #[test]
    fn round_trip() {
        let original = "\
[Interface]
PrivateKey = aGVsbG93b3JsZGhlbGxvd29ybGRoZWxsb3dvcmxkaGVsbA=
Address = 10.0.0.2/32, fd00::2/128
DNS = 1.1.1.1, 8.8.8.8
ListenPort = 51820
MTU = 1420

[Peer]
PublicKey = cHViMQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
AllowedIPs = 0.0.0.0/0
Endpoint = vpn.example.com:51820
PersistentKeepalive = 25

[Peer]
PublicKey = cHViMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
AllowedIPs = 192.168.1.0/24
";
        let parsed = parse(original).unwrap();
        let reserialised = serialize(&parsed);
        // The reserialised form parses to the same ParsedConfig.
        let reparsed = parse(&reserialised).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn omits_unset_optionals() {
        let cfg = "[Interface]\nPrivateKey = abc\n";
        let parsed = parse(cfg).unwrap();
        let s = serialize(&parsed);
        assert!(!s.contains("ListenPort"));
        assert!(!s.contains("MTU"));
        assert!(!s.contains("DNS"));
        assert!(!s.contains("Address"));
    }
}
