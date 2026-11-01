//! The trust model.
//!
//! **Three lines, and no more.**
//!
//! 1. **Loopback** — `127.0.0.1` or `::1` — is always allowed. No
//!    configuration, no ceremony: the request came from this machine, and
//!    the athlete is the one using the machine.
//! 2. **A trusted network** — a CIDR in `STRIDE_TRUSTED_NETWORKS` — is
//!    allowed, because the athlete may want to open the rig on their phone
//!    over their own tailnet.
//! 3. **Everything else** is refused, with a message that names the
//!    variable.
//!
//! ## What is not here, on purpose
//!
//! There are no pairing codes, no QR codes, no device tokens and no
//! capability cookies. A rig that is open to a network is a rig whose
//! Garmin key is on that network, and a pairing dance would be a second,
//! slower way of saying "trust this network" — one that leaves a token
//! lying around for somebody else to find.
//!
//! The trust model is the whole of it. There is nothing to configure, no
//! code to type into a phone, and nothing to revoke but the environment
//! variable itself.
//!
//! ## The variable
//!
//! `STRIDE_TRUSTED_NETWORKS` is a list of CIDR blocks, separated by commas
//! or by whitespace:
//!
//! ```text
//! STRIDE_TRUSTED_NETWORKS=192.168.1.0/24,fd00::/8
//! ```
//!
//! A plain address is a network of one: `192.168.1.5` means
//! `192.168.1.5/32`. A list that cannot be read is refused at start-up,
//! with the text that could not be read — a rig that silently trusts
//! nothing is a rig that looks broken.

use std::fmt;
use std::net::IpAddr;

use crate::error::{Error, Result};

/// The environment variable that names the trusted networks.
pub const TRUSTED_NETWORKS_VARIABLE: &str = "STRIDE_TRUSTED_NETWORKS";

/// One network the rig answers on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Network {
    /// The address the network starts at.
    address: IpAddr,
    /// How many bits of it are the network.
    prefix: u8,
}

impl Network {
    /// Read one network: `192.168.1.0/24`, or `192.168.1.5` for one address.
    pub fn parse(text: &str) -> Result<Network> {
        let text = text.trim();
        if text.is_empty() {
            return Err(Error::config(format!(
                "{TRUSTED_NETWORKS_VARIABLE} holds an empty network"
            )));
        }
        let (address, prefix) = match text.split_once('/') {
            Some((address, prefix)) => {
                let address: IpAddr = address.trim().parse().map_err(|_| {
                    Error::config(format!(
                        "{TRUSTED_NETWORKS_VARIABLE} holds {text:?}, and {address:?} is not an address"
                    ))
                })?;
                let prefix: u8 = prefix.trim().parse().map_err(|_| {
                    Error::config(format!(
                        "{TRUSTED_NETWORKS_VARIABLE} holds {text:?}, and {prefix:?} is not a prefix length"
                    ))
                })?;
                let longest = if address.is_ipv4() { 32 } else { 128 };
                if prefix > longest {
                    return Err(Error::config(format!(
                        "{TRUSTED_NETWORKS_VARIABLE} holds {text:?}, and a /{prefix} is longer than \
                         a /{longest}"
                    )));
                }
                (address, prefix)
            }
            None => {
                let address: IpAddr = text.parse().map_err(|_| {
                    Error::config(format!(
                        "{TRUSTED_NETWORKS_VARIABLE} holds {text:?}, and it is not an address"
                    ))
                })?;
                let prefix = if address.is_ipv4() { 32 } else { 128 };
                (address, prefix)
            }
        };
        Ok(Network { address, prefix })
    }

    /// The address the network starts at.
    pub fn address(&self) -> IpAddr {
        self.address
    }

    /// How many bits of the address are the network.
    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    /// Is this address inside the network?
    pub fn contains(&self, peer: &IpAddr) -> bool {
        match (self.address, peer) {
            (IpAddr::V4(network), IpAddr::V4(peer)) => {
                prefix_match(&network.octets(), &peer.octets(), self.prefix)
            }
            (IpAddr::V6(network), IpAddr::V6(peer)) => {
                prefix_match(&network.octets(), &peer.octets(), self.prefix)
            }
            _ => false,
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix)
    }
}

/// Do the first `prefix` bits of two addresses agree?
fn prefix_match(network: &[u8], peer: &[u8], prefix: u8) -> bool {
    let whole = (prefix / 8) as usize;
    let rest = prefix % 8;
    if network.len() != peer.len() || whole > network.len() {
        return false;
    }
    if network[..whole] != peer[..whole] {
        return false;
    }
    if rest == 0 {
        return true;
    }
    let mask = 0xffu8 << (8 - rest);
    network[whole] & mask == peer[whole] & mask
}

/// What the trust model says about a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The request is answered.
    Allow,
    /// The request is refused, and this is why.
    Refuse(Refusal),
}

/// A refusal, and the sentence that goes with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// Where the request came from.
    peer: IpAddr,
}

impl Refusal {
    /// The address the request came from.
    pub fn peer(&self) -> IpAddr {
        self.peer
    }

    /// The sentence the rig answers with.
    ///
    /// It names the variable, because a refusal that does not say what to do
    /// is a wall, and this rig is meant to be opened by the person who owns
    /// it.
    pub fn message(&self) -> String {
        format!(
            "This request came from {}, which is not this machine's own loopback and is not in a \
             trusted network.\n\nTo let it in, start the rig with {TRUSTED_NETWORKS_VARIABLE} set to \
             the network it is on — for example {TRUSTED_NETWORKS_VARIABLE}=192.168.1.0/24 — and \
             try again.",
            self.peer
        )
    }
}

/// The trust model.
#[derive(Debug, Clone, Default)]
pub struct Trust {
    networks: Vec<Network>,
}

impl Trust {
    /// The trust model with nothing trusted beyond loopback.
    pub fn loopback_only() -> Trust {
        Trust {
            networks: Vec::new(),
        }
    }

    /// The trust model the environment asks for.
    ///
    /// `STRIDE_TRUSTED_NETWORKS` unset or empty means loopback only.
    pub fn from_environment() -> Result<Trust> {
        match std::env::var(TRUSTED_NETWORKS_VARIABLE) {
            Ok(value) => Trust::from_list(&value),
            Err(_) => Ok(Trust::loopback_only()),
        }
    }

    /// The trust model a list asks for.
    ///
    /// The list is separated by commas or by whitespace. An empty list is
    /// loopback only.
    pub fn from_list(list: &str) -> Result<Trust> {
        let mut networks = Vec::new();
        for piece in list.split([',', ' ', '\t', '\n', '\r', ';']) {
            let piece = piece.trim();
            if piece.is_empty() {
                continue;
            }
            networks.push(Network::parse(piece)?);
        }
        Ok(Trust { networks })
    }

    /// The networks that are trusted.
    pub fn networks(&self) -> &[Network] {
        &self.networks
    }

    /// What the model says about a request from this address.
    pub fn decide(&self, peer: IpAddr) -> Decision {
        let peer = normalise(peer);
        if peer.is_loopback() {
            return Decision::Allow;
        }
        if self.networks.iter().any(|network| network.contains(&peer)) {
            return Decision::Allow;
        }
        Decision::Refuse(Refusal { peer })
    }
}

/// The address a request really came from.
///
/// A peer that arrives as `::ffff:127.0.0.1` is on the loopback, whatever
/// the socket says about it.
fn normalise(peer: IpAddr) -> IpAddr {
    match peer {
        IpAddr::V6(peer) => match peer.to_ipv4_mapped() {
            Some(mapped) => IpAddr::V4(mapped),
            None => IpAddr::V6(peer),
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An address, for the tests.
    fn ip(text: &str) -> IpAddr {
        text.parse()
            .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
    }

    #[test]
    fn a_network_is_read_from_a_cidr() -> Result<()> {
        let network = Network::parse("192.168.1.0/24")?;
        assert_eq!(network.prefix(), 24);
        assert!(network.contains(&ip("192.168.1.7")));
        assert!(!network.contains(&ip("192.168.2.7")));
        Ok(())
    }

    #[test]
    fn a_plain_address_is_a_network_of_one() -> Result<()> {
        let network = Network::parse("10.0.0.9")?;
        assert_eq!(network.prefix(), 32);
        assert!(network.contains(&ip("10.0.0.9")));
        assert!(!network.contains(&ip("10.0.0.10")));
        Ok(())
    }

    #[test]
    fn loopback_is_always_allowed() -> Result<()> {
        let trust = Trust::loopback_only();
        assert_eq!(trust.decide(ip("127.0.0.1")), Decision::Allow);
        assert_eq!(trust.decide(ip("::1")), Decision::Allow);
        Ok(())
    }
}
