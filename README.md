# FlowLog

<p align="center"> <img src="flowlog.png" alt="flowlog_logo" width="250"/> </p>

This repository contains the implementation for the paper **"FlowLog: Efficient and Extensible Datalog via Incrementality"**.

FlowLog is an efficient, scalable and extensible Datalog engine built atop Differential Dataflow.

## System Architecture

FlowLog uses a modular architecture that collectively form a Datalog execution pipeline as follows (see paper Figure 1):

```
├── parsing       # Parsing Datalog program
├── strata        # Stratification
├── planning      # Generate logical IR and optimize (per rule)
  ├── catalog       # Generate metadata 
  └── optimizing    # Query optimization 
└── executing     # Executor
  ├── reading       # Reading data from CSV
  └── macros        # Rust macros for code generate each differential operator
```


## A Quick Example (TODO!)

Here's a simple Datalog program that computes the transitive closure of a binary relation: // also write down the dependencies! Cargo/Rust/timely/DD versions

```prolog
# Program: examples/reach.dl
relation edge(a, b).
relation edge(b, c).
relation edge(c, d).

reach(X, Y) :- edge(X, Y).
reach(X, Y) :- edge(X, Z), reach(Z, Y).
```

To run this program, place it in a file called `reach.dl` and create a directory called `reach` containing the input fact files (EDBs).

## Building

```bash
# Release build
cargo build --release                                             # Batch mode (Present, default)
cargo build --release --features isize-type --no-default-features # Incremental mode (isize)

# Debug build
cargo build                                                       # Batch mode (Present, default)
cargo build --features isize-type --no-default-features           # Incremental mode (isize)
```

### Execution Modes

FlowLog currently supports two execution modes for Datalog applications:

- **Batch Mode** (default): Uses `differential_dataflow::difference::Present` for static Datalog semantics. This mode only tracks whether facts are present or absent, making it suitable for high-performance static Datalog execution.
- **Incremental Mode**: Uses `isize` as the `diff` type for DD's incremental semantics. This allows tracking how many times each fact is derived, supporting incremental view maintenance for Datalog programs.

#### Build Options

| Execution Mode | Build Command | Use Case |
|----------------|---------------|----------|
| **Batch Mode** (default) | `cargo build --release` | Static Datalog execution (used in the paper benchmarking) |
| **Incremental Mode** | `cargo build --release --features isize-type --no-default-features` | Incremental Datalog execution |


## Usage

After (release) build, use the `executing` binary to run Datalog programs:

```bash
# Basic usage
target/release/executing -p <program.dl> -f <facts_directory> -w <number_threads>

# Example with concrete paths
target/release/executing -p examples/reach.dl -f reach -w 8
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
  <td>Directory containing input fact files (EDBs)</td>
</tr>
<tr>
  <td align="center"><code>-c, --csvs &lt;DIR&gt;</code></td>
  <td><strong>Optional:</strong> Directory for emitting output results (IDBs). If not set, only print IDB sizes in terminal.</td>
</tr>
<tr>
  <td align="center"><code>-d, --delimiter &lt;CHAR&gt;</code></td>
  <td>Delimiter for input files (default: <code>,</code>)</td>
</tr>
<tr>
  <td align="center"><code>-w, --workers &lt;NUM&gt;</code></td>
  <td>Number of worker threads (default: 1)</td>
</tr>
<tr>
  <td align="center"><code>-O &lt;LEVEL&gt;</code></td>
  <td>Optimization level (0-3): <br>
  <code>0</code> - No optimization <br>
  <code>1</code> - Sideways Information Passing (SIP) <br>
  <code>2</code> - Structural Planning <br>
  <code>3</code> - Both optimizations (SIP + Planning)</td>
</tr>
</table>

#### Example Commands

```bash
# Basic execution under default settings
target/release/executing -p examples/reach.dl -f reach

# Multi-threaded (16 threads) execution, flushing IDBs to output/
target/release/executing -p examples/tc.dl -f tc -c output -w 16

# Robust execution using both SIP and Planning
target/release/executing -p examples/batik.dl -f batik -d $'\t' -w 32 -O 3

# Debug print RUST_LOG=debug
RUST_LOG=debug target/release/executing -p examples/batik.dl -f batik -c results -O 2
```

###  Datasets

All datasets used in the paper evaluation are publicly available:

**Paper Datasets**: https://pages.cs.wisc.edu/~m0riarty/dataset/

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

###  Current Limitations

- FlowLog currently supports `count`, `sum`, `min`, `max` aggregation operators. However, the aggregate field must be the **last argument** in the head IDB. All rules deriving the same IDB must conform to the same **aggregation type** (e.g. `count`, `sum`).

- (Compilation....) FlowLog currently compiles very slowly since it has heavy dependencies on DD (e.g. on r6525, it takes 16 minutes to compile)

## Example Datalog Programs

The `examples` directory contains several sample Datalog programs.

<!-- ## Testing

To run all bundled correctness tests:

```bash
bash env_test.sh
```
This script will automatically:
1. Download and extract the test dataset and programs
2. Run each test program with its corresponding input
3. Verify output files against expected results

You should see PASSED for each program if everything is correct. -->


## Reproducing Paper Figures

This repository includes [Datalog-DB-benchmark](https://github.com/HarukiMoriarty/Datalog-DB-benchmark) as a git submodule. You can use this submodule to reproduce the experiment figures from the paper. Please initialize submodules after cloning:

```bash
git submodule update --init --recursive
```

## Contributing

Contributions are welcome! Feel free to submit a PR.



