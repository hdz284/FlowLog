pub mod arrangements;
pub mod config;
pub mod inspect;
pub mod reader;
pub mod rel;
pub mod row;
pub mod session;

// export configuration constants for backwards compatibility
pub use config::{FALLBACK_ARITY, KV_MAX, PROD_MAX, ROW_MAX};

// feature propagation through dependency chain && mutually exclusive feature configuration
// workspace
//     ↓ --features isize-type
// executing crate
//     ↓ enables isize-type = ["reading/isize-type", "macros/isize-type"]
// macros crate
//     ↓ enables isize-type = ["reading/isize-type"]
// reading crate
//     ↓ compiles with isize type

pub type Time = ();
pub type Iter = u16;

// Conditional compilation for semiring type selection
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
use differential_dataflow::difference::Present;
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

// debug: expose which semiring type is active
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
pub const SEMIRING_TYPE: &str = "Present";

#[cfg(all(feature = "isize-type", not(feature = "present-type")))]
pub const SEMIRING_TYPE: &str = "isize";
