//! Build-time configuration constants for FlowLog
//! 
//! These constants control the compile-time code generation and runtime limits
//! for various operations in the FlowLog engine.

/// Maximum arity for key-value in code generation
pub const KV_MAX: usize = 6;

/// Maximum arity for row in code generation  
pub const ROW_MAX: usize = 8;

/// Maximum arity for product in code generation
pub const PROD_MAX: usize = 2;

/// Maximum arity before falling back to fat representations
pub const FALLBACK_ARITY: usize = ROW_MAX;

/// Configuration for compile-time code generation limits
pub struct CodegenLimits;

impl CodegenLimits {
    pub const KV_MAX: usize = KV_MAX;
    pub const ROW_MAX: usize = ROW_MAX;
    pub const PROD_MAX: usize = PROD_MAX;
    pub const FALLBACK_ARITY: usize = FALLBACK_ARITY;
}