use std::fmt;
use std::sync::Arc;
// use itertools::Itertools;
use std::collections::HashSet;

// use parsing::rule::FLRule;
use strata::stratification::Strata;
use catalog::rule::Catalog;
use crate::collections::CollectionSignature;
use crate::rule::RuleQueryPlan;
use crate::strata::GroupStrataQueryPlan;


#[derive(Debug, Clone)]
pub struct ProgramQueryPlan {
    program_plan: Vec<GroupStrataQueryPlan>,
}

impl ProgramQueryPlan {
    pub fn program_plan(&self) -> &Vec<GroupStrataQueryPlan> {
        &self.program_plan
    }

    pub fn new(program_plan: Vec<GroupStrataQueryPlan>) -> Self {
        Self {
            program_plan,
        }
    }

    pub fn from_strata(strata: &Strata, disable_sharing: bool) -> Self {
        let rule_plans  = strata
            .strata()
            .into_iter()
            .zip(strata.is_recursive_strata_bitmap())
            .flat_map(|(stratum, is_recursive)| {
                let mut rule_identifier = 0;
                let chain = stratum
                    .iter()
                    .map(|&rule| {
                        let catalog = Catalog::from_strata(rule);
                        let expanded_catalogs = if rule.is_sip() { catalog.sideways(rule_identifier) } else { vec![catalog] }; // sideways information passing
                        rule_identifier += 1;
                        expanded_catalogs
                            .into_iter()
                            .map(|catalog| RuleQueryPlan::from_catalog(&catalog, rule.is_planning()))
                            .collect::<Vec<RuleQueryPlan>>()
                    })
                    .flatten();
                
                // if it is non_recursive and there is some rule annotated by .optimize()
                // (because sideways information passing slices the strata into many cascading strata) 
                if !*is_recursive && stratum.iter().any(|rule| rule.is_sip()) {
                    chain
                        .map(|plan| (false, vec![plan])) // (to do) this is a hacky way to make sideways works
                        .collect::<Vec<(bool, Vec<RuleQueryPlan>)>>()
                } else {
                    vec![(*is_recursive, chain.collect())] // one strata group
                }
            })
            .collect::<Vec<(bool, Vec<RuleQueryPlan>)>>();

        // debugging for each rule plan in the group
        for (is_recursive, rule_plans) in &rule_plans {
            println!("-------------------------------- {} strata group --------------------------------", if *is_recursive { "recursive" } else { "non-recursive" });
            for rule_plan in rule_plans { println!("{}", rule_plan); }
        }

        // accumulative seen seet across all strata
        let mut seen_set: HashSet<Arc<CollectionSignature>> = HashSet::new();
        let program_plan: Vec<GroupStrataQueryPlan> = rule_plans
            .into_iter()
            .map(|(is_recursive, rule_plans)| {
                GroupStrataQueryPlan::new(is_recursive, rule_plans, &mut seen_set, disable_sharing)
            })
            .collect();

        Self::new(program_plan)
    }

    pub fn max_arity(&self) -> usize {
        self.program_plan
            .iter()
            .flat_map(|group_plan| {
                group_plan.strata_plan().into_iter().flat_map(|transformation| {
                    let mut arities = Vec::new();
                    
                    // Get output collection arity
                    let (key_arity, value_arity) = transformation.output().arity();
                    arities.push(key_arity);
                    arities.push(value_arity);
                    
                    // Get input collection(s) arity
                    if transformation.is_unary() {
                        let (key_arity, value_arity) = transformation.unary().arity();
                        arities.push(key_arity);
                        arities.push(value_arity);
                    } else {
                        let (left, right) = transformation.binary();
                        let (left_key, left_value) = left.arity();
                        let (right_key, right_value) = right.arity();
                        arities.push(left_key);
                        arities.push(left_value);
                        arities.push(right_key);
                        arities.push(right_value);
                    }
                    
                    arities
                })
            })
            .max()
            .unwrap_or(0)
    }

    /// Returns a list of maximal (key_arity, value_arity) tuples that are incomparable.
    /// Two tuples (k1, v1) and (k2, v2) are incomparable if neither dominates the other.
    /// (k1, v1) dominates (k2, v2) if k1 >= k2 AND v1 >= v2, with at least one being a strict inequality.
    pub fn maximal_arity_pairs(&self) -> Vec<(usize, usize)> {
        // Collect all (key_arity, value_arity) pairs from the program
        let mut all_pairs = Vec::new();
        
        // Collect all pairs
        for group_plan in &self.program_plan {
            for transformation in group_plan.strata_plan() {
                // Get output collection arity
                all_pairs.push(transformation.output().arity());
                
                // Get input collection(s) arity
                if transformation.is_unary() {
                    all_pairs.push(transformation.unary().arity());
                } else {
                    let (left, right) = transformation.binary();
                    all_pairs.push(left.arity());
                    all_pairs.push(right.arity());
                }
            }
        }
        
        // Filter out non-maximal pairs and duplicates
        let mut maximal_pairs = Vec::new();
        
        for pair in &all_pairs {
            if maximal_pairs.contains(pair) {
                continue; // Skip duplicates
            }
            
            let (k1, v1) = *pair;
            let is_dominated = all_pairs.iter().any(|&(k2, v2)| 
                k2 >= k1 && v2 >= v1 && (k2 > k1 || v2 > v1)
            );
            
            if !is_dominated {
                maximal_pairs.push(*pair);
            }
        }
        
        maximal_pairs
    }

    /// Determines if fat mode should be used based on the maximum arity required.
    /// Fat mode is REQUIRED for arities > fallback_arity (usually 3 or 8),
    /// as the fixed-size array implementations only support up to this arity.
    pub fn should_use_fat_mode(&self, user_requested_fat_mode: bool, fallback_arity: usize) -> bool {
        // If any key or value arity exceeds fallback_arity, fat mode must be used
        // Otherwise, it depends on the user's command-line argument
        let maximal_pairs = self.maximal_arity_pairs();
        let any_exceeds_fallback = maximal_pairs.iter().any(|(k, v)| *k > fallback_arity || *v > fallback_arity);
        any_exceeds_fallback || user_requested_fat_mode
    }

    /// Returns detailed arity information for debugging purposes.
    /// Returns a vector of (transformation_name, input_key_value_arities, output_key_value_arity) tuples.
    pub fn arity_analysis(&self) -> Vec<(String, Vec<(usize, usize)>, (usize, usize))> {
        self.program_plan
            .iter()
            .flat_map(|group_plan| {
                group_plan.strata_plan().into_iter().map(|transformation| {
                    let output_arity = transformation.output().arity();
                    
                    let input_arities = if transformation.is_unary() {
                        let arity = transformation.unary().arity();
                        vec![arity]
                    } else {
                        let (left, right) = transformation.binary();
                        let left_arity = left.arity();
                        let right_arity = right.arity();
                        vec![left_arity, right_arity]
                    };
                    
                    let transformation_name = transformation.output().signature().debug_name().to_string();
                    
                    (transformation_name, input_arities, output_arity)
                })
            })
            .collect()
    }
}


impl fmt::Display for ProgramQueryPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, group_plan) in self.program_plan.iter().enumerate() {
            writeln!(f, "#{}\n{}\n", i, group_plan)?;
        }
        Ok(())
    }
}
