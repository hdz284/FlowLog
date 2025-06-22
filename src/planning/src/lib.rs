// lib.rs :: the entry point for the planning crate
pub mod collections;
pub mod arguments;
pub mod transformations;
pub mod constraints;
pub mod flow;
pub mod arithmetic; 
pub mod compare;

pub mod program;
pub mod strata;
pub mod rule;

/// Maximum arity that can be handled by fixed-size arrays (using Row<N>)
/// Beyond this arity, we fall back to FatRow which uses SmallVec
pub const FALLBACK_ARITY: usize = 8;

