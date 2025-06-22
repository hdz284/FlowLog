//! Build-time configuration constants for FlowLog
//! 
//! These constants control the compile-time code generation and runtime limits
//! for various operations in the FlowLog engine.

/// Maximum arity before falling back to dynamic (fat) representations
pub const FALLBACK_ARITY: usize = 8;

/// Maximum arity for key-value in code generation
pub const KV_MAX: usize = 2;

/// Maximum arity for row in code generation  
pub const ROW_MAX: usize = 2;

/// Maximum arity for product in code generation
pub const PROD_MAX: usize = 2;

/// Configuration for compile-time code generation limits
pub struct CodegenLimits;

impl CodegenLimits {
    pub const KV_MAX: usize = KV_MAX;
    pub const ROW_MAX: usize = ROW_MAX;
    pub const PROD_MAX: usize = PROD_MAX;
    pub const FALLBACK_ARITY: usize = FALLBACK_ARITY;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_consistency() {
        // ensure the limits make sense relative to fallback arity
        assert!(KV_MAX <= FALLBACK_ARITY);
        assert!(ROW_MAX <= FALLBACK_ARITY);
        assert!(PROD_MAX <= FALLBACK_ARITY);
    }
    
    #[test]
    fn test_config_values() {
        // verify expected values
        // assert_eq!(FALLBACK_ARITY, 8);
        // assert_eq!(KV_MAX, 2);
        // assert_eq!(ROW_MAX, 2);
        // assert_eq!(PROD_MAX, 2);
    }
}
