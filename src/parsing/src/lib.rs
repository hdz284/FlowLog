// lib.rs :: the entry point for the parsing crate
pub mod decl;
pub mod head;
pub mod parser;
pub mod rule;
pub mod compare;
pub mod arithmetic;

extern crate pest; // import pest crate
#[macro_use]
extern crate pest_derive;
pub use pest::{iterators::Pair, Parser}; // Pair :: a pair of positions in a source string and the rule matching it

#[derive(Parser)] // attribute macro
#[grammar = "./grammar.pest"]
pub struct FlowLogParser;
