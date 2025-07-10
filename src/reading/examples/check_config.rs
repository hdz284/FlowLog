// Test to verify semiring types and configuration parameters
// cd /users/hangdong/FlowLog/src/reading && cargo run --example check_config --quiet
// cargo run --example check_config --features isize-type --no-default-features --quiet
use reading::{SEMIRING_TYPE, FALLBACK_ARITY, KV_MAX, ROW_MAX, PROD_MAX};

fn main() {
    println!("=== FlowLog Configuration Check ===\n");
    
    // Semiring configuration
    println!("--- Semiring Configuration ---");
    println!("Current semiring type: {}", SEMIRING_TYPE);
    
    let semiring_value = reading::semiring_one();
    println!("ONE: {:?}", semiring_value);
    println!("SIZE of semiring: {} bytes", std::mem::size_of_val(&semiring_value));
    
    // Conditional compilation features
    #[cfg(feature = "present-type")]
    println!("Feature enabled: present-type");
    #[cfg(feature = "isize-type")]
    println!("Feature enabled: isize-type");
    
    println!();
    
    // Configuration parameters
    println!("--- Configuration Parameters ---");
    println!("FALLBACK_ARITY: {} (max arity before forcing fat mode)", FALLBACK_ARITY);
    println!("KV_MAX: {} (max arity for key-value operations)", KV_MAX);
    println!("ROW_MAX: {} (max arity for row operations)", ROW_MAX);
    println!("PROD_MAX: {} (max arity for product operations)", PROD_MAX);
}
