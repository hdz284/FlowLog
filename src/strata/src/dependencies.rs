use std::collections::{HashMap, HashSet};
use parsing::parser::Program;
use parsing::rule::Predicate;
use itertools::Itertools;

use std::fmt;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    rule_idb_names: Vec<String>,
    rule_dependency_map: HashMap<usize, HashSet<usize>>,
    negation_dependency_map: HashMap<usize, HashSet<usize>>,
}

impl DependencyGraph {
    pub fn rule_idb_names(&self) -> &Vec<String> {
        &self.rule_idb_names
    }

    pub fn rule_dependency_map(&self) -> &HashMap<usize, HashSet<usize>> {
        &self.rule_dependency_map
    }

    pub fn negation_dependency_map(&self) -> &HashMap<usize, HashSet<usize>> {
        &self.negation_dependency_map
    }

    /* main constructor */
    pub fn from_parser(program: &Program) -> Self {
        let rules = program.rules();
        let rule_idb_names = rules.iter().map(|rule| rule.head().name().clone()).collect::<Vec<String>>();
        
        // println!(".depgraph rule_idb_names = {:?}", rule_idb_names);

        /* head2rule_ids_map maps head_name to rule_ids of that head */ 
        let mut head2rule_ids_map = HashMap::new();
        for (rule_id, rule) in rules.iter().enumerate() {
            let head_name = rule.head().name();
            let rule_ids = head2rule_ids_map
                .entry(String::from(head_name))
                .or_insert(Vec::new()); // or_insert() returns a mutable reference to the value
            rule_ids.push(rule_id);
        }

        let mut rule_dependency_map: HashMap<usize, HashSet<usize>> = (0..rules.len())
                                                .map(|i| (i, HashSet::new()))
                                                .collect();
        let mut negation_dependency_map: HashMap<usize, HashSet<usize>> = (0..rules.len())
                                                .map(|i| (i, HashSet::new()))
                                                .collect();


        for (rule_id, rule) in rules.iter().enumerate() {
            for predicate in rule.rhs() {
                let atom_name = match predicate {
                    // S :- ...
                    // T :- ... S ...
                    // T depends on S
                    Predicate::AtomPredicate(atom) => atom.name(),
                    
                    // S :- ...
                    // T :- ... !S ...
                    // T (the next strata) depends on S
                    Predicate::NegatedAtomPredicate(atom) => {
                        if let Some(atom_as_head_rule_ids) = head2rule_ids_map.get(atom.name()) {
                            negation_dependency_map.get_mut(&rule_id).unwrap().extend(atom_as_head_rule_ids.iter().copied());
                        }
                        atom.name()
                    },
                    _ => continue, /* skip comparison op */
                };

                if let Some(atom_as_head_rule_ids) = head2rule_ids_map.get(atom_name) {
                    rule_dependency_map.get_mut(&rule_id).unwrap().extend(atom_as_head_rule_ids.iter().copied()); // rule_id depends on as_head_rule_id （extends() adds all elements to the HashSet）
                }
            }
        }

        Self {
            rule_idb_names,
            rule_dependency_map,
            negation_dependency_map,
        }
    }
}



impl fmt::Display for DependencyGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dependency_graph_str = String::new();

        // formatting the Dependency Graph
        write!(
            dependency_graph_str,
            ".dependency graph (rule_id: dependent rule_ids): \n"
        )
        .unwrap();

        for (rule_id, dependent_rule_ids) in
            self.rule_dependency_map.iter().sorted_by_key(|x| x.0)
        {
            if !dependent_rule_ids.is_empty() {
                let dependent_rule_ids_str = dependent_rule_ids.iter()
                                            .sorted()
                                            .map(ToString::to_string)
                                            .collect::<Vec<_>>()
                                            .join(", ");
                write!(dependency_graph_str, "{}: [{}]\n", rule_id, dependent_rule_ids_str)?; // : here is equivalent to (depends on)
            } else {
                write!(dependency_graph_str, "{}: \n", rule_id).unwrap();
            }
        }

        // formatting the Negation Dependency Graph
        write!(
            dependency_graph_str,
            "\n.negation dependency graph (rule_id: dependent negation rule_ids): \n"
        )
        .unwrap();
        for (rule_id, dependent_rule_ids) in
            self.negation_dependency_map.iter().sorted_by_key(|x| x.0)
        {
            
            if !dependent_rule_ids.is_empty() {
                let dependent_rule_ids_str = dependent_rule_ids.iter()
                    .sorted()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(dependency_graph_str, "{}: [{}]\n", rule_id, dependent_rule_ids_str)?;
            }
        }

        write!(f, "{}", dependency_graph_str) 
    }
}
