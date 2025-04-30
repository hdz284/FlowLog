use std::collections::{HashMap, HashSet};
use parsing::parser::Program;
use parsing::rule::FLRule;
use crate::dependencies::DependencyGraph;
use itertools::Itertools;

use std::fmt;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Strata {
    fl_program: Program,
    dependency_graph: DependencyGraph,
    // sccs: HashMap<usize, Vec<usize>>,       /* sccs of rules maps scc_id to rule_ids in that scc (scc_id is the root rule_id of the scc) */
    // sccs_order: Vec<usize>,                 /* topological order of sccs is the order of evaluation  */
    strata: Vec<Vec<usize>>,                /* strata are the rules to be evaluated at once */
    is_recursive_strata_bitmap: Vec<bool>,  /* if each stratum is recursive */
}

impl Strata {
    pub fn program(&self) -> &Program {
        &self.fl_program
    }

    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    pub fn transpose_graph_from(
        rule_dependency_map: &HashMap<usize, HashSet<usize>>,
    ) -> HashMap<usize, HashSet<usize>> {
        let mut transpose_dependency_graph = HashMap::with_capacity(rule_dependency_map.len());
        
        for (&rule_id, dependent_rules_ids) in rule_dependency_map.iter() {
            for &dependent_rule_id in dependent_rules_ids {
                transpose_dependency_graph
                    .entry(dependent_rule_id)
                    .or_insert_with(HashSet::new)
                    .insert(rule_id);
            }
        }
        
        transpose_dependency_graph
    }

    pub fn processing_order_dfs(
        order: &mut Vec<usize>,
        visited: &mut Vec<bool>,
        rule_dependency_map: &HashMap<usize, HashSet<usize>>,
        rule_id: usize, 
    ) {
        if !visited[rule_id] {
            visited[rule_id] = true;
            
            if let Some(dependent_rules_ids) = rule_dependency_map.get(&rule_id) {
                for &dependent_rule_id in dependent_rules_ids {
                    Self::processing_order_dfs(
                        order,
                        visited,
                        rule_dependency_map,
                        dependent_rule_id,
                    );
                }
            }
    
            order.push(rule_id);
        }
    }
    
    pub fn assigning_scc_dfs(
        transpose_dependency_graph: &HashMap<usize, HashSet<usize>>,
        rule_sccs: &mut HashMap<usize, Vec<usize>>, // scc_id -> rule_ids in that scc
        sccs_order: &mut Vec<usize>,
        rule_assigned: &mut Vec<bool>,
        rule_id: usize,
        scc_id: usize,
    ) {
        if rule_assigned[rule_id] {
            return;
        }
    
        rule_assigned[rule_id] = true;
    
        let scc = rule_sccs
            .entry(scc_id)
            .or_insert_with(|| {
                // if no such scc, create a new one
                sccs_order.push(scc_id);
                Vec::new()
            });
        
        scc.push(rule_id); // assign the rule_id to the scc_id
    
        if let Some(reverse_dependent_rules) = transpose_dependency_graph.get(&rule_id) {
            for &reverse_dependent_rule_id in reverse_dependent_rules {
                Self::assigning_scc_dfs(
                    transpose_dependency_graph,
                    rule_sccs,
                    sccs_order,
                    rule_assigned,
                    reverse_dependent_rule_id,
                    scc_id,
                );
            }
        }
    }
    
    /* main entry */
    pub fn from_parser(program: Program) -> Self {
        // Kosaraju's: find sccs (https://www.youtube.com/watch?v=QlGuaHT1lzA)
        let dependency_graph = DependencyGraph::from_parser(&program);
        let rule_dependency_map = &dependency_graph.rule_dependency_map(); // e.g. rule_dependency_map: {0: {}, 1: {1, 0}, 2: {1, 0}}
         
        // dfs to determine the order of processing
        let mut rule_visited = vec![false; rule_dependency_map.len()];
        let mut processing_order = Vec::new();
        
        for &rule_id in rule_dependency_map.keys() {
            Self::processing_order_dfs(&mut processing_order, &mut rule_visited, &rule_dependency_map, rule_id);
        }
        processing_order.reverse();

        // dfs to assign sccs on the reversed dependency graph
        let transpose_dependency_graph = Self::transpose_graph_from(&rule_dependency_map);
        let mut rule_sccs = HashMap::new();
        let mut sccs_order = Vec::new();
        let mut rule_assigned = vec![false; processing_order.len()];
        for rule_id in processing_order {
            Self::assigning_scc_dfs(
                &transpose_dependency_graph,
                &mut rule_sccs,
                &mut sccs_order,
                &mut rule_assigned,
                rule_id,
                rule_id,
            );
        } // end of Kosaraju's algorithm over the rule_dependency_map

        // layout the evaluation order for the sccs (the topological order of the reversed dependency graph)
        sccs_order.reverse(); 

        // construct (initial) strata and recursive bitmap
        let mut strata = Vec::new();
        let mut is_recursive_strata_bitmap = Vec::new();
        for &scc_id in &sccs_order {
            if let Some(scc) = rule_sccs.get(&scc_id) {
                strata.push(scc.clone());
                is_recursive_strata_bitmap.push(scc.len() > 1 || 
                    dependency_graph.rule_dependency_map()
                        .get(&scc_id)
                        .map_or(false, |deps| deps.contains(&scc_id)));
            }
        }

        // --------------------------------------------------------------------------- //
        // merge independent strata
        let mut strata_dependencies: Vec<HashSet<usize>> = strata
            .iter()
            .map(|strata| {
                strata.iter()
                      .filter_map(|rule_id| rule_dependency_map.get(rule_id))
                      .flat_map(|deps| deps.iter().copied())
                      // exclude depend rules from the (recursive) strata itself 
                      .filter(|&dep_id| !strata.contains(&dep_id)) 
                      .collect()
            })
            .collect();
        // println!("strata_dependencies: {:?}", strata_dependencies);

        let mut merged = vec![false; strata.len()];
        let mut mergers = Vec::new();
        let mut is_recursive_merger_bitmap = Vec::new();

        while merged.iter().any(|&merged| !merged) { // while there are not merged strata
            let mut next_non_recursive: Vec<usize> = Vec::new();
            let mut next_recursive: Vec<Vec<usize>> = Vec::new();
            for (i, s) in strata.iter().enumerate() {
                if !merged[i] && strata_dependencies[i].is_empty() { // not yet merged and no dependencies
                    merged[i] = true;
                    // println!("merging stratum: {:?}", s);
                    if is_recursive_strata_bitmap[i] {
                        next_recursive.push(s.clone()); // batch non-recursive strata
                    } else {
                        next_non_recursive.extend(s.iter().copied());
                    }
                }
            }

            // remove dependencies on rules of the merged strata
            for dependencies in strata_dependencies.iter_mut() {
                dependencies.retain(|&rule_id| 
                    !next_non_recursive.contains(&rule_id) &&
                    !next_recursive.iter().any(|stratum| stratum.contains(&rule_id))
                );
            }

            // println!("next non-recursive strata: {:?}", next_non_recursive);
            // println!("next recursive strata: {:?}", next_recursive);
            
            if !next_non_recursive.is_empty() { 
                mergers.push(next_non_recursive); 
                is_recursive_merger_bitmap.push(false);
            }
            
            for s in next_recursive { 
                mergers.push(s); 
                is_recursive_merger_bitmap.push(true);
            }
        }

        // println!("merged strata: {:?}", mergers);
        // --------------------------------------------------------------------------- //
            
        Self {
            fl_program: program,
            dependency_graph,
            // sccs: rule_sccs,
            // sccs_order,
            strata: mergers, // strata,
            is_recursive_strata_bitmap: is_recursive_merger_bitmap, // is_recursive_strata_bitmap,
        }
    }
    
    /* fetch the strata */
    pub fn strata(&self) -> Vec<Vec<&FLRule>> {
        let mut strata = Vec::with_capacity(self.strata.len()); 
    
        for stratum_ids in &self.strata {
            let stratum = stratum_ids.iter()
                .map(|&rule_id| &self.fl_program.rules()[rule_id])
                .collect();
    
            strata.push(stratum);
        }
    
        strata
    }

    /* check the stratum recursive bitmap */
    pub fn is_recursive_stratum(&self, stratum_id: usize) -> bool {
        self.is_recursive_strata_bitmap[stratum_id]
    }

    pub fn is_recursive_strata_bitmap(&self) -> &Vec<bool> {
        &self.is_recursive_strata_bitmap
    }
}


impl fmt::Display for Strata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut strata_str = String::new();

        /* printing scc */
        // write!(strata_str, "Topological order of rule sccs:\n## scc_id: [rule_ids in that scc]\n").unwrap();
        // for scc_id in &self.sccs_order {
        //     let scc = self.sccs.get(scc_id).unwrap();
        //     let scc_rule_id_strs: Vec<String> = scc.iter().sorted().map(|r| r.to_string()).collect();
        //     let scc_rule_ids_str = scc_rule_id_strs.join(", ");
        //     write!(strata_str, "{}: [{}]\n", scc_id, scc_rule_ids_str).unwrap();
        // }

        for (stratum_id, stratum) in self.strata.iter().enumerate() {
            // display strata number
            write!(strata_str, "#{}: ", stratum_id + 1)?;

            let rule_id_strs: Vec<String> = stratum
                .iter()
                .sorted()
                .map(|rule_id| rule_id.to_string())
                .collect();

            write!(strata_str, "[{}]\n", rule_id_strs.join(", ")).unwrap();
            for rule_id in stratum {
                write!(
                    strata_str,
                    "{}\n",
                    self.fl_program.rules()[*rule_id]
                )
                .unwrap();
            }

            write!(strata_str, "\n").unwrap();
        }

        write!(f, "{}", strata_str)
    }
}
