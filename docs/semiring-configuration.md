# Semiring Configuration

This project supports configuring the semiring type used in differential dataflow at build time. You can choose between two semiring types:

1. **Present** (default) - Uses `differential_dataflow::difference::Present`
2. **isize** - Uses `isize` as the semiring type

## Usage

### Build with Present Semiring (default)
```bash
cargo build
# or explicitly
cargo build --features present-type
```

### Build with isize Semiring  
```bash
cargo build --features isize-type --no-default-features
```

### Run with isize Semiring
```bash
cargo run --features isize-type --no-default-features
```

### Run tests with isize Semiring
```bash
cargo test --features isize-type --no-default-features
```

## Features

- `present-type` (default): Uses `Present` as the difference type
- `isize-type`: Uses `isize` as the difference type

These features are mutually exclusive - you should only enable one at a time.

## Implementation

The semiring type is configured in `src/reading/src/lib.rs` using conditional compilation:

```rust
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
pub type Semiring = Present;

#[cfg(all(feature = "isize-type", not(feature = "present-type")))]
pub type Semiring = isize;

// Helper function to create the semiring identity element
#[cfg(all(feature = "present-type", not(feature = "isize-type")))]
pub fn semiring_one() -> Semiring {
    Present {}
}

#[cfg(all(feature = "isize-type", not(feature = "present-type")))]
pub fn semiring_one() -> Semiring {
    1
}
```

This type is then re-exported and used throughout the reading and executing crates for differential dataflow operations.

## Feature Propagation

The features are properly propagated through the dependency chain:
- `executing` crate → `reading` crate + `macros` crate
- `macros` crate → `reading` crate

When you specify `--features isize-type --no-default-features` on the workspace, it will:
1. Disable default features on all crates
2. Enable the `isize-type` feature on the top-level crates
3. Propagate this feature to their dependencies
