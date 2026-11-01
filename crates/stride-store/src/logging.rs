//! The store's log.
//!
//! There is no logging crate in the approved dependency set, so the store
//! carries its own: one sink, one string, one call.
//!
//! It exists for a reason that is not convenience. **The key must never be
//! printed**, and a rule that is only stated is a rule nobody checks. Every
//! diagnostic the store emits goes through [`log`], so a test can install a
//! sink, do everything the store does, and read back every line that was
//! produced. That test is in `tests/key_handling.rs`.
//!
//! The default sink writes to stderr. A library that logs to stdout would
//! corrupt the output of whatever is using it.

use std::sync::{Arc, Mutex};

/// A sink: something that takes a line and does something with it.
pub type Sink = Arc<dyn Fn(&str) + Send + Sync + 'static>;

/// The installed sink, if any.
static SINK: Mutex<Option<Sink>> = Mutex::new(None);

/// Install a sink. The lines go there instead of to stderr.
///
/// This is for tests and for hosts that want the store's diagnostics in their
/// own log. There is one sink, not a stack: a second call replaces the first.
pub fn set_sink(sink: Sink) {
    if let Ok(mut guard) = SINK.lock() {
        *guard = Some(sink);
    }
}

/// Remove the sink. The lines go to stderr again.
pub fn clear_sink() {
    if let Ok(mut guard) = SINK.lock() {
        *guard = None;
    }
}

/// Write one line.
///
/// The line is passed on as it is. Nothing here knows about keys, and
/// nothing that knows about keys calls this: a key is never turned into a
/// string in the first place (see [`crate::key`]).
pub fn log(message: &str) {
    // The sink is taken out of the lock before it is called: a sink that
    // logs would otherwise deadlock on itself.
    let sink = match SINK.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => None,
    };
    match sink {
        Some(sink) => sink(message),
        None => eprintln!("stride-store: {message}"),
    }
}
