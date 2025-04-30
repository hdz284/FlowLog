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

    pub fn from_strata(strata: &Strata, _is_global_optimized: bool) -> Self {
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
                GroupStrataQueryPlan::new(is_recursive, rule_plans, &mut seen_set)
            })
            .collect();

        Self::new(program_plan)
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