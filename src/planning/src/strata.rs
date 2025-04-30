use std::fmt;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use parsing::rule::FLRule;
use crate::collections::CollectionSignature;
use crate::rule::RuleQueryPlan;

use crate::transformations::Transformation;

/* a group of non-recursive strata or a recursive stratum */ 
#[derive(Debug, Clone)]
pub struct GroupStrataQueryPlan {
    is_recursive: bool,
    rules: Vec<FLRule>,

    enter_scope: HashSet<Arc<CollectionSignature>>,                                                    // base and intermediates rel to bring into scope
    last_signatures_map: HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,             // sinks of the dataflow DAG (map head to a vector of last signatures)

    strata_plan: Vec<Vec<Transformation>>                                                     
}

impl GroupStrataQueryPlan {
    pub fn new(
        is_recursive: bool, 
        rule_plans: Vec<RuleQueryPlan>, 
        seen_set: &mut HashSet<Arc<CollectionSignature>>
    ) -> Self {
        let rules = rule_plans
            .iter()
            .map(|rp| rp.rule().clone())
            .collect::<Vec<FLRule>>();

        // populate the last_signatures_map (map head to a vector of last signatures)
        let last_signatures_map = rule_plans.iter().fold(
            HashMap::new(),
            |mut map: HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>, rp| {
                let head = Arc::new(CollectionSignature::new_atom(rp.rule().head().name()));
                map.entry(head).or_default().push(Arc::clone(rp.rule_plan().0.output().signature()));
                map
            },
        );

        /* init */
        let mut strata_plan = Vec::new();
        let mut enter_scope = HashSet::new();
        let mut nested_seen = HashSet::new();

        for rule_plan in rule_plans.iter() {
            let (root, transformation_tree) = rule_plan.rule_plan();

            if !is_recursive {
                strata_plan.push(Self::construct_non_recursive(seen_set, root, &transformation_tree));
            } else {
                let (rule_plan, rule_enter_scope) = Self::construct_recursive(seen_set, &mut nested_seen, root, &transformation_tree);
                strata_plan.push(rule_plan);
                enter_scope.extend(rule_enter_scope);
            }      
        }
        
        Self {
            is_recursive,
            rules,
            enter_scope,
            last_signatures_map,
            strata_plan
        }
    }

    fn construct_non_recursive(
        seen: &mut HashSet<Arc<CollectionSignature>>,
        root: &Transformation,
        transformation_tree: &HashMap<Transformation, (Transformation, Transformation)>
    ) -> Vec<Transformation>     
    {
        let output_signature = root.output().signature();
    
        // base case (already seen)
        if seen.contains(output_signature) { return vec![]; }

        // mark as seen
        seen.insert(Arc::clone(output_signature));

        transformation_tree.get(root).map_or_else(
            || vec![root.clone()], // leaf op
            |(l_root, r_root)| {
                // recursive case
                let mut plan = Vec::new();
                plan.extend(Self::construct_non_recursive(seen, l_root, transformation_tree));
                plan.extend(Self::construct_non_recursive(seen, r_root, transformation_tree));
                plan.push(root.clone());
                plan
            }
        )
    }

    fn construct_recursive(
        seen: &mut HashSet<Arc<CollectionSignature>>,
        nested_seen: &mut HashSet<Arc<CollectionSignature>>,
        root: &Transformation,
        transformation_tree: &HashMap<Transformation, (Transformation, Transformation)>
    ) -> (Vec<Transformation>, HashSet<Arc<CollectionSignature>>)  
    {
        let output_signature = root.output().signature();
    
        // base case (already seen)
        if seen.contains(output_signature) {
            // it can't be the that global scope has an intermediate rel that is produced by some recursive idb of this strata (we can safely reuse it)
            // println!("borrow {} from global", output_signature);
            return (vec![], HashSet::from([Arc::clone(&output_signature)]));
        }

        // base case (already nested_seen)
        if nested_seen.contains(output_signature) {
            // println!("borrow {} from nested", output_signature);
            return (vec![], HashSet::new());
        }

        // mark as nested_seen
        nested_seen.insert(Arc::clone(output_signature));

        transformation_tree.get(root).map_or_else(
            // base case (enter base atom into scope at a leaf op)
            // (careful) enter_scope contains idbs that are first defined in the recursive strata, the execution layer should inspect those and fetch from variables_map
            || (vec![root.clone()], HashSet::from([Arc::clone(root.unary().signature())])),  
            |(l_root, r_root)| {
                // recursive case
                let (l_plan, l_enter_scope) = Self::construct_recursive(seen, nested_seen, l_root, transformation_tree);
                let (r_plan, r_enter_scope) = Self::construct_recursive(seen, nested_seen, r_root, transformation_tree);

                (
                    l_plan.into_iter()
                        .chain(r_plan)
                        .chain(std::iter::once(root.clone()))
                        .collect(),
                    l_enter_scope.union(&r_enter_scope).cloned().collect()
                )
            }
        )
    }
        
    pub fn is_recursive(&self) -> bool {
        self.is_recursive
    }

    pub fn rules(&self) -> &Vec<FLRule> {
        &self.rules
    }

    pub fn strata_plan(&self) -> Vec<&Transformation> {
        self.strata_plan.iter().flatten().collect()
    }

    // head collection signatures of the strata
    pub fn head_signatures_set(&self) -> HashSet<Arc<CollectionSignature>> {
        self.last_signatures_map
            .keys()
            .cloned()
            .collect()
    }

    // heads (name and arity) of the strata
    pub fn heads(&self) -> HashMap<String, usize> { 
        self.rules
            .iter()
            .map(|rule| {
                (rule.head().name().to_string(), rule.head().arity())
            })
            .collect()
    }
    
    pub fn last_signatures_map(&self) -> &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>> > {
        &self.last_signatures_map
    }

    pub fn enter_scope_set(&self) -> &HashSet<Arc<CollectionSignature>> {
        &self.enter_scope
    }
}



impl fmt::Display for GroupStrataQueryPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.enter_scope.is_empty() {
            // print the first one
            write!(f, "[ent] {}", self.enter_scope.iter().next().unwrap())?;
            // print the rest
            for signature in self.enter_scope.iter().skip(1) {
                write!(f, " && {}", signature)?;
            }
            write!(f, "\n")?;
        }

        // if strata_plan is empty, print noop
        if self.strata_plan.is_empty() {
            write!(f, "[∅]")
        } else {
            write!(
                f, "{}",
                self.strata_plan
                    .iter()
                    .enumerate()
                    .map(|(i, transformations_per_rule)| {
                        let print_per_rule = transformations_per_rule
                            .iter()
                            .enumerate()
                            .map(|(j, transformation)| {
                                let prefix = if j == transformations_per_rule.len() - 1 {
                                    "└── "
                                } else {
                                    "├── "
                                };
                                format!("  {}{}", prefix, transformation)
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
            
                        format!("{}\n{}", &self.rules[i], print_per_rule)
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
    }
}