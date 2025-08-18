# FlowLog

<p align="center"> <img src="flowlog.png" alt="flowlog_logo" width="250"/> </p>

This repository contains the implementation for the paper **"FlowLog: Efficient and Extensible Datalog via Incrementality"**.

FlowLog is an efficient, scalable and extensible Datalog engine built atop Differential Dataflow.

## Project Structure

FlowLog follows a modular architecture where each component handles a specific part of the Datalog execution pipeline. The structure reflects the execution order as shown in the system architecture (paper Figure 1):

```
├── parsing       # Parsing datalog language
├── strata        # Stratification logic
├── catalog       # Program metadata representation
├── optimizing    # Query optimization
├── planning      # Query planning
├── reading       # File and data input components
├── executing     # Runtime execution engine
├── macros        # Rust macros
├── debugging     # Debugging utilities
└── examples      # Example programs and datasets
```

## Building

```bash
# Release build
cargo build --release                                             # PRESENT semiring (default)
cargo build --release --features isize-type --no-default-features # ISIZE semiring

# Debug build
cargo build                                                       # PRESENT semiring (default)
cargo build --features isize-type --no-default-features           # ISIZE semiring
```

### Semiring Configuration

FlowLog supports two semiring types for differential dataflow computations:

- **Present** (default): Uses `differential_dataflow::difference::Present` for standard Datalog semantics. This semiring only tracks whether facts are present or absent, making it suitable for traditional Datalog evaluation.
- **isize**: Uses `isize` as the semiring type to enable incremental semantics with multiplicities. This allows tracking how many times each fact is derived, enabling more sophisticated incremental computation and debugging capabilities.

#### Build Options

| Semiring Type | Build Command | Use Case |
|---------------|---------------|----------|
| **Present** (default) | `cargo build --release` | Traditional Datalog evaluation, production use, better performance |
| **isize** | `cargo build --release --features isize-type --no-default-features` | Advanced incremental computation, debugging derivations, tracking multiplicities |


## Usage

After building, use the `executing` binary to run Datalog programs:

```bash
# Basic usage
./target/release/executing -p <program.dl> -f <facts_directory> -w <number_threads>

# Example with concrete paths
./target/release/executing -p ./examples/programs/reach.dl -f ./examples/facts -w 8
```

## Command Options

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

# Run on debug output and custom output directory
RUST_LOG=debug ./target/release/executing -p ./examples/programs/batik.dl -f ./examples/csvs -c ./results
```

###  Datasets

All datasets used in the paper evaluation are available for download:

**📊 Paper Datasets**: https://pages.cs.wisc.edu/~m0riarty/dataset/

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
count_paths(x, z, count(y)) :- edge(x, y), edge(y, z).
max_salary(dept, max(salary)) :- employee(emp_id, salary), works_in(emp_id, dept).
```

Notes:

1. Aggregation: FlowLog supports `count`, `sum`, `min`, `max` aggregation operators. Aggregation must be the **last argument** in the head predicate. All rules for the same predicate must use the **same aggregation type**


## Examples

The `examples/` directory contains several sample Datalog programs:

- `examples/programs/batik.dl`: DOOP program for batik
- `examples/programs/`: Other sample programs tested -->

<!-- ## Testing

To run all bundled correctness tests:

```bash
bash env_test.sh
```
This script will automatically:
1. Download and extract the test dataset and programs
2. Run each test program with its corresponding input
3. Verify output files against expected results

You should see ✅ PASSED for each program if everything is correct. -->


## Reproducing Paper Figures

This repository includes the [Datalog-DB-benchmark](https://github.com/HarukiMoriarty/Datalog-DB-benchmark) as a git submodule. You can use this submodule to reproduce the experiment figures from the paper. Please initialize submodules after cloning:

```bash
git submodule update --init --recursive
```

## Contributing

Contributions are welcome! Feel free to submit a PR.



