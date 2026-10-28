//! The static files and the single-page application.
//!
//! The desktop serves the built React app from `frontend/dist` — the same
//! files the Python server serves today. Two rules, both of them ported from
//! `app/main.py`:
//!
//! * A path that names a file inside the directory is served as that file.
//! * Any other path that is not under `/api/` is served the app's
//!   `index.html`, because the app is a single-page application and the
//!   router lives in the browser.
//!
//! ## The traversal hole
//!
//! The path arrives from the wire, percent-decoded, so it can hold `..`, an
//! absolute path, or a symlink pointing anywhere. The security audit found a
//! live traversal hole in exactly this behaviour. The Python fix resolves the
//! candidate and refuses anything that is not inside the directory, and the
//! fix is ported here: [`resolve`] canonicalises both the root and the
//! candidate and only answers when the candidate is *under* the root.
//!
//! A file that is not there is not an error — it is the SPA, which is what
//! the reference does.

use std::path::{Path, PathBuf};

/// The content type of a file, by its extension.
///
/// The reference serves the SPA through Starlette, which guesses from the
/// extension with `mimetypes`. The table here covers what a built React app
/// actually ships; anything else is served as a byte stream.
pub fn content_type(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "webmanifest" => "application/manifest+json",
        "txt" => "text/plain; charset=utf-8",
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
        "eot" => "application/vnd.ms-fontobject",
        "wasm" => "application/wasm",
        "xml" => "application/xml",
        "csv" => "text/csv; charset=utf-8",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

/// Resolve a request path to a file inside `dist`.
///
/// `None` means "there is no such file here" — which is not an error, it is
/// the SPA. A path that escapes the directory is also `None`, and it is
/// `None` *after* the resolution, not before: a symlink inside the directory
/// that points outside it is refused exactly like `..` is.
pub fn resolve(dist: &Path, request_path: &str) -> Option<PathBuf> {
    if request_path.is_empty() {
        return None;
    }
    let root = std::fs::canonicalize(dist).ok()?;
    // The path is joined onto the root, so an absolute path cannot escape:
    // "/etc/passwd" becomes "<dist>/etc/passwd".
    let relative = request_path.trim_start_matches('/');
    let candidate = root.join(relative);
    let resolved = std::fs::canonicalize(&candidate).ok()?;
    // The resolved path has to be inside the resolved root. `starts_with` on
    // a `Path` compares components, so "<dist>/assets-evil" is not inside
    // "<dist>".
    if !resolved.starts_with(&root) {
        return None;
    }
    if !resolved.is_file() {
        return None;
    }
    Some(resolved)
}

/// The app's `index.html`.
pub fn index(dist: &Path) -> Option<PathBuf> {
    let index = dist.join("index.html");
    if index.is_file() {
        Some(index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stride-desktop-spa-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("assets")).expect("the scratch directory");
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><title>Stride</title>",
        )
        .expect("the index");
        std::fs::write(dir.join("assets/app.js"), "console.log(1)").expect("the asset");
        dir
    }

    #[test]
    fn a_file_inside_the_directory_is_served() {
        let dist = scratch("inside");
        let resolved = resolve(&dist, "/assets/app.js");
        assert!(resolved.is_some(), "the asset is not resolved");
        assert!(resolved.expect("the asset").ends_with("assets/app.js"));
    }

    #[test]
    fn a_traversal_escapes_nothing() {
        let dist = scratch("traversal");
        // The secret is next to the directory, not inside it.
        let outside = dist
            .parent()
            .expect("a parent")
            .join("stride-desktop-secret.txt");
        std::fs::write(&outside, "the Garmin key").expect("the secret");
        assert!(resolve(&dist, "/../stride-desktop-secret.txt").is_none());
        assert!(resolve(&dist, "/../../etc/passwd").is_none());
        assert!(resolve(&dist, "/assets/../../stride-desktop-secret.txt").is_none());
        std::fs::remove_file(&outside).ok();
    }

    #[test]
    fn a_missing_file_is_not_a_file() {
        let dist = scratch("missing");
        assert!(resolve(&dist, "/activities/42").is_none());
        assert!(resolve(&dist, "/").is_none());
    }
}
