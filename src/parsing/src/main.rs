use parsing::parser::Lexeme;
use parsing::{FlowLogParser, Parser, Rule};
use std::fs;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let program_source = "./examples/programs/greater_equal.dl";
    let unparsed_str = fs::read_to_string(program_source)
        .unwrap_or_else(|_| panic!("can't read program from \"{}\"", program_source));

    let parsed_rule = FlowLogParser::parse(Rule::main_grammar, &unparsed_str)
        .unwrap_or_else(|error| {
            panic!(
                "can't parse program from \"{}\": \n{:?}",
                program_source, error
            )
        })
        .next()
        .unwrap();

    // .next() returns the first Pair in the iterator or None if there are no more Pairs
    // Pairs :: Vec<Pair> | Pair :: a matching pair of tokens from the input (https://docs.rs/pest/2.1.3/pest/iterators/struct.Pair.html)

    // print_rule(parsed_rule, 0); // print the parsed rule
    // print_rule_as_tree(parsed_rule, 0, true); // print the parsed rule as a tree

    debug!(
        "{}",
        parsing::parser::Program::from_parsed_rule(parsed_rule)
    );

    info!("success parse");
}

// fn print_rule(rule: pest::iterators::Pair<Rule>, depth: usize) {
//     let indent = " ".repeat(depth * 2);
//     let rule_name = rule.as_rule();              // returns the rule that matched the input
//     let rule_span = rule.as_span();              // returns a span of the input string
//     let rule_str = rule_span.as_str();           // returns a string slice of the input string

//     debug!("{}{:?} >> {}", indent, rule_name, rule_str);

//     rule.into_inner()
//         .for_each(|rule| print_rule(rule, depth + 1));
// }

// fn print_rule_as_tree(rule: pest::iterators::Pair<Rule>, depth: usize, is_last: bool) {
//     let indent = if depth == 0 {
//         String::new()
//     } else if is_last {
//         "   ".repeat(depth - 1) + "└──"
//     } else {
//         "   ".repeat(depth - 1) + "├──"
//     };

//     let rule_name = format!("{:?}", rule.as_rule());
//     let rule_str = rule.as_span().as_str();

//     // print the rule with tree structure
//     if rule_str.len() > 240 {
//         debug!("{}{} >> ({:.240}...)", indent, rule_name, rule_str);
//     } else {
//         debug!("{}{} >> ({})", indent, rule_name, rule_str);
//     }

//     // transform into inner pairs and print them as part of the tree
//     let inner_rules: Vec<_> = rule.into_inner().collect();
//     let len = inner_rules.len();
//     for (i, inner_rule) in inner_rules.into_iter().enumerate() {
//         let is_last = i == len - 1;
//         print_rule_as_tree(inner_rule, depth + 1, is_last);
//     }
// }
