//! The app, and the files it is made of.
//!
//! **One bundle, embedded in the binary.** A clean checkout carries the
//! released bundle in `desktop/assets`, so `cargo run -p stride-desktop`
//! needs no Node, no `node_modules`, and no second process to keep alive.
//! A developer checkout may have `frontend/dist`; that bundle is preferred
//! while it is there, which keeps the normal edit-reload loop working.
//!
//! ## The guard
//!
//! A request path is never used as a path into the machine. Two rules keep
//! that true, and both of them are checked before anything is looked up:
//!
//! * **the request path is checked** — a path that walks upwards
//!   (`..`), a path with a backslash in it, and a path with a null byte in
//!   it are refused outright;
//! * **the file that is served is checked** — when the rig serves a
//!   directory from disk, the file it resolved has to be *inside* that
//!   directory, after the path has been canonicalised. A symbolic link out
//!   of the directory is caught by the second check, because the path is
//!   compared after the link has been followed.
//!
//! The first rule is what `GET /../../etc/passwd` runs into.
//!
//! ## The fallback
//!
//! A path the bundle does not hold, and that does not look like a file, is
//! answered with the app's own `index.html` — because the app has its own
//! router, and it is the app's job to say what `/runs` means. The rig does
//! not have an opinion about the app's routes, and it never answers with a
//! stack trace.

use std::path::{Path, PathBuf};

use rust_embed::RustEmbed;

use crate::http::Response;

/// The development bundle, embedded in the binary.
///
/// The path is relative to this crate: `desktop/../frontend/dist` is the
/// bundle the frontend's build makes.
#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct DevelopmentAssets;

/// The released bundle, embedded in the binary.
///
/// This is what a clean Rust-only checkout carries. It is the same app,
/// built from the frontend at the commit that produced this bundle.
#[derive(RustEmbed)]
#[folder = "assets"]
struct ReleasedAssets;

/// Was the bundle there when this binary was built?
pub fn bundle_is_built() -> bool {
    env!("STRIDE_FRONTEND_BUILT") == "1"
}

/// Is the development bundle the one to serve?
fn use_development_bundle() -> bool {
    env!("STRIDE_USE_DEVELOPMENT_BUNDLE") == "1"
}

/// The app: the embedded bundle, and a directory to serve from disk.
#[derive(Debug, Clone, Default)]
pub struct Spa {
    static_dir: Option<PathBuf>,
}

impl Spa {
    /// The app.
    ///
    /// `static_dir` is the directory a developer points at while they are
    /// working — the rig's own fast loop: rebuild the frontend, reload the
    /// page, and the rig serves the new bundle without being rebuilt. It is
    /// optional, and it is confined: see [`Spa::serve`].
    pub fn new(static_dir: Option<PathBuf>) -> Spa {
        Spa { static_dir }
    }

    /// The directory the rig serves files from, if it was given one.
    pub fn static_dir(&self) -> Option<&Path> {
        self.static_dir.as_deref()
    }

    /// Answer a request for a path.
    pub fn serve(&self, path: &str) -> Response {
        // The guard comes first, and it comes first always: it is a
        // property of the rig, not of the bundle. A rig built without the
        // bundle refuses a path that walks upwards exactly as a rig with it
        // does.
        if !is_safe_path(path) {
            return Response::text(
                400,
                "That path is not one the rig will follow. A path that walks upwards is refused \
                 before anything is looked up.",
            );
        }

        if !bundle_is_built() {
            // A directory to serve from is asked for before this: the
            // development loop is exactly this — a rig built without the
            // bundle, serving the bundle the developer just built.
            if let Some(directory) = &self.static_dir {
                if let Some(response) = serve_from_disk(directory, path) {
                    return response;
                }
            }
            return self.not_built();
        }

        if let Some(directory) = &self.static_dir {
            if let Some(response) = serve_from_disk(directory, path) {
                return response;
            }
        }

        let relative = path.trim_start_matches('/');
        let file = if use_development_bundle() {
            DevelopmentAssets::get(relative)
        } else {
            ReleasedAssets::get(relative)
        };
        if let Some(file) = file {
            return Response::bytes(200, content_type_for(relative), file.data.into_owned());
        }

        // The app has its own router. A path that is not a file is the
        // app's business, not the rig's.
        if !looks_like_a_file(path) {
            return self.index();
        }

        Response::text(
            404,
            "There is no file at that path. The app's own routes are answered by the app.",
        )
    }

    /// The app's `index.html`.
    pub fn index(&self) -> Response {
        let file = if use_development_bundle() {
            DevelopmentAssets::get("index.html")
        } else {
            ReleasedAssets::get("index.html")
        };
        match file {
            Some(file) => {
                Response::bytes(200, content_type_for("index.html"), file.data.into_owned())
            }
            None => self.not_built(),
        }
    }

    /// The page the rig serves when it was built without the bundle.
    fn not_built(&self) -> Response {
        Response::html(
            200,
            "<!doctype html>\n\
             <html lang=\"en\">\n\
             <head><meta charset=\"utf-8\"><title>Stride</title></head>\n\
             <body style=\"font-family: system-ui, sans-serif; margin: 3rem auto; max-width: 40rem; \
             line-height: 1.5\">\n\
             <h1>The app was not built</h1>\n\
             <p>This binary was compiled without the app bundle, so there is nothing to serve \
             yet. Build the rig again from a checkout that carries the released bundle.</p>\n\
             </body></html>\n",
        )
    }
}

/// Is this path one the rig will follow?
///
/// A path that walks upwards is refused. So is a path with a backslash in
/// it — on Windows a backslash is a separator, and a path that mixes the
/// two separators is a path whose meaning depends on the platform — and a
/// path with a null byte in it, which is not a path at all.
pub fn is_safe_path(path: &str) -> bool {
    if path.is_empty() || !path.starts_with('/') {
        return false;
    }
    if path.contains('\0') || path.contains('\\') {
        return false;
    }
    path.split('/').all(|segment| segment != "..")
}

/// Does this path name a file rather than a route?
pub fn looks_like_a_file(path: &str) -> bool {
    let last = path.rsplit('/').next().unwrap_or_default();
    match last.rsplit_once('.') {
        Some((_, extension)) => !extension.is_empty(),
        None => false,
    }
}

/// The content type of a file, from its name.
pub fn content_type_for(path: &str) -> &'static str {
    let extension = path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// Serve a path from a directory on disk, if it is there.
///
/// The file that is served has to be inside the directory, after the path
/// has been canonicalised: a symbolic link out of the directory is caught
/// there, and not before.
fn serve_from_disk(directory: &Path, path: &str) -> Option<Response> {
    let root = std::fs::canonicalize(directory).ok()?;
    let relative = path.trim_start_matches('/');
    if relative.is_empty() {
        return None;
    }
    let candidate = root.join(relative);
    let resolved = std::fs::canonicalize(&candidate).ok()?;
    if !resolved.starts_with(&root) {
        return None;
    }
    if !resolved.is_file() {
        return None;
    }
    let bytes = std::fs::read(&resolved).ok()?;
    Some(Response::bytes(200, content_type_for(relative), bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_that_walks_upwards_is_refused() {
        assert!(!is_safe_path("/../../etc/passwd"));
        assert!(!is_safe_path("/a/../../b"));
        assert!(!is_safe_path("/a\\b"));
        assert!(is_safe_path("/runs"));
        assert!(is_safe_path("/assets/index.js"));
    }

    #[test]
    fn a_file_is_told_from_a_route() {
        assert!(looks_like_a_file("/assets/index.js"));
        assert!(!looks_like_a_file("/runs"));
        assert!(!looks_like_a_file("/compare/12"));
    }
}
