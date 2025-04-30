# FlowLog

<p align="center"> <img src="flowlog.png" alt="flowlog_logo" width="250"/> </p>


## Command Line

- `./src`  
  - `./src/parsing` - the parsing crate
     
     ```bash
     cargo build // build the parsing crate
     ```
     
     run the binary (i.e., `./src/parsing/src/main.rs`) of built parsing crate
     ```bash
     cargo run -p parsing
     ```
  - `./src/executing` - end to end execution
      ```bash
      cargo build --release
      ./target/release/executing -p ./examples/programs/batik.dl -f ./examples/csvs -c ./examples/csvs -d $'\t' -w 64 // 64 threads execuition of batik.dl
      ```






<!-- ## To Dos

#### more supports

- filtering (cross-joins)

  ```
  sg(x, y) :- arc(p, x), arc(p, y), x != y.
  ```

- aliasing

  ```
  valueFlow(y, x) :- assign(x, x), pointsto(x, y). // in the rhs
  valueFlow(x, x) :- assign(x, y). // in the head
  ```

- constant equality constraints

  ```
  sg(x, y) :- arc(p, x), arc(p, y), x = 3.
  ``` -->

  
<!-- 
#### doop setups

  ```
  python3 ../monoid/doop/convert.py ../doop/last-analysis .
  python3 ../monoid/doop/replace.py ../monoid/doop/Literal.facts ../FlowLogTest/examples/programs/doop-souffle.dl batik-souffle.dl
  python3 ../monoid/doop/replace.py ../monoid/doop/Literal.facts ../FlowLogTest/examples/programs/doop.dl batik.dl

  scp ./batik.zip hangdong@royal-05.cs.wisc.edu:/home/hangdong/public/html/data/
  ```



run souffle
  ```
  souffle -o batik-souffle ./batik-souffle.dl -j 16
  // -p log if want profiler
  time ./batik-souffle -F/users/hangdong/batik -j 16
  souffleprof log -j // then download html
  ```

run eclair
  ```
  ./target/release/executing -p /users/hangdong/batik/batik.dl -f /users/hangdong/batik -c ./examples/csvs -d $'\t' -w 64
  ``` -->


## To Do List

Planned features and optimizations for the next versions of FlowLog.

### Compile-time
- Parallel Compilation
      [Cutting down Rust compile times](https://www.feldera.com/blog/cutting-down-rust-compile-times-from-30-to-2-minutes-with-one-thousand-crates)
- Macro fallback arity
      Clamp macro-generated arities to a fixed max (e.g., 10) to reduce codegen overhead


### Rule Support

#### Fact Rules
- Constant rules like:  
  `T(a) :- true or false.`

#### Boolean Rules
- Zero-arity heads:  
  `T( ) :- R(x), S(x, y).`


### Sideways Information Passing (SIP)
- Integrate SIP into the planner  
      (currently via rule rewriting at catalog level)


### Query Optimization
- Refine the cost model
- Pessimistic cardinality estimators for join ordering and planning


### Arithmetics in Heads
- Expressions and constants in rule heads:  
  `T(x + z, 1) :- R(x, y), S(y, z).`


### Group-By Aggregations
- Translate group-by to `reduce_core` in Differential Dataflow
- Test novel recursive aggregation optimizations Simon proposed
