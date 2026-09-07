//! Two in-memory ring buffers.
//!
//! **The Trace ring.** The default log level is Info, which means the bug has already
//! happened at the wrong level. Asking a user to reproduce at Trace works only for
//! reproducible bugs, and those are the easy ones. So the last 500 records are held
//! here *regardless of the configured file level*, and written into a Diagnostics
//! Bundle as `trace-tail.log`. It costs a fixed allocation and it is frequently the
//! only record of an intermittent failure.
//!
//! **The problem ring.** Errors that are logged but never surfaced are never reported.
//! The last ten feed Settings → Advanced → Recent problems, with their correlation
//! ids, so a developer can quote one without opening a log file.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use specta::Type;

use crate::error::ErrorCode;

/// Enough to hold the run-up to a failure, small enough to never matter.
const TRACE_CAPACITY: usize = 500;
/// Settings → Advanced shows ten. More is a log file, and there is one of those.
const PROBLEM_CAPACITY: usize = 10;

/// One captured log record. Held whatever the configured file level is.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraceRecord {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// One failure, as Settings → Advanced shows it.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProblemRecord {
    pub timestamp: String,
    pub code: ErrorCode,
    pub correlation_id: String,
    pub context: BTreeMap<String, String>,
}

fn trace_ring() -> &'static Mutex<VecDeque<TraceRecord>> {
    static RING: OnceLock<Mutex<VecDeque<TraceRecord>>> = OnceLock::new();
    RING.get_or_init(|| Mutex::new(VecDeque::with_capacity(TRACE_CAPACITY)))
}

fn problem_ring() -> &'static Mutex<VecDeque<ProblemRecord>> {
    static RING: OnceLock<Mutex<VecDeque<ProblemRecord>>> = OnceLock::new();
    RING.get_or_init(|| Mutex::new(VecDeque::with_capacity(PROBLEM_CAPACITY)))
}

/// Record a log line. Called from the logging pipeline for **every** record, at every
/// level, before the level filter for the file target is applied.
pub fn push_trace(record: TraceRecord) {
    // A poisoned lock here must not take the app down: this is the diagnostics path,
    // and logging must never become the thing that breaks the app.
    let Ok(mut ring) = trace_ring().lock() else {
        return;
    };
    if ring.len() == TRACE_CAPACITY {
        ring.pop_front();
    }
    ring.push_back(record);
}

/// Record a failure. Called by `AppError`'s builder, so nothing can produce an error
/// that Recent problems does not know about.
pub fn push_problem(code: ErrorCode, correlation_id: &str, context: &BTreeMap<String, String>) {
    let Ok(mut ring) = problem_ring().lock() else {
        return;
    };
    if ring.len() == PROBLEM_CAPACITY {
        ring.pop_front();
    }
    ring.push_back(ProblemRecord {
        timestamp: crate::logging::timestamp(),
        code,
        correlation_id: correlation_id.to_owned(),
        context: context.clone(),
    });
}

/// The Trace tail, oldest first — the order a log file is read in.
pub fn trace_tail() -> Vec<TraceRecord> {
    trace_ring()
        .lock()
        .map(|ring| ring.iter().cloned().collect())
        .unwrap_or_default()
}

/// Recent problems, newest first — the order a list is read in.
pub fn recent_problems() -> Vec<ProblemRecord> {
    problem_ring()
        .lock()
        .map(|ring| ring.iter().rev().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(message: &str) -> TraceRecord {
        TraceRecord {
            timestamp: "2026-09-07T14:22:31.123+02:00".to_owned(),
            level: "DEBUG".to_owned(),
            target: "wgm::test".to_owned(),
            message: message.to_owned(),
        }
    }

    #[test]
    fn the_trace_ring_keeps_the_most_recent_records_and_drops_the_oldest() {
        for index in 0..TRACE_CAPACITY + 25 {
            push_trace(record(&format!("record {index}")));
        }

        let tail = trace_tail();

        assert_eq!(tail.len(), TRACE_CAPACITY, "the ring must be bounded");
        assert!(
            tail.last()
                .expect("the ring is not empty")
                .message
                .ends_with(&format!("{}", TRACE_CAPACITY + 24)),
            "the newest record must survive"
        );
    }

    /// Asserted as *relative* order rather than absolute position: the rings are
    /// process-global by design, and other tests in this binary push into them too.
    #[test]
    fn problems_come_back_newest_first() {
        push_problem(ErrorCode::ExportFailed, "ring01", &BTreeMap::new());
        push_problem(ErrorCode::ImportInvalid, "ring02", &BTreeMap::new());

        let problems = recent_problems();
        let position = |id: &str| problems.iter().position(|p| p.correlation_id == id);

        let newer = position("ring02").expect("the second push must be in the ring");
        let older = position("ring01").expect("the first push must be in the ring");

        assert!(newer < older, "recent_problems() must return newest first");
    }
}
