pub mod arrangements;
pub mod config;
pub mod inspect;
pub mod reader;
pub mod rel;
pub mod row;
pub mod session;
pub mod semiring;

// export configuration constants for backwards compatibility
pub use config::{FALLBACK_ARITY, KV_MAX, PROD_MAX, ROW_MAX};

// export semiring types and functions for convenience
pub use semiring::{Semiring, semiring_one, SEMIRING_TYPE, Min};

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