//! Dated history checkpoints and per-attempt transport bounds.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use stride_core::date::Date;
use stride_ingest::{HttpRequest, HttpResponse, HttpTransport, TransportError};

use crate::error::{ApiError, ApiResult};
use crate::routes::core_date;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct HistoryProgress {
    pub target_from: String,
    pub next_date: Option<String>,
    pub checked_from: Option<String>,
    pub checked_to: Option<String>,
    pub window_days: i64,
}

impl HistoryProgress {
    pub fn new(from: &str, today: Date) -> Self {
        Self {
            target_from: from.to_string(),
            next_date: Some(today.to_iso()),
            checked_from: None,
            checked_to: None,
            window_days: 28,
        }
    }

    pub fn extend(&mut self, from: &str, today: Date) -> ApiResult<()> {
        self.validate()?;
        if core_date(&self.target_from)? > today
            || self
                .next_date
                .as_deref()
                .map(core_date)
                .transpose()?
                .is_some_and(|date| date > today)
            || self
                .checked_to
                .as_deref()
                .map(core_date)
                .transpose()?
                .is_some_and(|date| date > today)
        {
            return Err(ApiError::internal(
                "the stored history progress is in the future",
            ));
        }
        if core_date(from)? > today {
            return Err(ApiError::bad_request("the history target is in the future"));
        }
        if from < self.target_from.as_str() {
            if self.next_date.is_none() {
                self.next_date = Some(core_date(&self.target_from)?.add_days(-1).to_iso());
            }
            self.target_from = from.to_string();
        }
        // A long absence can leave a gap between old coverage and the overlap.
        if let Some(to) = &self.checked_to {
            if core_date(to)?.add_days(13) < today {
                self.next_date = Some(today.to_iso());
                self.checked_from = None;
                self.checked_to = None;
            }
        }
        Ok(())
    }

    pub fn window(&self) -> ApiResult<Option<(String, String)>> {
        self.validate()?;
        let Some(to) = &self.next_date else {
            return Ok(None);
        };
        let end = core_date(to)?;
        let start = end.add_days(-(self.window_days - 1));
        Ok(Some((
            start.to_iso().max(self.target_from.clone()),
            end.to_iso(),
        )))
    }

    pub fn confirm(&mut self, from: &str, to: &str) -> ApiResult<()> {
        let start = core_date(from)?;
        let end = core_date(to)?;
        if start > end || from < self.target_from.as_str() {
            return Err(ApiError::internal("the checked history range is not valid"));
        }
        // A recent refresh after an absence must not bridge unqueried dates.
        // Keep one contiguous range; older completed dates may be checked again.
        if let Some(old_to) = &self.checked_to {
            if core_date(old_to)?.add_days(1) < start {
                self.checked_from = None;
                self.checked_to = None;
                self.next_date = Some(to.to_string());
            }
        }
        if let Some(next) = &self.next_date {
            if from <= next.as_str() && next.as_str() <= to {
                self.checked_from = Some(from.to_string());
                self.next_date = if from <= self.target_from.as_str() {
                    None
                } else {
                    Some(start.add_days(-1).to_iso())
                };
            }
        }
        if self.checked_to.as_deref().map_or(true, |old| old < to) {
            self.checked_to = Some(to.to_string());
        }
        self.window_days = 28;
        Ok(())
    }

    pub fn narrow_window(&mut self, from: &str, to: &str) -> ApiResult<()> {
        let mut days = 1;
        let mut day = core_date(from)?;
        let end = core_date(to)?;
        if day > end {
            return Err(ApiError::internal("the history window is reversed"));
        }
        while day < end {
            day = day.add_days(1);
            days += 1;
        }
        self.window_days = (days / 2).max(1);
        Ok(())
    }

    pub fn validate(&self) -> ApiResult<()> {
        let target = core_date(&self.target_from)?;
        if !(1..=28).contains(&self.window_days) {
            return Err(ApiError::internal("the stored history window is not valid"));
        }
        if let Some(next) = &self.next_date {
            if core_date(next)? < target {
                return Err(ApiError::internal("the stored history cursor is not valid"));
            }
        }
        match (&self.checked_from, &self.checked_to) {
            (Some(from), Some(to))
                if target <= core_date(from)? && core_date(from)? <= core_date(to)? => {}
            (None, None) => {}
            _ => {
                return Err(ApiError::internal(
                    "the stored history coverage is not valid",
                ))
            }
        }
        if self.next_date.is_none()
            && self.checked_from.as_deref() != Some(self.target_from.as_str())
        {
            return Err(ApiError::internal(
                "the stored history target is not checked",
            ));
        }
        if let (Some(next), Some(from)) = (&self.next_date, &self.checked_from) {
            if next >= from {
                return Err(ApiError::internal(
                    "the stored history cursor overlaps checked dates",
                ));
            }
        }
        Ok(())
    }

    pub fn pending(&self) -> bool {
        self.next_date.is_some()
    }
}

#[derive(Default)]
struct Usage {
    requests: usize,
    response_bytes: usize,
    paused: bool,
}

pub(crate) struct BudgetTransport {
    inner: Arc<dyn HttpTransport>,
    usage: Mutex<Usage>,
    started: Instant,
    requests: usize,
    response_bytes: usize,
    duration: Duration,
}

impl BudgetTransport {
    pub fn new(
        inner: Arc<dyn HttpTransport>,
        requests: usize,
        response_bytes: usize,
        seconds: u64,
    ) -> Self {
        Self {
            inner,
            usage: Mutex::new(Usage::default()),
            started: Instant::now(),
            requests,
            response_bytes,
            duration: Duration::from_secs(seconds),
        }
    }

    pub fn paused(&self) -> ApiResult<bool> {
        self.usage
            .lock()
            .map(|usage| usage.paused)
            .map_err(|_| ApiError::internal("the sync budget could not be read"))
    }
}

impl HttpTransport for BudgetTransport {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        {
            let mut usage = self
                .usage
                .lock()
                .map_err(|_| TransportError::new("the sync budget could not be read"))?;
            if usage.requests >= self.requests
                || usage.response_bytes >= self.response_bytes
                || self.started.elapsed() >= self.duration
            {
                usage.paused = true;
                return Err(TransportError::new("the bounded sync attempt is paused"));
            }
            usage.requests += 1;
        }
        let response = self.inner.execute(request)?;
        let mut usage = self
            .usage
            .lock()
            .map_err(|_| TransportError::new("the sync budget could not be read"))?;
        // HTTP response-body bytes, not wire bytes. A bounded in-flight response
        // may cross the soft byte/time limit; the next request is not started.
        usage.response_bytes = usage.response_bytes.saturating_add(response.body.len());
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_windows_repeat_until_confirmed_and_can_extend_older() -> ApiResult<()> {
        let today = core_date("2026-05-12")?;
        let mut progress = HistoryProgress::new("2026-04-12", today);
        let window = progress.window()?;
        assert_eq!(window, progress.window()?);
        let (from, to) = window.ok_or_else(|| ApiError::internal("the test window is absent"))?;
        progress.confirm(&from, &to)?;
        assert_eq!(progress.next_date.as_deref(), Some("2026-04-14"));
        progress.confirm("2026-04-12", "2026-04-14")?;
        assert!(!progress.pending());
        progress.extend("2026-03-01", today)?;
        assert_eq!(progress.next_date.as_deref(), Some("2026-04-11"));
        assert_eq!(progress.target_from, "2026-03-01");
        Ok(())
    }

    #[test]
    fn a_recent_day_after_an_absence_does_not_claim_the_gap() -> ApiResult<()> {
        let mut progress = HistoryProgress::new("2026-04-01", core_date("2026-05-01")?);
        progress.confirm("2026-04-15", "2026-05-01")?;
        progress.extend("2026-04-01", core_date("2026-05-07")?)?;
        progress.confirm("2026-05-07", "2026-05-07")?;
        assert_eq!(progress.checked_from.as_deref(), Some("2026-05-07"));
        assert_eq!(progress.checked_to.as_deref(), Some("2026-05-07"));
        assert_eq!(progress.next_date.as_deref(), Some("2026-05-06"));
        Ok(())
    }

    #[test]
    fn capped_windows_shrink_without_claiming_coverage() -> ApiResult<()> {
        let mut progress = HistoryProgress::new("2026-04-01", core_date("2026-05-12")?);
        progress.narrow_window("2026-04-29", "2026-05-12")?;
        assert_eq!(
            progress.window()?,
            Some(("2026-05-06".to_string(), "2026-05-12".to_string()))
        );
        assert!(progress.checked_from.is_none());
        assert!(progress.narrow_window("2026-05-12", "2026-05-01").is_err());
        progress.window_days = 0;
        assert!(progress.window().is_err());
        Ok(())
    }

    #[test]
    fn invalid_persisted_progress_cannot_claim_completion_or_future_coverage() -> ApiResult<()> {
        let today = core_date("2026-05-12")?;
        let mut progress = HistoryProgress::new("2026-04-01", today);
        progress.next_date = None;
        assert!(progress.validate().is_err());
        progress.next_date = Some(today.to_iso());
        progress.confirm("2026-05-01", "2026-05-12")?;
        progress.next_date = Some("2026-05-01".to_string());
        assert!(progress.validate().is_err());
        progress.next_date = Some("2026-04-30".to_string());
        progress.checked_to = Some("2026-05-13".to_string());
        assert!(progress.extend("2026-04-01", today).is_err());
        Ok(())
    }
}
