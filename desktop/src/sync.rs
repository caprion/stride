//! The rig's pull policy.
//!
//! **The client makes one exchange and one attempt; the rig may make one
//! more.** That is the contract's rule (`docs/contract/core-contract.md`
//! §7 — "The retry rule, and who owns it"), and this file is the rig's
//! side of it: the same rule the phone's `PullPolicy` keeps in Kotlin, in
//! the same place — the shell, not the client.
//!
//! ## Why it is here and not in the router
//!
//! The router (`stride-api`) performs **one** token exchange and **one**
//! HTTP attempt per call, and it adds no retry of its own: a session that
//! has ended comes back as the typed failure `needs_reconnect`. If the
//! router retried as well, the pull would exchange and try twice, and
//! "exactly one" would be a coincidence rather than a rule.
//!
//! So the rig — the shell on this host — makes the second attempt, and
//! only the second one. A second refusal stops the pull: the answer is
//! the refusal, and the next scheduled sync tries again.
//!
//! ## What a refusal is
//!
//! The typed failure `needs_reconnect` (the session ended) and the typed
//! failure `garmin_rejected` (Garmin would not take the key) are the same
//! thing to the rig: the key has to be exchanged again, once, or it has
//! to be replaced. Both travel as a `401`.

use crate::api::{Request, Response, Router};

/// How many attempts a pull makes: the first, and one more.
pub const MAX_ATTEMPTS: usize = 2;

/// The typed failures that earn one more attempt.
const REFUSALS: [&str; 2] = ["needs_reconnect", "garmin_rejected"];

/// Is this answer a refusal that earns one more attempt?
///
/// A `401` is a refusal whatever shape it takes: the router's typed
/// failures both travel as one, and the status is what the rig reads
/// first.
pub fn is_refusal(status: u16, body: &[u8]) -> bool {
    if status == 401 {
        return true;
    }
    let Ok(text) = std::str::from_utf8(body) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    match parsed.get("code").and_then(|code| code.as_str()) {
        Some(code) => REFUSALS.contains(&code),
        None => false,
    }
}

/// The pull, with the rule in it.
///
/// One attempt. A refusal is answered with **one** more attempt, and that
/// attempt is the last: a second refusal is the answer.
pub fn pull(router: &dyn Router, request: &Request<'_>) -> Response {
    let mut answer = router.call(request);
    let mut attempt = 1;
    while attempt < MAX_ATTEMPTS && is_refusal(answer.status, &answer.body) {
        answer = router.call(request);
        attempt += 1;
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Response;
    use std::sync::Mutex;

    /// A router that answers with a fixed sequence, and counts the calls.
    struct Counting {
        answers: Mutex<Vec<Response>>,
        calls: Mutex<usize>,
    }

    impl Counting {
        fn new(answers: Vec<Response>) -> Counting {
            Counting {
                answers: Mutex::new(answers),
                calls: Mutex::new(0),
            }
        }

        fn count(&self) -> usize {
            match self.calls.lock() {
                Ok(guard) => *guard,
                Err(_) => 0,
            }
        }
    }

    impl Router for Counting {
        fn call(&self, _request: &Request<'_>) -> Response {
            if let Ok(mut calls) = self.calls.lock() {
                *calls += 1;
            }
            let mut answers = match self.answers.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if answers.len() > 1 {
                answers.remove(0)
            } else {
                answers
                    .first()
                    .cloned()
                    .unwrap_or_else(|| Response::json(500, "{}"))
            }
        }
    }

    fn request() -> Request<'static> {
        Request {
            method: "POST",
            path: "/api/v1/sync/run",
            query: "",
            body: &[],
        }
    }

    #[test]
    fn a_pull_that_works_is_not_retried() {
        let router = Counting::new(vec![Response::json(200, "{\"activities\":1}")]);
        let answer = pull(&router, &request());
        assert_eq!(answer.status, 200);
        assert_eq!(router.count(), 1);
    }

    #[test]
    fn a_refusal_is_answered_with_one_more_attempt() {
        let router = Counting::new(vec![
            Response::json(401, "{\"code\":\"needs_reconnect\"}"),
            Response::json(200, "{\"activities\":1}"),
        ]);
        let answer = pull(&router, &request());
        assert_eq!(answer.status, 200);
        assert_eq!(router.count(), 2, "one attempt, and one more");
    }

    #[test]
    fn a_second_refusal_stops_the_pull() {
        let router = Counting::new(vec![
            Response::json(401, "{\"code\":\"needs_reconnect\"}"),
            Response::json(401, "{\"code\":\"needs_reconnect\"}"),
        ]);
        let answer = pull(&router, &request());
        assert_eq!(answer.status, 401);
        assert_eq!(router.count(), 2, "two attempts, and no more");
    }

    #[test]
    fn a_failure_that_is_not_a_refusal_is_not_retried() {
        let router = Counting::new(vec![Response::json(500, "{\"code\":\"internal\"}")]);
        let answer = pull(&router, &request());
        assert_eq!(answer.status, 500);
        assert_eq!(router.count(), 1);
    }

    #[test]
    fn a_refusal_is_a_refusal_in_every_shape() {
        assert!(is_refusal(401, b"{}"));
        assert!(is_refusal(200, b"{\"code\":\"needs_reconnect\"}"));
        assert!(is_refusal(200, b"{\"code\":\"garmin_rejected\"}"));
        assert!(!is_refusal(200, b"{\"code\":\"not_found\"}"));
        assert!(!is_refusal(500, b"not json"));
    }
}
