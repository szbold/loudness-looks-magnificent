//! Loudness analysis (FR-M1-3): sliding-window RMS normalized to `[0, 1]`.

use crate::errors::LmemError;

/// dB range mapped onto `[0, 1]`: loudness 0 = −60 dBFS reference floor,
/// loudness 1 = 0 dBFS. Documented reference normalization (FR-M1-3,
/// docs/PRODUCT.md §5.1).
pub const REF_RANGE_DB: f32 = 60.0;

/// Sliding-window RMS meter with O(1) bookkeeping per sample and O(1) query.
///
/// Fixed preallocated ring buffer (NFR-3): `push` never allocates.
#[derive(Debug)]
pub struct LoudnessMeter {
    window_samples: usize,
    buf: Vec<f32>,
    cursor: usize,
    filled: usize,
    sum_sq: f32,
}

impl LoudnessMeter {
    /// New meter with the given ring length (must be positive).
    pub fn new(window_samples: usize) -> Result<Self, LmemError> {
        if window_samples == 0 {
            return Err(LmemError::Config(
                "loudness window must be at least 1 sample".into(),
            ));
        }
        Ok(Self {
            window_samples,
            buf: Vec::with_capacity(window_samples),
            cursor: 0,
            filled: 0,
            sum_sq: 0.0,
        })
    }

    /// Push one captured chunk (FR-M1-2 source contract: `[-1, 1]` mono).
    ///
    /// Ring bookkeeping is O(n_chunk), allocation-free.
    pub fn push(&mut self, chunk: &[f32]) {
        for &s in chunk {
            let s = s.clamp(-1.0, 1.0); // defensive clamp (source contract)
            let s2 = s * s;
            if self.filled < self.window_samples {
                self.buf.push(s);
                self.sum_sq += s2;
                self.filled += 1;
            } else {
                let old = self.buf[self.cursor];
                self.sum_sq += s2 - old * old;
                self.buf[self.cursor] = s;
                self.cursor += 1;
                if self.cursor == self.window_samples {
                    self.cursor = 0;
                }
            }
        }
    }

    /// Normalized loudness in `[0, 1]`, smoothed to the configured window.
    ///
    /// Mapping: dBFS ∈ `[-REF_RANGE_DB, 0]` → `[0, 1]` (clamped; silence → 0).
    pub fn loudness(&self) -> f32 {
        if self.filled == 0 {
            return 0.0;
        }
        let rms = (self.sum_sq / self.filled as f32).max(0.0).sqrt();
        if rms <= 0.0 {
            return 0.0;
        }
        let db = 20.0 * rms.log10();
        ((db + REF_RANGE_DB) / REF_RANGE_DB).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test policy (CONVENTIONS.md)
    use super::*;

    #[test]
    fn empty_meter_is_silent() {
        assert_eq!(LoudnessMeter::new(64).unwrap().loudness(), 0.0);
    }

    #[test]
    fn silence_is_zero_loudness() {
        let mut m = LoudnessMeter::new(64).unwrap();
        m.push(&[0.0; 64]);
        assert_eq!(m.loudness(), 0.0);
    }

    #[test]
    fn steady_state_matches_signal_rms() {
        let mut m = LoudnessMeter::new(64).unwrap();
        // Constant 0.5 signal: RMS 0.5 → −6.02 dBFS → well inside [0, 1].
        let chunk = [0.5f32; 16];
        for _ in 0..8 {
            m.push(&chunk); // slide past one full window
        }
        let want = (20.0 * 0.5f32.log10() + REF_RANGE_DB) / REF_RANGE_DB;
        let got = m.loudness();
        assert!((got - want).abs() < 1e-6f32, "got {got}, want {want}");
    }

    #[test]
    fn zero_window_is_rejected() {
        assert!(LoudnessMeter::new(0).is_err());
    }
}
