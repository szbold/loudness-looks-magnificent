//! Deterministic synthetic audio source — the CI/e2e `AudioSource` fake
//! (FR-M1-6 `--fake-audio <seed>`; NFR-6 requires fake sources instead of
//! audio devices in tests).

use crate::errors::LmemError;
use crate::traits::AudioSource;

/// Default evaluation rate in Hz (FR-M1-2 contract; 44.1/48 kHz are the
/// consumer norms; 48 kHz is safe on Linux/Windows).
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
/// Default chunk length: 10 ms of mono samples at [`DEFAULT_SAMPLE_RATE`].
pub const DEFAULT_CHUNK_SAMPLES: usize = 480;

/// Deterministic SplitMix64 (tiny, dependency-free, well-defined).
#[derive(Clone, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Deterministic synthetic source: a 220 Hz sine under a slow (8 s) amplitude
/// envelope, seeded phase. Same seed ⇒ identical stream (NFR-6).
#[derive(Clone, Debug)]
pub struct SynthAudioSource {
    sample_rate: u32,
    chunk_len: usize,
    phase0: f64,
    sample_index: u64,
}

impl SynthAudioSource {
    /// New source emitting `chunk_len` samples at `sample_rate` per chunk.
    pub fn new(seed: u64, sample_rate: u32, chunk_len: usize) -> Result<Self, LmemError> {
        if sample_rate == 0 {
            return Err(LmemError::Config("sample rate must be positive".into()));
        }
        if chunk_len == 0 {
            return Err(LmemError::Config("chunk length must be positive".into()));
        }
        let phase0 =
            (SplitMix64::new(seed).next() as f64 / u64::MAX as f64) * std::f64::consts::TAU;
        Ok(Self {
            sample_rate,
            chunk_len,
            phase0,
            sample_index: 0,
        })
    }

    /// Exact synthetic value at a global sample index (shared by chunks and
    /// tests so the stream never depends on chunking).
    fn sample_at(&self, index: u64) -> f32 {
        let t = index as f64 / self.sample_rate as f64;
        let env = 0.25 + 0.25 * (1.0 + (std::f64::consts::TAU * (t / 8.0) + self.phase0).sin());
        let period = std::f64::consts::TAU * 220.0 * t;
        let s = env * period.sin();
        (s.clamp(-1.0, 1.0)) as f32
    }
}

impl AudioSource for SynthAudioSource {
    fn next_chunk(&mut self, out: &mut Vec<f32>) {
        out.clear();
        for i in 0..self.chunk_len {
            out.push(self.sample_at(self.sample_index + i as u64));
        }
        self.sample_index += self.chunk_len as u64;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test policy (CONVENTIONS.md)
    use super::*;

    #[test]
    fn same_seed_yields_identical_stream() {
        let mut a = SynthAudioSource::new(42, DEFAULT_SAMPLE_RATE, DEFAULT_CHUNK_SAMPLES).unwrap();
        let mut b = SynthAudioSource::new(42, DEFAULT_SAMPLE_RATE, DEFAULT_CHUNK_SAMPLES).unwrap();
        let (mut ca, mut cb) = (Vec::new(), Vec::new());
        a.next_chunk(&mut ca);
        b.next_chunk(&mut cb);
        assert_eq!(ca, cb);
    }

    #[test]
    fn amplitude_stays_in_contract() {
        let mut src = SynthAudioSource::new(7, DEFAULT_SAMPLE_RATE, DEFAULT_CHUNK_SAMPLES).unwrap();
        let mut chunk = Vec::new();
        for _ in 0..4 {
            src.next_chunk(&mut chunk);
            for &s in &chunk {
                assert!(s.abs() <= 1.0, "sample {s} violates the [−1, 1] contract");
            }
        }
    }

    #[test]
    fn zero_configuration_rejected() {
        assert!(SynthAudioSource::new(1, 0, 64).is_err());
        assert!(SynthAudioSource::new(1, 48_000, 0).is_err());
    }
}
