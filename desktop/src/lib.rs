//! The desktop rig.
//!
//! **One binary** that serves the app and the router, on this machine, and
//! nothing else. It is not a product: it is the development rig (a fast
//! feedback loop), the community tool (a stranger clones the repository and
//! runs it), and the reference host (the process the parity differ and the
//! harness talk to).
//!
//! ## What it is made of
//!
//! | module | what it is |
//! |---|---|
//! | [`main`] | the command line: `serve`, `migrate`, `doctor` |
//! | [`server`] | the HTTP surface: the trust gate, the router, the app, TLS |
//! | [`http`] | HTTP/1.1, written out |
//! | [`trust`] | the trust model: loopback, a trusted network, or a refusal |
//! | [`key`] | the Garmin key: in memory, in the environment, or in a file |
//! | [`dbkey`] | the database key: generated, stored with `0600`, and the plaintext conversion |
//! | [`spa`] | the embedded app, and the guard that keeps it inside |
//! | [`api`] | the seam to the router, and what the rig answers with until it lands |
//! | [`paths`] | where the rig keeps its files |
//! | [`log`] | the rig's log, and the sink a test reads |
//!
//! ## The rules it keeps
//!
//! * **The trust model is three lines.** Loopback is always allowed; a
//!   network in `STRIDE_TRUSTED_NETWORKS` is allowed; everything else is
//!   refused with a message that names the variable. There are no pairing
//!   codes, no QR codes and no device tokens, and there never will be.
//! * **The Garmin key never goes to disk in the clear** unless the athlete
//!   puts it there themselves, in a file with `0600`. It is never logged,
//!   never returned by an endpoint, and never written into the database.
//! * **The database is encrypted**, with a key of its own, and the key that
//!   opens it is not the Garmin key.
//! * **The desktop is a host, not a brain.** Every number comes from
//!   `stride-core`, every reading from `stride-store`, and every request
//!   goes through `stride-api`'s router.

#![deny(missing_docs)]

pub mod api;
pub mod dbkey;
pub mod error;
pub mod http;
pub mod key;
pub mod log;
pub mod paths;
pub mod server;
pub mod spa;
pub mod trust;
