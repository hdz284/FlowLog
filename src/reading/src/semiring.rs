use differential_dataflow::difference::{IsZero, Monoid, Multiply, Semigroup};
use serde::{Deserialize, Serialize};

// Re-export Present for convenience
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
use differential_dataflow::difference::Present;

// Conditional compilation for semiring type selection
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
pub type Semiring = Present;

#[cfg(all(feature = "isize-type", not(feature = "present-type")))]
pub type Semiring = isize;

// Helper function to create the appropriate semiring value
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
pub fn semiring_one() -> Semiring {
    Present {}
}

#[cfg(all(feature = "isize-type", not(feature = "present-type")))]
pub fn semiring_one() -> Semiring {
    1
}

// Compile-time check to ensure exactly one semiring feature is enabled
#[cfg(all(feature = "present-type", feature = "isize-type"))]
compile_error!("Cannot enable both present-type and isize-type features at once");

#[cfg(not(any(feature = "present-type", feature = "isize-type")))]
compile_error!("Must enable exactly one semiring feature: either present-type or isize-type");

// Debug: expose which semiring type is active
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
pub const SEMIRING_TYPE: &str = "Present";

#[cfg(all(feature = "isize-type", not(feature = "present-type")))]
pub const SEMIRING_TYPE: &str = "isize";

/// MIN Semiring
#[derive(Copy, Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct Min {
    pub value: u32,
}

impl Min {
    /// Creates a new `Min` with a value.
    pub fn new(value: u32) -> Self {
        Min { value }
    }

    /// Creates a new `Min` representing infinity (u32::MAX).
    /// This serves as the additive identity in the MIN semiring:
    /// min(a, ∞) = a for any value a.
    pub fn infinity() -> Self {
        Min { value: u32::MAX }
    }

    /// Returns true if this Min represents infinity.
    pub fn is_infinity(&self) -> bool {
        self.value == u32::MAX
    }
}

impl IsZero for Min {
    fn is_zero(&self) -> bool {
        false // always return false
    }
}

impl Semigroup for Min {
    fn plus_equals(&mut self, rhs: &Self) {
        self.value = std::cmp::min(self.value, rhs.value);
    }
}

impl Monoid for Min {
    fn zero() -> Self {
        Min::infinity() // additive identity is infinity
    }
}

// For converting i64 differences to Min (preserves the Min value)
impl Multiply<i64> for Min {
    type Output = Min;

    fn multiply(self, _rhs: &i64) -> Self::Output {
        self
    }
}

// Convenience implementations for easier use
impl From<u32> for Min {
    fn from(value: u32) -> Self {
        Min::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_semigroup() {
        let mut a = Min::new(5);
        let b = Min::new(3);
        a.plus_equals(&b);
        assert_eq!(a.value, 3);

        let mut inf = Min::infinity();
        inf.plus_equals(&Min::new(42));
        assert_eq!(inf.value, 42);
    }

    #[test]
    fn test_min_zero() {
        let zero = Min::zero();
        assert!(zero.is_zero());
        assert!(zero.is_infinity());
        assert_eq!(zero.value, u32::MAX);
    }
}
