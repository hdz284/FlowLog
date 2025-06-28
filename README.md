# FlowLog

<p align="center"> <img src="flowlog.png" alt="flowlog_logo" width="250"/> </p>

FlowLog is an efficient, scalable and extensible Datalog engine built atop Differential Dataflow.

## Project Structure

```
├── catalog       # Program metadata representation
├── debugging     # Debugging utilities
├── executing     # Runtime execution engine
├── macros        # Rust macros
├── optimizing    # Query optimization
├── parsing       # Parsing datalog language
├── planning      # Query planning
├── reading       # File and data input components
├── strata        # Stratification logic
└── examples      # Example programs and datasets
```

## Installation

### Prerequisites
- Rust and Cargo (latest stable version recommended)
- Differential Dataflow (0.14.2)

### Required Dependency Modification

Before building, you need to modify the differential dataflow crate (version 0.13.7) by adding the following function to `differential_dataflow::collection.rs` at line 329 (after the `explode` function):

```rust
/// (udf) Brute-force replaces each record with another w/ a new difference type.
///
pub fn expand<D2, R2, I, L>(&self, mut logic: L) -> Collection<G, D2, R2>
where
    D2: Data,
    R2: Semigroup + 'static,
    I: IntoIterator<Item = (D2, R2)>,
    L: FnMut(D) -> I + 'static,
{
    self.inner
        .flat_map(move |(x, t, _)| logic(x).into_iter().map(move |(x, d2)| (x, t.clone(), d2)))
        .as_collection()
}
```

### Building
```bash
# Default build (Present semiring)
cargo build --release

# Build with isize semiring for incremental semantics
cargo build --release --features isize-type --no-default-features

# Debug builds
cargo build                                           # Present semiring (default)
cargo build --features isize-type --no-default-features  # isize semiring
```

### Semiring Configuration

FlowLog supports two semiring types for differential dataflow computations:

- **Present** (default): Uses `differential_dataflow::difference::Present` for standard semantics
- **isize**: Uses `isize` as the semiring type to enable incremental semantics

#### Build Options

| Configuration | Command | Use Case |
|--------------|---------|----------|
| Present (default) | `cargo build --release` | Standard usage, backwards compatible |
| isize | `cargo build --release --features isize-type --no-default-features` | Incremental semantics, multiplicities |


## Command Line

- `./src`  
  - `./src/parsing` - the parsing crate
     
     ```bash
     cargo build # build the parsing crate
     ```
     
     run the binary (i.e., `./src/parsing/src/main.rs`) of built parsing crate
     ```bash
     cargo run -p parsing
     ```
  - `./src/executing` - end to end execution
      ```bash
      # Build with default Present semiring
      cargo build --release
      
      # Build with isize semiring for incremental semantics
      cargo build --release --features isize-type --no-default-features
      
      # Run on 64 threads for batik.dl program
      ./target/release/executing -p ./examples/programs/batik.dl -f ./examples/csvs -c ./examples/csvs -d $'\t' -w 64 
      ```

## Usage

### Command Options

<table>
<tr>
  <th align="center">Option</th>
  <th align="center">Description</th>
</tr>
<tr>
  <td align="center"><code>-p, --program &lt;FILE&gt;</code></td>
  <td>Path to the Datalog program file (<code>.dl</code> extension)</td>
</tr>
<tr>
  <td align="center"><code>-f, --facts &lt;DIR&gt;</code></td>
  <td>Path containing input facts</td>
</tr>
<tr>
  <td align="center"><code>-c, --csvs &lt;DIR&gt;</code></td>
  <td>Path for output results</td>
</tr>
<tr>
  <td align="center"><code>-d, --delimiter &lt;CHAR&gt;</code></td>
  <td>Delimiter for input files (default: <code>,</code>)</td>
</tr>
<tr>
  <td align="center"><code>-w, --workers &lt;NUM&gt;</code></td>
  <td>Number of threads (default: single core)</td>
</tr>
<tr>
  <td align="center"><code>-v, --verbose</code></td>
  <td>Enable verbose logging</td>
</tr>
<!-- <tr>
  <td align="center"><code>-h, --help</code></td>
  <td>Print help information</td>
</tr> -->
</table>

#### Example Commands

```bash
# Run a program with default settings (Present semiring)
./target/release/executing -p ./examples/programs/reach.dl -f ./examples/facts

# Run with isize semiring for incremental semantics
./target/release/executing -p ./examples/programs/reach.dl -f ./examples/facts

# Run on 16 threads and tab as delimiter
./target/release/executing -p ./examples/programs/tc.dl -f ./examples/csvs -d $'\t' -w 16

# Run on verbose output and custom output directory
./target/release/executing -p ./examples/programs/batik.dl -f ./examples/csvs -c ./results -v
```

**Note**: To use the isize semiring version for incremental semantics, build with:
```bash
cargo build --release --features isize-type --no-default-features
```

### Datalog Syntax

FlowLog supports standard Datalog with common extensions:

```datalog
// Simple graph reach
reach(x) :- source(x).
reach(y) :- reach(x), edge(x, y).

// constraints
two_hops(x, z) :- edge(x, y), edge(y, z), x != z.

// negation
indirect_only(x, z) :- edge(x, y), edge(y, z), !edge(x, z).

// aggregation
count_paths(x, z, COUNT(y)) :- edge(x, y), edge(y, z).
```


## Examples

The `examples/` directory contains several sample Datalog programs:

- `examples/programs/batik.dl`: DOOP program for batik
- `examples/programs/`: Other sample programs tested

## Testing

To run all bundled correctness tests:

```bash
bash env_test.sh
```
This script will automatically:
1. Download and extract the test dataset and programs
2. Run each test program with its corresponding input
3. Verify output files against expected results

You should see ✅ PASSED for each program if everything is correct.

## Performance

FlowLog supports two semiring configurations:
- **Present semiring** (default): Standard Datalog carrying set semantics
- **isize semiring**: Incremental semantics via multiplicities (slower but supports richer semantics)


## Contributing

Contributions are welcome! Feel free to submit a PR.



