//! The model client.
//!
//! **One shape, every provider.** The client speaks the OpenAI-compatible
//! chat shape, which Azure OpenAI, DeepSeek, Groq and MiniMax all speak —
//! and which a local server speaks on `127.0.0.1`. "Local" is then only a
//! different base URL, and a local model is a first-class choice.
//!
//! ## The key
//!
//! The key is **not** in the database and **not** in the settings: the
//! settings name the environment variable it is read from, so it is never
//! in an export and never in a log line. It is held the way the Garmin key
//! is held: reached through a closure, with no `Display` and no accessor,
//! so there is nothing to print.
//!
//! ## Nothing leaves the machine unless the athlete says so
//!
//! The rig checks the base URL itself: with `allow_remote` off, only a
//! loopback address may be called. A remote provider is a choice the
//! athlete makes in the settings, and the rig makes it again before every
//! call.

use std::fmt;
use std::time::Duration;

use crate::error::{Error, Result};

/// How long the rig waits for a model.
const TIMEOUT: Duration = Duration::from_secs(90);

/// The environment variable the model's key is read from, when the settings
/// name another one.
pub const DEFAULT_KEY_ENV: &str = "STRIDE_MODEL_KEY";

/// A model that can be asked to write.
///
/// The rig talks to this, and not to the HTTP client, so that the
/// orchestration can be tested without a server.
pub trait ChatModel {
    /// The model's own answer to a prompt, at a base URL.
    ///
    /// A refusal is a refusal: an error here is an error there, and
    /// nothing is kept.
    fn complete(&self, base_url: &str, model: &str, prompt: &str) -> Result<String>;
}

/// A model's key.
///
/// The bytes are private: there is no way to read them out of this type
/// except through [`ModelKey::with_secret`], and the `Debug` output is a
/// fixed phrase.
pub struct ModelKey {
    value: String,
}

impl ModelKey {
    /// Take a key from the environment.
    ///
    /// An empty key is refused: a provider that would take it would take
    /// nothing, and a rig that stored it would hold a credential that opens
    /// nothing.
    pub fn new(value: &str) -> Result<ModelKey> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(Error::key("the model key is empty"));
        }
        Ok(ModelKey {
            value: trimmed.to_string(),
        })
    }

    /// Hand the key to something that needs it, and forget it.
    pub fn with_secret<R>(&self, use_it: impl FnOnce(&str) -> R) -> R {
        use_it(&self.value)
    }
}

impl fmt::Debug for ModelKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a model key")
    }
}

/// The client that speaks to an OpenAI-compatible server.
pub struct ChatClient {
    client: reqwest::blocking::Client,
    key: Option<ModelKey>,
}

impl ChatClient {
    /// A client, with the key it may use.
    pub fn new(key: Option<ModelKey>) -> Result<ChatClient> {
        let client = reqwest::blocking::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(|error| {
                Error::http(format!("the model client could not be built: {error}"))
            })?;
        Ok(ChatClient { client, key })
    }

    /// The URL of the chat completions of a base URL.
    ///
    /// The base URL is the provider's own — `https://api.openai.com/v1` or
    /// `http://127.0.0.1:11434/v1` — and the chat is the one path every
    /// OpenAI-compatible server has.
    pub fn chat_url(base_url: &str) -> String {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    }
}

impl ChatModel for ChatClient {
    fn complete(&self, base_url: &str, model: &str, prompt: &str) -> Result<String> {
        let base = base_url.trim();
        if base.is_empty() {
            return Err(Error::config("no base URL has been set for the model"));
        }
        if !base.starts_with("http://") && !base.starts_with("https://") {
            return Err(Error::config("the base URL of the model is not an address"));
        }
        let body = serde_json::json!({
            "model": model,
            "messages": [{ "role": "system", "content": prompt }],
            "temperature": 0.2,
        });

        let mut request = self
            .client
            .post(ChatClient::chat_url(base))
            .json(&body)
            .header("content-type", "application/json");
        if let Some(key) = &self.key {
            let token = key.with_secret(|secret| secret.to_string());
            request = request.bearer_auth(token);
        }

        let answer = request
            .send()
            .map_err(|error| Error::http(format!("the model could not be reached: {error}")))?;
        let status = answer.status();
        let text = answer.text().map_err(|error| {
            Error::http(format!("the model's answer could not be read: {error}"))
        })?;
        if !status.is_success() {
            // The status is said; the body is not, because it may carry
            // something the athlete did not ask to be logged.
            return Err(Error::http(format!(
                "the model refused the request ({})",
                status.as_u16()
            )));
        }
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|_| Error::http("the model's answer was not JSON"))?;
        content_of(&parsed)
    }
}

/// The text of an answer.
///
/// The chat shape puts it in `choices[0].message.content`; a server that
/// answers with the older `choices[0].text` is read too, because a local
/// server may well be older than the cloud one.
fn content_of(answer: &serde_json::Value) -> Result<String> {
    let choice = answer
        .get("choices")
        .and_then(|choices| choices.get(0))
        .ok_or_else(|| Error::http("the model's answer carried no choice"))?;
    let content = choice
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .or_else(|| choice.get("text").and_then(|text| text.as_str()))
        .ok_or_else(|| Error::http("the model's answer carried no words"))?;
    if content.trim().is_empty() {
        return Err(Error::http("the model's answer was empty"));
    }
    Ok(content.trim().to_string())
}

/// Is this base URL a local one?
///
/// A loopback address is the only one that keeps the payload on the
/// machine: `127.0.0.1`, `localhost` and `::1`. Everything else is a
/// remote provider, and a remote provider needs the athlete's word.
pub fn is_local(base_url: &str) -> bool {
    let rest = match base_url.trim().strip_prefix("http://") {
        Some(rest) => rest,
        None => match base_url.trim().strip_prefix("https://") {
            Some(rest) => rest,
            None => return false,
        },
    };
    let authority = rest.split('/').next().unwrap_or_default();
    let host = match authority.rfind(':') {
        Some(at) if !authority[at + 1..].contains(']') => &authority[..at],
        _ => authority,
    };
    let host = host.trim_matches(|character| character == '[' || character == ']');
    matches!(host, "127.0.0.1" | "localhost" | "::1" | "0.0.0.0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_loopback_address_is_local_and_nothing_else_is() {
        assert!(is_local("http://127.0.0.1:11434/v1"));
        assert!(is_local("http://localhost:1234/v1"));
        assert!(is_local("http://[::1]:1234/v1"));
        assert!(is_local("https://127.0.0.1/v1"));
        assert!(!is_local("https://api.openai.com/v1"));
        assert!(!is_local("http://192.168.1.10:1234/v1"));
        assert!(!is_local("not a url"));
        assert!(!is_local(""));
    }

    #[test]
    fn the_chat_url_is_the_one_every_server_has() {
        assert_eq!(
            ChatClient::chat_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            ChatClient::chat_url("http://127.0.0.1:11434/v1/"),
            "http://127.0.0.1:11434/v1/chat/completions"
        );
    }

    #[test]
    fn the_answer_is_read_out_of_the_chat_shape() {
        let answer = serde_json::json!({
            "choices": [{ "message": { "content": "  The run was 10 km.  " } }]
        });
        assert_eq!(content_of(&answer).unwrap(), "The run was 10 km.");

        // The older shape is read too.
        let answer = serde_json::json!({ "choices": [{ "text": "The run was 10 km." }] });
        assert_eq!(content_of(&answer).unwrap(), "The run was 10 km.");

        // And an answer with no words is a refusal.
        let answer = serde_json::json!({ "choices": [] });
        assert!(content_of(&answer).is_err());
    }

    #[test]
    fn the_key_has_no_printable_form() {
        let key = ModelKey::new("a-key").unwrap();
        assert_eq!(format!("{key:?}"), "a model key");
        assert!(ModelKey::new("  ").is_err());
    }
}
