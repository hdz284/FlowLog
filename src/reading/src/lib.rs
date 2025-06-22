pub mod reader; 
pub mod row;
pub mod rel;
pub mod inspect;
pub mod arrangements;
pub mod session;

// Re-export FALLBACK_ARITY from planning
pub use planning::FALLBACK_ARITY;

use differential_dataflow::difference::Present;

pub type Time = (); 
pub type Iter = u16;
pub type Semiring = Present;




