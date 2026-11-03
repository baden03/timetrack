//! Time arithmetic (concept §8 and §18): durations, validation, midnight
//! splitting performed by the author's client, and the consistency check the
//! hub runs on a recorded split without any timezone database.

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

pub const MAX_INTERVAL_MINUTES: i64 = 24 * 60;
/// An entry may end this far ahead of the clock, so that a time can be
/// rounded up (14:27 to 14:30) (concept §8.3).
pub const FUTURE_TOLERANCE_MINUTES: i64 = 15;
const MIN_OFFSET_MINUTES: i32 = -12 * 60;
const MAX_OFFSET_MINUTES: i32 = 14 * 60;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TimeError {
    #[error("end must be after start")]
    EndNotAfterStart,
    #[error("interval longer than 24 hours")]
    TooLong,
    #[error("duration must be between one minute and 24 hours")]
    DurationOutOfRange,
    #[error("entry ends in the future")]
    InFuture,
    #[error("local time does not exist in this zone")]
    NonexistentLocalTime,
    #[error("split group is inconsistent: {0}")]
    InconsistentSplit(SplitProblem),
}

impl TimeError {
    /// Machine-readable code; the catalog entry is `error.<code>`.
    pub fn code(&self) -> &'static str {
        match self {
            TimeError::EndNotAfterStart => "end_not_after_start",
            TimeError::TooLong => "interval_too_long",
            TimeError::DurationOutOfRange => "duration_out_of_range",
            TimeError::InFuture => "entry_in_future",
            TimeError::NonexistentLocalTime => "nonexistent_local_time",
            TimeError::InconsistentSplit(_) => "inconsistent_split",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SplitProblem {
    #[error("no pieces")]
    Empty,
    #[error("piece numbering is incomplete")]
    Incomplete,
    #[error("gap or overlap between pieces")]
    NotContiguous,
    #[error("pieces do not cover the original span")]
    SpanMismatch,
    #[error("piece minutes do not add up")]
    MinutesMismatch,
    #[error("piece date does not match its start")]
    DateMismatch,
    #[error("boundary is not local midnight")]
    BoundaryNotMidnight,
    #[error("offset out of range")]
    OffsetOutOfRange,
}

/// Whole minutes of real elapsed time between two instants.
pub fn minutes_between(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    (end - start).num_minutes()
}

/// `hh:mm`, hours not capped at 24 (for example `126:30`).
pub fn format_hhmm(minutes: i64) -> String {
    let sign = if minutes < 0 { "-" } else { "" };
    let m = minutes.abs();
    format!("{sign}{}:{:02}", m / 60, m % 60)
}

/// Parses `h:mm`, `hh:mm` or `hhh:mm` into minutes.
pub fn parse_hhmm(text: &str) -> Option<i64> {
    let (h, m) = text.trim().split_once(':')?;
    if m.len() != 2 || h.is_empty() || !h.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let minutes: i64 = m.parse().ok()?;
    if minutes >= 60 {
        return None;
    }
    Some(h.parse::<i64>().ok()? * 60 + minutes)
}

/// Validates an interval entry. `now` is the device clock; the end may be up
/// to `FUTURE_TOLERANCE_MINUTES` after it.
pub fn validate_interval(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<i64, TimeError> {
    if end <= start {
        return Err(TimeError::EndNotAfterStart);
    }
    let minutes = minutes_between(start, end);
    if minutes < 1 {
        return Err(TimeError::EndNotAfterStart);
    }
    if end - start > Duration::hours(24) {
        return Err(TimeError::TooLong);
    }
    if end > now + Duration::minutes(FUTURE_TOLERANCE_MINUTES) {
        return Err(TimeError::InFuture);
    }
    Ok(minutes)
}

/// Validates a duration-only entry.
pub fn validate_duration(minutes: i64) -> Result<i64, TimeError> {
    if (1..=MAX_INTERVAL_MINUTES).contains(&minutes) {
        Ok(minutes)
    } else {
        Err(TimeError::DurationOutOfRange)
    }
}

/// True when two intervals of the same person overlap (judged on instants).
pub fn intervals_overlap(
    a: (DateTime<Utc>, DateTime<Utc>),
    b: (DateTime<Utc>, DateTime<Utc>),
) -> bool {
    a.0 < b.1 && b.0 < a.1
}

/// Interprets a wall-clock time in a zone. The repeated hour at the end of
/// daylight-saving time takes its first occurrence (concept §8.3). A time
/// that was skipped by a clock jump does not exist.
pub fn local_to_utc(local: NaiveDateTime, tz: Tz) -> Result<DateTime<Utc>, TimeError> {
    tz.from_local_datetime(&local)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
        .ok_or(TimeError::NonexistentLocalTime)
}

/// UTC offset in minutes in force at an instant.
pub fn offset_minutes_at(instant: DateTime<Utc>, tz: Tz) -> i32 {
    tz.offset_from_utc_datetime(&instant.naive_utc())
        .fix()
        .local_minus_utc()
        / 60
}

/// Calendar date of an instant in a zone.
pub fn local_date(instant: DateTime<Utc>, tz: Tz) -> NaiveDate {
    instant.with_timezone(&tz).date_naive()
}

/// First instant of a calendar date in a zone. If local midnight does not
/// exist (a clock jump at midnight), it is the first instant after the jump.
pub fn start_of_day(date: NaiveDate, tz: Tz) -> DateTime<Utc> {
    let midnight = date.and_hms_opt(0, 0, 0).expect("midnight exists");
    let mut candidate = midnight;
    // Clock jumps never exceed a few hours; scan minute by minute.
    for _ in 0..(24 * 60) {
        if let Some(t) = tz.from_local_datetime(&candidate).earliest() {
            return t.with_timezone(&Utc);
        }
        candidate += Duration::minutes(1);
    }
    unreachable!("a calendar date always has a first instant")
}

/// One piece of a (possibly split) interval entry, as recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub minutes: i64,
    /// Local calendar date of the piece in the entry's zone.
    pub work_date: NaiveDate,
    /// UTC offset in minutes in force at `start_at`.
    pub start_offset: i32,
    /// UTC offset in minutes in force at `end_at`.
    pub end_offset: i32,
    pub piece_index: u32,
    pub piece_count: u32,
    pub span_start_at: DateTime<Utc>,
    pub span_end_at: DateTime<Utc>,
}

/// Splits an interval at local midnight in the entry's zone. An interval
/// that does not cross midnight yields one piece. Calling this again with
/// changed times is how pieces are re-split or merged.
pub fn split_at_midnight(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    tz: Tz,
) -> Result<Vec<Piece>, TimeError> {
    if end <= start {
        return Err(TimeError::EndNotAfterStart);
    }
    let mut bounds = vec![start];
    let mut date = local_date(start, tz);
    loop {
        date = date.succ_opt().expect("date in range");
        let boundary = start_of_day(date, tz);
        if boundary >= end {
            break;
        }
        if boundary > start {
            bounds.push(boundary);
        }
    }
    bounds.push(end);

    let count = (bounds.len() - 1) as u32;
    Ok(bounds
        .windows(2)
        .enumerate()
        .map(|(i, w)| Piece {
            start_at: w[0],
            end_at: w[1],
            minutes: minutes_between(w[0], w[1]),
            work_date: local_date(w[0], tz),
            start_offset: offset_minutes_at(w[0], tz),
            end_offset: offset_minutes_at(w[1], tz),
            piece_index: i as u32,
            piece_count: count,
            span_start_at: start,
            span_end_at: end,
        })
        .collect())
}

fn shifted_date(instant: DateTime<Utc>, offset_minutes: i32) -> NaiveDate {
    (instant + Duration::minutes(i64::from(offset_minutes))).date_naive()
}

fn is_local_midnight(instant: DateTime<Utc>, offset_minutes: i32) -> bool {
    let local = (instant + Duration::minutes(i64::from(offset_minutes))).naive_utc();
    local.time() == chrono::NaiveTime::MIN
}

/// Checks a recorded split group from its stored values alone, with no
/// timezone database (concept §8.5, "What the hub checks"). The client runs
/// the same check before saving.
pub fn check_split_group(pieces: &[Piece]) -> Result<(), TimeError> {
    use SplitProblem::*;
    let bad = |p| Err(TimeError::InconsistentSplit(p));

    let (Some(first), Some(last)) = (pieces.first(), pieces.last()) else {
        return bad(Empty);
    };
    let count = pieces.len() as u32;
    for (i, p) in pieces.iter().enumerate() {
        if p.piece_index != i as u32 || p.piece_count != count {
            return bad(Incomplete);
        }
        if p.span_start_at != first.span_start_at || p.span_end_at != first.span_end_at {
            return bad(SpanMismatch);
        }
        for offset in [p.start_offset, p.end_offset] {
            if !(MIN_OFFSET_MINUTES..=MAX_OFFSET_MINUTES).contains(&offset) {
                return bad(OffsetOutOfRange);
            }
        }
        if p.end_at <= p.start_at {
            return bad(NotContiguous);
        }
    }
    if first.start_at != first.span_start_at || last.end_at != first.span_end_at {
        return bad(SpanMismatch);
    }
    for pair in pieces.windows(2) {
        if pair[0].end_at != pair[1].start_at {
            return bad(NotContiguous);
        }
        if !is_local_midnight(pair[0].end_at, pair[0].end_offset)
            || !is_local_midnight(pair[1].start_at, pair[1].start_offset)
        {
            return bad(BoundaryNotMidnight);
        }
    }
    if pieces
        .iter()
        .any(|p| shifted_date(p.start_at, p.start_offset) != p.work_date)
    {
        return bad(DateMismatch);
    }
    let sum: i64 = pieces.iter().map(|p| p.minutes).sum();
    let exact = pieces
        .iter()
        .all(|p| p.minutes == minutes_between(p.start_at, p.end_at));
    if !exact || sum != minutes_between(first.span_start_at, first.span_end_at) {
        return bad(MinutesMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn local(s: &str, tz: Tz) -> DateTime<Utc> {
        local_to_utc(
            NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap(),
            tz,
        )
        .unwrap()
    }

    #[derive(Deserialize)]
    struct Fixtures {
        format: Vec<FormatCase>,
        sums: Vec<SumCase>,
        intervals: Vec<IntervalCase>,
        splits: Vec<SplitCase>,
        invalid: Vec<InvalidCase>,
        bad_groups: Vec<BadGroup>,
        durations: Vec<DurationCase>,
    }
    #[derive(Deserialize)]
    struct FormatCase {
        minutes: i64,
        hhmm: String,
    }
    #[derive(Deserialize)]
    struct SumCase {
        name: String,
        spans: Vec<[String; 2]>,
        total_minutes: i64,
        hhmm: String,
    }
    #[derive(Deserialize)]
    struct IntervalCase {
        name: String,
        tz: String,
        start: String,
        end: String,
        minutes: i64,
    }
    #[derive(Deserialize)]
    struct PieceExpect {
        date: String,
        minutes: i64,
    }
    #[derive(Deserialize)]
    struct SplitCase {
        name: String,
        tz: String,
        start: String,
        end: String,
        pieces: Vec<PieceExpect>,
        total_minutes: i64,
    }
    #[derive(Deserialize)]
    struct InvalidCase {
        name: String,
        start: String,
        end: String,
        now: String,
        code: String,
    }
    #[derive(Deserialize)]
    struct BadGroup {
        name: String,
        tz: String,
        start: String,
        end: String,
        mutation: String,
        problem: String,
    }
    #[derive(Deserialize)]
    struct DurationCase {
        minutes: i64,
        valid: bool,
    }

    fn fixtures() -> Fixtures {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/time.json");
        serde_json::from_str(&std::fs::read_to_string(path).expect("fixtures/time.json"))
            .expect("fixtures parse")
    }

    fn tz(name: &str) -> Tz {
        name.parse().unwrap()
    }

    #[test]
    fn fixture_formatting_round_trips() {
        for c in fixtures().format {
            assert_eq!(format_hhmm(c.minutes), c.hhmm);
            assert_eq!(parse_hhmm(&c.hhmm), Some(c.minutes));
        }
        assert_eq!(parse_hhmm("1:5"), None);
        assert_eq!(parse_hhmm("1:75"), None);
        assert_eq!(parse_hhmm("abc"), None);
    }

    #[test]
    fn fixture_sums() {
        for c in fixtures().sums {
            let total: i64 = c
                .spans
                .iter()
                .map(|[a, b]| {
                    let t = tz("Europe/Berlin");
                    minutes_between(local(a, t), local(b, t))
                })
                .sum();
            assert_eq!(total, c.total_minutes, "{}", c.name);
            assert_eq!(format_hhmm(total), c.hhmm, "{}", c.name);
        }
    }

    #[test]
    fn fixture_real_elapsed_minutes() {
        for c in fixtures().intervals {
            let z = tz(&c.tz);
            assert_eq!(
                minutes_between(local(&c.start, z), local(&c.end, z)),
                c.minutes,
                "{}",
                c.name
            );
        }
    }

    #[test]
    fn fixture_splits() {
        for c in fixtures().splits {
            let z = tz(&c.tz);
            let pieces = split_at_midnight(local(&c.start, z), local(&c.end, z), z).unwrap();
            let got: Vec<(String, i64)> = pieces
                .iter()
                .map(|p| (p.work_date.to_string(), p.minutes))
                .collect();
            let want: Vec<(String, i64)> = c
                .pieces
                .iter()
                .map(|p| (p.date.clone(), p.minutes))
                .collect();
            assert_eq!(got, want, "{}", c.name);
            assert_eq!(
                pieces.iter().map(|p| p.minutes).sum::<i64>(),
                c.total_minutes,
                "{}",
                c.name
            );
            assert_eq!(check_split_group(&pieces), Ok(()), "{}", c.name);
        }
    }

    #[test]
    fn fixture_invalid_entries() {
        for c in fixtures().invalid {
            let err = validate_interval(utc(&c.start), utc(&c.end), utc(&c.now)).unwrap_err();
            assert_eq!(err.code(), c.code, "{}", c.name);
        }
    }

    #[test]
    fn an_end_up_to_fifteen_minutes_ahead_is_allowed() {
        let now = utc("2026-11-02T14:27:00Z");
        let start = utc("2026-11-02T13:00:00Z");
        // Rounding 14:27 up to 14:30 (3 minutes ahead) and the 15 minute limit itself.
        assert_eq!(
            validate_interval(start, utc("2026-11-02T14:30:00Z"), now),
            Ok(90)
        );
        assert_eq!(
            validate_interval(start, utc("2026-11-02T14:42:00Z"), now),
            Ok(102)
        );
        assert_eq!(
            validate_interval(start, utc("2026-11-02T14:43:00Z"), now),
            Err(TimeError::InFuture)
        );
    }

    #[test]
    fn fixture_duration_range() {
        for c in fixtures().durations {
            assert_eq!(
                validate_duration(c.minutes).is_ok(),
                c.valid,
                "{}",
                c.minutes
            );
        }
    }

    #[test]
    fn fixture_bad_groups_are_rejected() {
        for c in fixtures().bad_groups {
            let z = tz(&c.tz);
            let mut pieces = split_at_midnight(local(&c.start, z), local(&c.end, z), z).unwrap();
            assert!(pieces.len() >= 2, "{}", c.name);
            match c.mutation.as_str() {
                "gap" => pieces[1].start_at += Duration::minutes(5),
                "overlap" => pieces[1].start_at -= Duration::minutes(5),
                "drop_piece" => {
                    pieces.remove(0);
                }
                "minutes" => pieces[0].minutes += 1,
                "date" => pieces[1].work_date = pieces[1].work_date.succ_opt().unwrap(),
                "offset" => pieces[0].start_offset = 15 * 60,
                other => panic!("unknown mutation {other}"),
            }
            let err = check_split_group(&pieces).unwrap_err();
            let TimeError::InconsistentSplit(problem) = err else {
                panic!("{}: wrong error {err:?}", c.name);
            };
            assert_eq!(format!("{problem:?}"), c.problem, "{}", c.name);
        }
    }

    #[test]
    fn editing_the_end_merges_the_pieces() {
        let z = tz("Europe/Berlin");
        let start = local("2026-11-02T23:00", z);
        assert_eq!(
            split_at_midnight(start, local("2026-11-03T02:00", z), z)
                .unwrap()
                .len(),
            2
        );
        let merged = split_at_midnight(start, local("2026-11-02T23:30", z), z).unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].minutes, 30);
    }

    #[test]
    fn ambiguous_local_time_uses_first_occurrence() {
        let z = tz("Europe/Berlin");
        // 02:30 on 2026-10-25 happens twice; the first is still summer time.
        assert_eq!(local("2026-10-25T02:30", z), utc("2026-10-25T00:30:00Z"));
    }

    #[test]
    fn skipped_local_time_is_rejected() {
        let z = tz("Europe/Berlin");
        let skipped = NaiveDateTime::parse_from_str("2026-03-29T02:30", "%Y-%m-%dT%H:%M").unwrap();
        assert_eq!(
            local_to_utc(skipped, z),
            Err(TimeError::NonexistentLocalTime)
        );
    }

    #[test]
    fn no_local_midnight_boundary_is_first_instant_of_new_date() {
        // Sao Paulo skipped local midnight (00:00 to 01:00) on 2018-11-04.
        let z = tz("America/Sao_Paulo");
        let start = utc("2018-11-04T01:00:00Z"); // 23:00 on 3 Nov local
        let end = utc("2018-11-04T05:00:00Z");
        let pieces = split_at_midnight(start, end, z).unwrap();
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].end_at, utc("2018-11-04T03:00:00Z"));
        assert_eq!(pieces[1].work_date.to_string(), "2018-11-04");
        assert_eq!(pieces[0].minutes + pieces[1].minutes, 240);
    }

    #[test]
    fn long_interval_over_dst_change_still_consistent() {
        let z = tz("Europe/Berlin");
        let pieces =
            split_at_midnight(local("2026-10-24T01:00", z), utc("2026-10-25T01:00:00Z"), z)
                .unwrap();
        assert_eq!(check_split_group(&pieces), Ok(()));
    }

    #[test]
    fn overlap_is_judged_on_instants() {
        let a = (utc("2026-11-02T09:00:00Z"), utc("2026-11-02T11:00:00Z"));
        let b = (utc("2026-11-02T10:00:00Z"), utc("2026-11-02T12:00:00Z"));
        let c = (utc("2026-11-02T11:00:00Z"), utc("2026-11-02T12:00:00Z"));
        assert!(intervals_overlap(a, b));
        assert!(!intervals_overlap(a, c));
    }
}
