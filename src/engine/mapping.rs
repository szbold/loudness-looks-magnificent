//! Audio → shape mappings (FR-M1-4 now; the FR-M2-2 mapping engine generalizes
//! this calculus to per-vertex/edge targets).

use crate::errors::LmemError;

/// Default scale-map lower bound (FR-M1-4: silence → base size 0.6).
pub const DEFAULT_SCALE_MIN: f32 = 0.6;
/// Default scale-map upper bound (FR-M1-4: full scale → 3.0).
pub const DEFAULT_SCALE_MAX: f32 = 3.0;

/// Monotonic loudness → scale mapping: scale ∈ `[scale_min, scale_max]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleMapping {
    scale_min: f32,
    scale_max: f32,
}

impl ScaleMapping {
    /// New mapping with finite, positive, non-decreasing endpoints.
    pub fn new(scale_min: f32, scale_max: f32) -> Result<Self, LmemError> {
        if !(scale_min.is_finite() && scale_max.is_finite())
            || scale_min <= 0.0
            || scale_max < scale_min
        {
            return Err(LmemError::Config(format!(
                "scale range [{scale_min}, {scale_max}] must be finite, positive, and non-decreasing"
            )));
        }
        Ok(Self {
            scale_min,
            scale_max,
        })
    }

    /// Map loudness (clamped to `[0, 1]`) to a scale factor.
    ///
    /// FR-M1-4 normative properties: monotonic non-decreasing, exact endpoints,
    /// no imaginary/NaN inputs affect the output (input clamped).
    pub fn apply(&self, loudness: f32) -> f32 {
        let l = loudness.clamp(0.0, 1.0);
        self.scale_min + (self.scale_max - self.scale_min) * l
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test policy (CONVENTIONS.md)
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn endpoints_are_exact() {
        let m = ScaleMapping::new(DEFAULT_SCALE_MIN, DEFAULT_SCALE_MAX).unwrap();
        assert_eq!(m.apply(0.0), DEFAULT_SCALE_MIN);
        assert_eq!(m.apply(1.0), DEFAULT_SCALE_MAX);
    }

    #[test]
    fn out_of_range_input_is_clamped() {
        let m = ScaleMapping::new(DEFAULT_SCALE_MIN, DEFAULT_SCALE_MAX).unwrap();
        assert_eq!(m.apply(-5.0), DEFAULT_SCALE_MIN);
        assert_eq!(m.apply(9.0), DEFAULT_SCALE_MAX);
    }

    #[test]
    fn invalid_ranges_are_rejected() {
        assert!(ScaleMapping::new(0.0, 1.0).is_err());
        assert!(ScaleMapping::new(3.0, 0.6).is_err());
        assert!(ScaleMapping::new(f32::NAN, 1.0).is_err());
    }

    proptest! {
        /// FR-M1-4: scale MUST be a monotonic, non-decreasing function of loudness.
        #[test]
        fn monotonic_non_decreasing(l1 in 0f32..=1.0, l2 in 0f32..=1.0) {
            let m = ScaleMapping::new(DEFAULT_SCALE_MIN, DEFAULT_SCALE_MAX).unwrap();
            let (a, b) = if l1 <= l2 { (l1, l2) } else { (l2, l1) };
            prop_assert!(m.apply(a) <= m.apply(b) + f32::EPSILON);
        }
    }
}
