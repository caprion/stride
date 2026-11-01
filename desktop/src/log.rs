//! The rig's log.
//!
//! One sink, one line, one call — the same shape as the store's log
//! (`stride_store::logging`), and for the same reason: **the Garmin key must
//! never be printed**, and a rule that is only stated is a rule nobody
//! checks. Every line the rig emits goes through [`log`], so a test can
//! install a sink, drive the whole rig, and read back every line that was
//! produced.
//!
//! The default sink writes to stderr. A rig that wrote its log to stdout
//! would corrupt the output of whatever is using it.

use std::sync::{Arc, Mutex};

/// A sink: something that takes a line and does something with it.
pub type Sink = Arc<dyn Fn(&str) + Send + Sync + 'static>;

/// The installed sink, if any.
static SINK: Mutex<Option<Sink>> = Mutex::new(None);

/// Install a sink. The lines go there instead of to stderr.
///
/// This is for tests, and for a host that wants the rig's diagnostics in its
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
/// nothing that knows about a key calls this: a key is never turned into a
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
        None => eprintln!("stride-desktop: {message}"),
    }
}
