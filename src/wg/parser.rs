//! Parser for WireGuard `.conf` files.
//!
//! WireGuard uses an INI-like format with one `[Interface]` section
//! and zero or more `[Peer]` sections. We parse line-by-line because
//! generic INI crates collapse duplicate `[Peer]` sections.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing [Interface] section")]
    MissingInterface,
    #[error("key=value line outside any section: {0:?}")]
    OrphanLine(String),
    #[error("unknown section: [{0}]")]
    UnknownSection(String),
    #[error("malformed line (no '='): {0:?}")]
    Malformed(String),
    #[error("missing required key: {0}")]
    MissingKey(&'static str),
    #[error("invalid value for {key}: {value:?}")]
    InvalidValue { key: &'static str, value: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interface {
    pub private_key: String,
    pub address: Vec<String>,
    pub dns: Vec<String>,
    pub listen_port: Option<u16>,
    pub mtu: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Peer {
    pub public_key: String,
    pub preshared_key: Option<String>,
    pub allowed_ips: Vec<String>,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedConfig {
    pub interface: Interface,
    pub peers: Vec<Peer>,
}

#[derive(Debug)]
enum Section {
    Interface(Interface),
    Peer(Peer),
}

pub fn parse(text: &str) -> Result<ParsedConfig, ParseError> {
    let mut interface: Option<Interface> = None;
    let mut peers: Vec<Peer> = Vec::new();
    let mut current: Option<Section> = None;

    let commit = |current: Option<Section>,
                  interface: &mut Option<Interface>,
                  peers: &mut Vec<Peer>| match current {
        Some(Section::Interface(i)) => *interface = Some(i),
        Some(Section::Peer(p)) => peers.push(p),
        None => {}
    };

    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            commit(current.take(), &mut interface, &mut peers);
            current = Some(match name.trim().to_ascii_lowercase().as_str() {
                "interface" => Section::Interface(Interface::default()),
                "peer" => Section::Peer(Peer::default()),
                other => return Err(ParseError::UnknownSection(other.to_string())),
            });
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| ParseError::Malformed(line.to_string()))?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().to_string();

        match current.as_mut() {
            None => return Err(ParseError::OrphanLine(line.to_string())),
            Some(Section::Interface(i)) => apply_interface(i, &key, value)?,
            Some(Section::Peer(p)) => apply_peer(p, &key, value)?,
        }
    }
    commit(current, &mut interface, &mut peers);

    let interface = interface.ok_or(ParseError::MissingInterface)?;
    if interface.private_key.is_empty() {
        return Err(ParseError::MissingKey("PrivateKey"));
    }
    for peer in &peers {
        if peer.public_key.is_empty() {
            return Err(ParseError::MissingKey("PublicKey"));
        }
    }
    Ok(ParsedConfig { interface, peers })
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(head, _)| head)
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn apply_interface(i: &mut Interface, key: &str, value: String) -> Result<(), ParseError> {
    match key {
        "privatekey" => i.private_key = value,
        "address" => i.address = split_csv(&value),
        "dns" => i.dns = split_csv(&value),
        "listenport" => {
            i.listen_port = Some(value.parse().map_err(|_| ParseError::InvalidValue {
                key: "ListenPort",
                value,
            })?);
        }
        "mtu" => {
            i.mtu = Some(
                value
                    .parse()
                    .map_err(|_| ParseError::InvalidValue { key: "MTU", value })?,
            );
        }
        // Recognised but ignored keys (advanced features handled by wg-quick itself).
        "table" | "preup" | "postup" | "predown" | "postdown" | "saveconfig" | "fwmark" => {}
        _ => {} // Ignore unknown keys to be forgiving.
    }
    Ok(())
}

fn apply_peer(p: &mut Peer, key: &str, value: String) -> Result<(), ParseError> {
    match key {
        "publickey" => p.public_key = value,
        "presharedkey" => p.preshared_key = Some(value),
        "allowedips" => p.allowed_ips = split_csv(&value),
        "endpoint" => p.endpoint = Some(value),
        "persistentkeepalive" => {
            p.persistent_keepalive = Some(value.parse().map_err(|_| ParseError::InvalidValue {
                key: "PersistentKeepalive",
                value,
            })?);
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# A sample WireGuard config
[Interface]
PrivateKey = aGVsbG93b3JsZGhlbGxvd29ybGRoZWxsb3dvcmxkaGVsbA=
Address = 10.0.0.2/32, fd00::2/128
DNS = 1.1.1.1, 8.8.8.8
ListenPort = 51820
MTU = 1420

[Peer]
PublicKey = cHViMQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
AllowedIPs = 0.0.0.0/0, ::/0
Endpoint = vpn.example.com:51820
PersistentKeepalive = 25

[Peer]
# Second peer
PublicKey = cHViMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
AllowedIPs = 192.168.1.0/24
";

    #[test]
    fn parses_full_sample() {
        let cfg = parse(SAMPLE).unwrap();
        assert_eq!(cfg.interface.address, ["10.0.0.2/32", "fd00::2/128"]);
        assert_eq!(cfg.interface.dns, ["1.1.1.1", "8.8.8.8"]);
        assert_eq!(cfg.interface.listen_port, Some(51820));
        assert_eq!(cfg.interface.mtu, Some(1420));
        assert_eq!(cfg.peers.len(), 2);
        assert_eq!(
            cfg.peers[0].endpoint.as_deref(),
            Some("vpn.example.com:51820")
        );
        assert_eq!(cfg.peers[0].persistent_keepalive, Some(25));
        assert_eq!(cfg.peers[1].allowed_ips, ["192.168.1.0/24"]);
    }

    #[test]
    fn requires_interface() {
        assert!(matches!(parse(""), Err(ParseError::MissingInterface)));
        // A bare [Peer] without any [Interface] section is the
        // same error from the user's perspective.
        assert!(matches!(
            parse("[Peer]\nPublicKey = foo"),
            Err(ParseError::MissingInterface)
        ));
    }

    #[test]
    fn requires_private_key() {
        let cfg = "[Interface]\nAddress = 10.0.0.1/32";
        assert!(matches!(
            parse(cfg),
            Err(ParseError::MissingKey("PrivateKey"))
        ));
    }

    #[test]
    fn rejects_unknown_section() {
        let cfg = "[Garbage]\nFoo = bar";
        assert!(matches!(parse(cfg), Err(ParseError::UnknownSection(_))));
    }

    #[test]
    fn keys_are_case_insensitive() {
        let cfg = "[interface]\nPRIVATEKEY = abc\nadress = 10.0.0.1/32";
        let parsed = parse(cfg).unwrap();
        assert_eq!(parsed.interface.private_key, "abc");
    }

    #[test]
    fn ignores_inline_comments() {
        let cfg = "[Interface]\nPrivateKey = abc # with comment\nAddress = 10/8";
        let parsed = parse(cfg).unwrap();
        assert_eq!(parsed.interface.private_key, "abc");
    }
}
