//! Frame-time statistics for `--fps-report` (FR-M1-6, spec §9).

/// Nearest-rank percentile of a **sorted** slice; `None` for empty input.
///
/// Nearest-rank keeps the definition trivially testable and allocation-free.
pub fn percentile_sorted(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let last = sorted.len() - 1;
    let rank = (last as f64 * p).round();
    // p is caller-bounded in practice; the clamp keeps the function total.
    let idx = rank.clamp(0.0, last as f64) as usize;
    Some(sorted[idx])
}

/// The report percentiles consumed by the e2e gate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSummary {
    /// 50th percentile (median), milliseconds.
    pub p50: f64,
    /// 95th percentile, milliseconds.
    pub p95: f64,
    /// 99th percentile, milliseconds.
    pub p99: f64,
}

/// Summarize unsorted frame timings (milliseconds); `None` when empty.
pub fn summarize(mut samples: Vec<f64>) -> Option<FrameSummary> {
    if samples.is_empty() {
        return None;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p50 = percentile_sorted(&samples, 0.50)?;
    let p95 = percentile_sorted(&samples, 0.95)?;
    let p99 = percentile_sorted(&samples, 0.99)?;
    Some(FrameSummary { p50, p95, p99 })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test policy (CONVENTIONS.md)
    use super::*;

    #[test]
    fn empty_input_is_none() {
        assert!(percentile_sorted(&[], 0.5).is_none());
        assert!(summarize(Vec::new()).is_none());
    }

    #[test]
    fn nearest_rank_on_known_data() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile_sorted(&v, 0.0), Some(1.0));
        assert_eq!(percentile_sorted(&v, 0.50), Some(3.0));
        assert_eq!(percentile_sorted(&v, 0.95), Some(5.0));
        assert_eq!(percentile_sorted(&v, 0.99), Some(5.0));
    }

    #[test]
    fn summarize_sorts_its_input() {
        let s = summarize(vec![5.0, 1.0, 3.0, 2.0, 4.0]).unwrap();
        assert_eq!(s.p50, 3.0);
    }

    #[test]
    fn single_sample_yields_that_sample() {
        let s = summarize(vec![7.0]).unwrap();
        assert_eq!(
            s,
            FrameSummary {
                p50: 7.0,
                p95: 7.0,
                p99: 7.0
            }
        );
    }
}
