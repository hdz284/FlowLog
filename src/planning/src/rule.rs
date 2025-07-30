use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::vec;

use catalog::atoms::{AtomArgumentSignature, AtomSignature};
use catalog::compare::ComparisonExprPos;
use catalog::rule::Catalog;
use parsing::rule::FLRule;

use crate::collections::{Collection, CollectionSignature};
use crate::transformations::Transformation;
use optimizing::optimizer::PlanTree;

#[derive(Debug, Clone)]
pub struct RuleQueryPlan {
    rule: FLRule,
    dependent_atom_names: HashSet<String>,
    plan: PlanTree,                      // join spanning tree
    last_transformation: Transformation, // root of the transformation tree (including head transformation)
    transformation_tree: HashMap<Transformation, (Transformation, Option<Transformation>)>, // transformation tree with optional second child
}

impl RuleQueryPlan {
    pub fn rule(&self) -> &FLRule {
        &self.rule
    }

    pub fn rule_plan(
        &self,
    ) -> (
        &Transformation,
        &HashMap<Transformation, (Transformation, Option<Transformation>)>,
    ) {
        (&self.last_transformation, &self.transformation_tree)
    }

    pub fn dependent_atom_names(&self) -> &HashSet<String> {
        &self.dependent_atom_names
    }

    /* main entry */
    pub fn from_catalog(catalog: &Catalog, is_optimized: bool) -> Self {
        let plan = PlanTree::from_catalog(catalog, is_optimized);
        // debug!("join spanning tree: {:?}", plan);
        let mut is_active_negation_bitmap = vec![true; catalog.negated_atom_names().len()];

        // a vector of length catalog.atom_names().len()
        // for non-core atoms, initialize to true (active)
        let mut is_active_non_core_atom_bitmap = catalog
            .is_core_atom_bitmap()
            .iter()
            .map(|&is_core| !is_core)
            .collect::<Vec<bool>>();

        // all comparison predicates are active initially
        let active_comparison_predicates: Vec<usize> =
            (0..catalog.comparison_predicates().len()).collect();

        // head arithmics decomposed as strings
        let head_value_arguments: Vec<String> = catalog.head_variables_list();

        let (last_transformation, mut transformation_tree) = Self::recursive_transformations(
            &catalog,          // ground truth
            &plan.sub_trees(), // ground truth
            plan.root(),
            plan.tree(),
            &[],                   // final key arguments
            &head_value_arguments, // final value arguments
            &mut is_active_negation_bitmap,
            &mut is_active_non_core_atom_bitmap,
            &active_comparison_predicates,
        );

        // is_active_negation_bitmap and is_active_non_core_atom_bitmap are all false
        assert!(is_active_negation_bitmap.iter().all(|&x| !x));
        assert!(is_active_non_core_atom_bitmap.iter().all(|&x| !x));

        // post mapping to get actual head arithmics
        let atom_signatures = plan
            .sub_trees()
            .get(&plan.root())
            .unwrap()
            .into_iter()
            .map(|&rhs_id| AtomSignature::new(true, rhs_id))
            .collect::<Vec<AtomSignature>>();
        let head_transformation = Transformation::head(
            last_transformation.output().clone(),
            &catalog.top_down_trace_exprs(catalog.head_arithmetic_exprs(), &atom_signatures),
        );

        // Add head transformation as unary (no second child)
        transformation_tree.insert(
            head_transformation.clone(),
            (last_transformation.clone(), None),
        );

        Self {
            rule: catalog.rule().clone(),
            dependent_atom_names: catalog.dependent_atom_names(),
            plan,
            last_transformation: head_transformation.clone(), // Head transformation becomes the new root
            transformation_tree,
        }
    }

    fn recursive_transformations(
        catalog: &Catalog,
        sub_trees: &HashMap<usize, Vec<usize>>,
        root: usize,
        tree: &HashMap<usize, Vec<usize>>,
        head_key_arguments: &[String], // key arguments of the sub-query
        head_value_arguments: &[String], // value arguments of the sub-query
        is_active_negation_bitmap: &mut Vec<bool>,
        is_active_non_core_atom_bitmap: &mut Vec<bool>,
        active_comparison_predicates: &[usize], // invariant: active_comparison_predicates must be subsumed by all variables under the subtree at root
    ) -> (
        Transformation,
        HashMap<Transformation, (Transformation, Option<Transformation>)>,
    ) {
        /* decompose plan into sub-root ⋈ (...) ⋈ (...) ... */
        /* planning_atom_signature is the sub-root atom for processing */
        let planning_atom_signature = AtomSignature::new(true, root);
        let children = tree.get(&root).unwrap();

        if tree.get(&root).unwrap().is_empty() {
            // if is_leaf(root)
            /* base case (leaf atom) */
            /* semijoin (or antijoin) first if there are subatoms or negated atoms to push down */
            Self::per_atom_recursive_semijoins_and_antijoins(
                catalog,
                &planning_atom_signature,
                head_key_arguments,
                head_value_arguments,
                is_active_negation_bitmap,
                is_active_non_core_atom_bitmap,
                active_comparison_predicates,
            )
        } else {
            /* recursive case (peeling off the last subtree of the root as the planning side) */
            let planning_child = children.last().unwrap().clone();
            let planning_subtree = sub_trees.get(&planning_child).unwrap();

            /* root and (the child subtrees) except the last one are the leftover side */
            let leftover_subtrees = std::iter::once(root)
                .chain(
                    children[..children.len() - 1]
                        .iter()
                        .flat_map(|&x| sub_trees.get(&x).unwrap().clone()),
                )
                .collect::<Vec<usize>>();

            let head_arguments_set: HashSet<&String> = head_key_arguments
                .iter()
                .chain(head_value_arguments.iter())
                .collect();

            let mut join_key_strs = Vec::new();
            let mut planning_value_strs = Vec::new();
            let mut join_key_strs_set = HashSet::new();

            /* variables arguments for the both sides */
            let leftover_vars_set = catalog.vars_set(&leftover_subtrees);
            let planning_vars_set = catalog.vars_set(&planning_subtree);

            /* comparison predicates partitioning */
            let (join_comp_ids, left_comp_ids, right_comp_ids) = catalog
                .partition_comparison_predicates(
                    &leftover_vars_set,
                    &planning_vars_set,
                    active_comparison_predicates,
                );

            /* negation attached on the join */
            let negated_atom_signatures = catalog.attach_negated_atoms_on_joins(
                &leftover_vars_set,
                &planning_vars_set,
                is_active_negation_bitmap,
            );
            let negated_vars = catalog.negated_vars(
                &negated_atom_signatures
                    .iter()
                    .map(|signature| signature.rhs_id())
                    .collect::<Vec<usize>>(),
            );
            let negated_vars_set = negated_vars.iter().cloned().collect::<HashSet<&String>>(); // all negated vars needed for the antijoins after the join

            /* variables arguments for the comparisons fused inside the join */
            let join_active_vars = catalog.comparison_predicates_vars_set(&join_comp_ids);
            let join_active_vars_set = join_active_vars
                .clone()
                .into_iter()
                .collect::<HashSet<&String>>();

            /* list the join key arguments (and list the value arguments) of the planning side */
            let mut planning_vars_seen_set = HashSet::new();
            for planning_rhs_id in planning_subtree {
                for planning_argument_signature in
                    &catalog.atom_argument_signatures()[*planning_rhs_id]
                {
                    if catalog.is_const_or_var_eq_or_placeholder(planning_argument_signature) {
                        continue; // skip const, var_eq, and placeholder signatures
                    }

                    let planning_argument_str =
                        &catalog.signature_to_argument_str_map()[planning_argument_signature];
                    if planning_vars_seen_set.contains(planning_argument_str) {
                        // only consider the first occurrence of the argument from the planning side
                        continue;
                    }

                    planning_vars_seen_set.insert(planning_argument_str); // (optimization) mark the argument as seen to make sure repeated arguments are not being requested downstream twice

                    if leftover_vars_set.contains(planning_argument_str) {
                        /* joining */
                        join_key_strs.push(planning_argument_str.clone());
                        join_key_strs_set.insert(planning_argument_str.clone());
                    } else if head_arguments_set.contains(planning_argument_str)
                        || join_active_vars_set.contains(planning_argument_str)
                        || negated_vars_set.contains(planning_argument_str)
                    {
                        /* not joining but necessary for (1) head, (2) the fused comparison expressions, or (3) cross join negations */
                        planning_value_strs.push(planning_argument_str.clone());
                    }
                }
            }

            /* verify the join key arguments (and list the value arguments) of the leftover side */
            let mut leftover_value_strs = Vec::new();
            let mut leftover_vars_seen_set = HashSet::new();
            for leftover_rhs_id in &leftover_subtrees {
                for leftover_argument_signature in
                    &catalog.atom_argument_signatures()[*leftover_rhs_id]
                {
                    if catalog.is_const_or_var_eq_or_placeholder(leftover_argument_signature) {
                        continue; // skip const, var_eq, and placeholder signatures
                    }

                    let leftover_argument_str =
                        &catalog.signature_to_argument_str_map()[leftover_argument_signature];
                    if leftover_vars_seen_set.contains(leftover_argument_str) {
                        // only consider the first occurrence of the argument from the leftover side
                        continue;
                    }

                    leftover_vars_seen_set.insert(leftover_argument_str); // (optimization) mark the argument as seen to make sure repeated arguments are not being requested downstream twice

                    if planning_vars_set.contains(leftover_argument_str) {
                        /* joining (already recorded previously) */
                        assert!(
                            join_key_strs_set.contains(leftover_argument_str),
                            "join key arguments not inconsistent"
                        );
                    } else if head_arguments_set.contains(leftover_argument_str)
                        || join_active_vars_set.contains(leftover_argument_str)
                        || negated_vars_set.contains(leftover_argument_str)
                    {
                        /* not joining but necessary for (1) head, (2) the fused comparison expressions, or (3) cross join negations */
                        leftover_value_strs.push(leftover_argument_str.clone());
                    }
                }
            }

            // debug!("join key arguments: {:?}", join_key_strs);
            // debug!("leftover value arguments: {:?}", leftover_value_strs);
            // debug!("planning value arguments: {:?}", planning_value_strs);

            /* ------------------------------------ recursive calls ------------------------------------ */
            // clone the original tree and remove the last child from the entry for the root
            let mut truncated_tree = tree.clone();
            truncated_tree.get_mut(&root).unwrap().pop();

            /* recursive construct transformations on the planning subtree */
            let (right_transformation, right_tree) = Self::recursive_transformations(
                catalog,
                sub_trees,
                planning_child,
                &truncated_tree,
                &join_key_strs,
                &planning_value_strs,
                is_active_negation_bitmap,
                is_active_non_core_atom_bitmap,
                &right_comp_ids,
            );

            /* recursive construct transformations on the left subplan */
            let (left_transformation, left_tree) = Self::recursive_transformations(
                catalog,
                sub_trees,
                root,
                &truncated_tree,
                &join_key_strs,
                &leftover_value_strs,
                is_active_negation_bitmap,
                is_active_non_core_atom_bitmap,
                &left_comp_ids,
            );
            /* ------------------------------------ end of recursive calls ------------------------------------ */

            // for &comp_id in &join_comp_ids {
            //     debug!("join filters to be fused: {}", catalog.comparison_predicates()[comp_id]);
            // }

            /* ------------------------------------ final join (follow by one or more antijoins) transformations ------------------------------------ */
            // order of fetching atom signatures -- pre-order of join tree (equivalently, always using the leftmost in the transformation tree)
            let subtree_atom_signatures = sub_trees
                .get(&root)
                .unwrap()
                .into_iter()
                .map(|&rhs_id| AtomSignature::new(true, rhs_id))
                .collect::<Vec<AtomSignature>>();

            /* fused comparison filters */
            let compare_expr_signatures = Self::assemble_comparisons(
                catalog,
                &join_comp_ids,
                &subtree_atom_signatures,
                &leftover_vars_set
                    .union(&planning_vars_set)
                    .cloned()
                    .collect::<HashSet<&String>>(),
            );

            let (last_join, mut top_tree) = if negated_atom_signatures.is_empty() {
                /* no negation to be consumed at the join */
                let base_join = Transformation::join(
                    (
                        Arc::clone(left_transformation.output()),
                        Arc::clone(right_transformation.output()),
                    ), // the two collections must have identical keys (as string)
                    &catalog.top_down_trace(head_key_arguments, &subtree_atom_signatures),
                    &catalog.top_down_trace(head_value_arguments, &subtree_atom_signatures),
                    &compare_expr_signatures,
                );

                (
                    base_join.clone(),
                    HashMap::from([(base_join, (left_transformation, Some(right_transformation)))]),
                )
            } else {
                /* some negated case */
                // (1) know the keys for the first antijoin
                let first_negated_atom_signature = negated_atom_signatures.first().unwrap();
                let first_negated_rhs_id = first_negated_atom_signature.rhs_id();
                let first_negated_atom_argument_signatures =
                    &catalog.negated_atom_argument_signatures()[first_negated_rhs_id];
                let first_negated_atom_var_signatures = first_negated_atom_argument_signatures
                    .iter()
                    .filter(|signature| !catalog.is_const_or_var_eq_or_placeholder(signature))
                    .cloned()
                    .collect::<Vec<AtomArgumentSignature>>(); // discarding const, placeholder, and var_eq signatures

                let first_antijoin_key_arguments =
                    catalog.signature_to_argument_strs(&first_negated_atom_var_signatures);
                let first_antijoin_key_arguments_set: HashSet<&String> =
                    first_antijoin_key_arguments.iter().collect();

                // (2) know the values for the first antijoin
                let mut seen_set = HashSet::new();
                let first_antijoin_value_arguments: Vec<_> = join_key_strs
                    .iter()
                    .chain(leftover_value_strs.iter())
                    .chain(planning_value_strs.iter())
                    .filter(|&argument_str| {
                        // not already a key argument and is a necessary head argument or a necessary negated argument
                        !first_antijoin_key_arguments_set.contains(argument_str)
                            && seen_set.insert(argument_str) // (optimization) make sure repeated arguments are not being requested downstream twice
                    })
                    .cloned()
                    .collect();

                let base_join = Transformation::join(
                    (
                        Arc::clone(left_transformation.output()),
                        Arc::clone(right_transformation.output()),
                    ), // the two collections must have identical keys (as string)
                    &catalog
                        .top_down_trace(&first_antijoin_key_arguments, &subtree_atom_signatures),
                    &catalog
                        .top_down_trace(&first_antijoin_value_arguments, &subtree_atom_signatures),
                    &compare_expr_signatures,
                );

                /* apply a sequence of antijoins */
                let (last_antijoin, mut antijoin_tree) = Self::recursive_antijoins(
                    catalog,
                    &subtree_atom_signatures,
                    base_join.clone(),
                    &negated_atom_signatures,
                    head_key_arguments,
                    head_value_arguments,
                    active_comparison_predicates,
                );

                antijoin_tree.insert(base_join, (left_transformation, Some(right_transformation)));

                (last_antijoin, antijoin_tree)
            };

            /* graft the right tree and the left tree */
            top_tree.extend(left_tree);
            top_tree.extend(right_tree);

            /* ------------------------------------ end of final join transformation ------------------------------------ */
            (last_join, top_tree)
        }
    }

    fn per_atom_recursive_semijoins_and_antijoins(
        catalog: &Catalog,
        planning_atom_signature: &AtomSignature,
        head_key_arguments: &[String],
        head_value_arguments: &[String],
        is_active_negation_bitmap: &mut Vec<bool>,
        is_active_non_core_atom_bitmap: &mut Vec<bool>,
        active_comparison_predicates: &[usize],
    ) -> (
        Transformation,
        HashMap<Transformation, (Transformation, Option<Transformation>)>,
    ) {
        let planning_rhs_id = planning_atom_signature.rhs_id();
        let planning_atom_argument_signatures =
            &catalog.atom_argument_signatures()[planning_rhs_id];
        let planning_atom_var_signatures = planning_atom_argument_signatures
            .iter()
            .filter(|signature| !catalog.is_const_or_var_eq_or_placeholder(signature))
            .cloned()
            .collect::<Vec<AtomArgumentSignature>>(); // discarding const, placeholder, and var_eq signatures

        // for each subatom, check if it is active
        // if not, skip it
        // otherwise, set it to not active in the is_active_non_core_atom_bitmap and semijoin it
        let subatom_signatures = catalog
            .sub_atoms(&planning_atom_var_signatures)
            .into_iter()
            .filter(|subatom_signature| {
                let subatom_rhs_id = subatom_signature.rhs_id();
                if is_active_non_core_atom_bitmap[subatom_rhs_id] {
                    is_active_non_core_atom_bitmap[subatom_rhs_id] = false;
                    true // retain the subatom
                } else {
                    false // skip the subatom
                }
            })
            .collect::<Vec<AtomSignature>>();

        let negated_atom_signatures = catalog
            .sub_negated_atoms(&planning_atom_var_signatures)
            .into_iter()
            .filter(|negated_atom_signature| {
                let negated_rhs_id = negated_atom_signature.rhs_id();
                if is_active_negation_bitmap[negated_rhs_id] {
                    is_active_negation_bitmap[negated_rhs_id] = false;
                    true // retain the negated atom
                } else {
                    false // skip the negated atom
                }
            })
            .collect::<Vec<AtomSignature>>();

        if negated_atom_signatures.is_empty() {
            /* no negation */
            Self::recursive_semijoins(
                catalog,
                &planning_atom_signature,
                &subatom_signatures,
                head_key_arguments,
                head_value_arguments,
                active_comparison_predicates,
            )
        } else {
            /* some negated case */
            let first_negated_atom_signature = negated_atom_signatures.first().unwrap();
            let first_negated_rhs_id = first_negated_atom_signature.rhs_id();
            let first_negated_atom_argument_signatures =
                &catalog.negated_atom_argument_signatures()[first_negated_rhs_id];
            let first_negated_atom_var_signatures = first_negated_atom_argument_signatures
                .iter()
                .filter(|signature| !catalog.is_const_or_var_eq_or_placeholder(signature))
                .cloned()
                .collect::<Vec<AtomArgumentSignature>>(); // discarding const, placeholder, and var_eq signatures

            let subsequent_antijoin_key_arguments_str =
                catalog.signature_to_argument_strs(&first_negated_atom_var_signatures);
            let negated_vars_set = catalog.negated_vars_set(
                &negated_atom_signatures
                    .iter()
                    .map(|signature| signature.rhs_id())
                    .collect::<Vec<usize>>(),
            );

            let (left_transformation, bottom_tree) = Self::recursive_semijoins(
                catalog,
                &planning_atom_signature,
                &subatom_signatures,
                /* the key and value arguements are prepared for subsequent antijoins */
                &subsequent_antijoin_key_arguments_str,
                &catalog
                    .signature_to_argument_strs(&planning_atom_var_signatures)
                    .iter()
                    .filter(|&argument_str| {
                        // not already a key argument and is a necessary head argument or a necessary negated argument
                        !subsequent_antijoin_key_arguments_str
                            .iter()
                            .collect::<HashSet<&String>>()
                            .contains(argument_str)
                            && (head_key_arguments
                                .iter()
                                .chain(head_value_arguments.iter())
                                .collect::<HashSet<&String>>()
                                .contains(argument_str)
                                || negated_vars_set.contains(argument_str))
                    })
                    .cloned()
                    .collect::<Vec<String>>(),
                active_comparison_predicates,
            );

            /* apply a sequence of antijoins */
            let (root_transformation, mut top_tree) = Self::recursive_antijoins(
                catalog,
                &vec![*planning_atom_signature],
                left_transformation,
                &negated_atom_signatures,
                head_key_arguments,
                head_value_arguments,
                active_comparison_predicates,
            );

            // graft the bottom tree below the top tree
            top_tree.extend(bottom_tree);

            (root_transformation, top_tree)
        }
    }

    fn recursive_antijoins(
        catalog: &Catalog,
        planning_atom_signatures: &Vec<AtomSignature>,
        last_transformation: Transformation,
        negated_atom_signatures: &Vec<AtomSignature>,
        head_key_arguments: &[String],
        head_value_arguments: &[String],
        active_comparison_predicates: &[usize], // only consumed by the negated atoms, not the planning atoms
    ) -> (
        Transformation,
        HashMap<Transformation, (Transformation, Option<Transformation>)>,
    ) {
        if negated_atom_signatures.is_empty() {
            /* caller of recursive_antijoins should have prepared previous transformations */
            (last_transformation, HashMap::new())
        } else {
            let negated_atom_signature = negated_atom_signatures.last().unwrap();
            let negated_rhs_id = negated_atom_signature.rhs_id();

            let negated_atom_argument_signatures =
                &catalog.negated_atom_argument_signatures()[negated_rhs_id];
            let negated_base_collection = Collection::new(
                CollectionSignature::new_atom(&catalog.negated_atom_names()[negated_rhs_id]),
                &vec![],
                negated_atom_argument_signatures,
            );

            let negated_atom_var_signatures = negated_atom_argument_signatures
                .iter()
                .filter(|signature| !catalog.is_const_or_var_eq_or_placeholder(signature))
                .cloned()
                .collect::<Vec<AtomArgumentSignature>>(); // discarding const, placeholder, and var_eq signatures

            /* determine the head (k, v) for the recursive call */
            let negated_head_key_arguments =
                catalog.signature_to_argument_strs(&negated_atom_var_signatures);
            let negated_head_key_arguments_set: HashSet<&String> =
                negated_head_key_arguments.iter().collect();
            let mut negated_head_arguments_set: HashSet<&String> = HashSet::new(); // (optimization) for de-duplication, insert returns true if the element is not in the set yet
            let negated_head_value_arguments: Vec<_> = head_key_arguments
                .iter()
                .chain(head_value_arguments.iter())
                .filter(
                    |&argument_str| {
                        !negated_head_key_arguments
                            .iter()
                            .collect::<HashSet<&String>>()
                            .contains(argument_str)
                            && negated_head_arguments_set.insert(argument_str)
                    }, // (optimization) make sure repeated head arguments are not being requested downstream twice
                )
                .cloned()
                .collect();

            /* prepare for active comparison expressions (the base relation should be capable of consuming every active comparisons) */
            let compare_expr_signatures = Self::assemble_comparisons(
                catalog,
                active_comparison_predicates,
                &vec![*negated_atom_signature],
                &negated_head_key_arguments_set,
            );

            // for compare_expr_signature in &compare_expr_signatures {
            //     debug!("comparison signatures for {}: {}", negated_atom_signature, compare_expr_signature);
            // }

            let (left_transformation, mut tree) = Self::recursive_antijoins(
                catalog,
                planning_atom_signatures,
                last_transformation,
                &negated_atom_signatures[..negated_atom_signatures.len() - 1].to_vec(),
                &negated_head_key_arguments,
                &negated_head_value_arguments,
                active_comparison_predicates,
            );

            /* get the output collection of the last transformation */
            let subplan_output_collection = left_transformation.output();

            /* the output collection is gonna antijoin the last subatom_signature */
            /* (1) the leaf level row-to-kv transformation of the last subatom_signature */
            let right_transformation = Transformation::kv_to_kv(
                Arc::new(negated_base_collection),
                &negated_atom_var_signatures,
                &vec![],
                &catalog.const_signatures(negated_atom_argument_signatures),
                &catalog.var_eq_signatures(negated_atom_argument_signatures),
                &compare_expr_signatures,
            );

            /* (2) the full antijoin transformation */
            // the search scope for the head arguments only from the planning collection as it is an antijoin
            /*  unfortunatlely differential dataflow doesn't allow concat of kv collections (hence we force a antijoin followed by a row-to-kv transformation if necessary) */
            let root_transformation = Transformation::antijoin(
                (
                    Arc::clone(subplan_output_collection),
                    Arc::clone(right_transformation.output()),
                ), // the two collections must have identical keys (as string)
                &catalog.top_down_trace(&head_key_arguments, planning_atom_signatures),
                &catalog.top_down_trace(&head_value_arguments, planning_atom_signatures),
            );

            // debug!("antijoin inserted: {}", root_transformation.flow());

            tree.insert(
                root_transformation.clone(),
                (left_transformation, Some(right_transformation)),
            );

            (root_transformation, tree)
        }
    }

    fn recursive_semijoins(
        catalog: &Catalog,
        planning_atom_signature: &AtomSignature,
        subatom_signatures: &Vec<AtomSignature>,
        head_key_arguments: &[String],
        head_value_arguments: &[String],
        active_comparison_predicates: &[usize],
    ) -> (
        Transformation,
        HashMap<Transformation, (Transformation, Option<Transformation>)>,
    ) {
        if subatom_signatures.is_empty() {
            let planning_rhs_id = planning_atom_signature.rhs_id();
            let planning_atom_argument_signatures =
                &catalog.atom_argument_signatures()[planning_rhs_id];

            let planning_row_collection = Collection::new(
                CollectionSignature::new_atom(&catalog.atom_names()[planning_rhs_id]),
                &vec![],
                planning_atom_argument_signatures,
            );

            /* prepare for active comparison expressions (the base relation should be capable of consuming every active comparisons) */
            let compare_expr_signatures = Self::assemble_comparisons(
                catalog,
                active_comparison_predicates,
                &vec![*planning_atom_signature],
                &catalog
                    .signature_to_argument_strs(&planning_atom_argument_signatures)
                    .iter()
                    .collect::<HashSet<&String>>(),
            );
            assert!(
                active_comparison_predicates.len() == compare_expr_signatures.len(),
                "active comparisons for semijoins are not fully consumed by the base"
            );

            // for compare_expr_signature in &compare_expr_signatures {
            //     debug!("comparison signatures for {}: {}", planning_atom_signature, compare_expr_signature);
            // }

            let leaf_transformation = Transformation::kv_to_kv(
                Arc::new(planning_row_collection),
                &catalog.top_down_trace(head_key_arguments, &vec![*planning_atom_signature]),
                &catalog.top_down_trace(head_value_arguments, &vec![*planning_atom_signature]),
                &catalog.const_signatures(planning_atom_argument_signatures),
                &catalog.var_eq_signatures(planning_atom_argument_signatures),
                &compare_expr_signatures,
            );

            (leaf_transformation, HashMap::new())
        } else {
            let subatom_signature = subatom_signatures.last().unwrap();
            let subatom_rhs_id = subatom_signature.rhs_id();

            let subatom_argument_signatures = &catalog.atom_argument_signatures()[subatom_rhs_id]; // full signature
            let subatom_base_collection = Collection::new(
                CollectionSignature::new_atom(&catalog.atom_names()[subatom_rhs_id]),
                &vec![],
                subatom_argument_signatures,
            );

            let subatom_var_signatures = subatom_argument_signatures
                .iter()
                .filter(|signature| !catalog.is_const_or_var_eq_or_placeholder(signature))
                .cloned()
                .collect::<Vec<AtomArgumentSignature>>(); // discarding const, placeholder, and var_eq signatures from the full signature

            /* determine the head (k, v) for the recursive call */
            let sub_head_key_arguments =
                catalog.signature_to_argument_strs(&subatom_var_signatures);
            let sub_head_key_arguments_set: HashSet<&String> =
                sub_head_key_arguments.iter().collect();
            let mut sub_head_arguments_set: HashSet<&String> = HashSet::new(); // (optimization) for de-duplication, insert returns true if the element is not in the set yet
            let sub_head_value_arguments: Vec<_> = head_key_arguments
                .iter()
                .chain(head_value_arguments.iter())
                .filter(
                    |&argument_str| {
                        !sub_head_key_arguments
                            .iter()
                            .collect::<HashSet<&String>>()
                            .contains(argument_str)
                            && sub_head_arguments_set.insert(argument_str)
                    }, // (optimization) make sure repeated head arguments are not being requested downstream twice
                )
                .cloned()
                .collect();

            /* prepare for active comparison expressions (those that are subsumed by the subatom signatures are consumed) */
            let compare_expr_signatures = Self::assemble_comparisons(
                catalog,
                active_comparison_predicates,
                &vec![*subatom_signature],
                &sub_head_key_arguments_set,
            );

            // for compare_expr_signature in &compare_expr_signatures {
            //     debug!("comparison signatures for {}: {}", subatom_signature, compare_expr_signature);
            // }

            let (left_transformation, mut tree) = Self::recursive_semijoins(
                catalog,
                planning_atom_signature,
                &subatom_signatures[..subatom_signatures.len() - 1].to_vec(),
                &sub_head_key_arguments,
                &sub_head_value_arguments,
                active_comparison_predicates,
            );

            /* get the output collection of the last transformation */
            let subplan_output_collection = left_transformation.output();

            /* the output collection is gonna semijoin the last subatom_signature */
            /* (1) the leaf level row-to-kv transformation of the last subatom_signature */
            let right_transformation = Transformation::kv_to_kv(
                Arc::new(subatom_base_collection),
                &subatom_var_signatures,
                &vec![],
                &catalog.const_signatures(subatom_argument_signatures),
                &catalog.var_eq_signatures(subatom_argument_signatures),
                &compare_expr_signatures,
            );

            /* (2) the semijoin transformation */
            let root_transformation = Transformation::join(
                (
                    Arc::clone(subplan_output_collection),
                    Arc::clone(right_transformation.output()),
                ), // the two collections must have identical keys (as string)
                &catalog.top_down_trace(&head_key_arguments, &vec![*planning_atom_signature]), // the search scope for the head arguments should only from the planning atom as it is a semijoin
                &catalog.top_down_trace(&head_value_arguments, &vec![*planning_atom_signature]),
                &vec![], // no comparison expressions at the semijoin level
            );

            tree.insert(
                root_transformation.clone(),
                (left_transformation, Some(right_transformation)),
            );

            (root_transformation, tree)
        }
    }

    // helper that identifies active comparisons to push to the atom level
    fn assemble_comparisons(
        catalog: &Catalog,
        active_comparison_predicates: &[usize],
        atom_signatures: &Vec<AtomSignature>,
        atom_vars_set: &HashSet<&String>,
    ) -> Vec<ComparisonExprPos> {
        active_comparison_predicates
            .iter()
            .filter_map(|&comp_id| {
                let compare_expr = &catalog.comparison_predicates()[comp_id];
                let left_vars: Vec<String> =
                    compare_expr.left_vars().into_iter().cloned().collect();
                let right_vars: Vec<String> =
                    compare_expr.right_vars().into_iter().cloned().collect();

                // only consider the comparison expressions if left_vars and right_vars are both subsets of atom_vars_set
                if left_vars
                    .iter()
                    .chain(right_vars.iter())
                    .all(|var| atom_vars_set.contains(var))
                {
                    if atom_signatures
                        .iter()
                        .all(|atom_signature| atom_signature.is_positive())
                    {
                        // positive atom
                        Some(ComparisonExprPos::from_comparison_expr(
                            compare_expr,
                            &catalog.top_down_trace(&left_vars, atom_signatures),
                            &catalog.top_down_trace(&right_vars, atom_signatures),
                        ))
                    } else {
                        // negative atom (expect only one negation atom in the atom_signatures)
                        Some(ComparisonExprPos::from_comparison_expr(
                            compare_expr,
                            &catalog.top_down_trace_negated(&left_vars, atom_signatures),
                            &catalog.top_down_trace_negated(&right_vars, atom_signatures),
                        ))
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

impl fmt::Display for RuleQueryPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // trace visited nodes to avoid re-printing them (in case of cycles in the DAG)
        let mut visited = HashSet::new();

        // helper
        fn print_transformation(
            f: &mut fmt::Formatter<'_>,
            transformation_tree: &HashMap<Transformation, (Transformation, Option<Transformation>)>,
            transformation: &Transformation,
            visited: &mut HashSet<Transformation>,
            indent: &str,  // indentation string
            is_last: bool, // whether this node is the last child of its parent
        ) -> fmt::Result {
            if visited.contains(transformation) {
                return Ok(()); // if the transformation is already visited, don't print it again.
            }

            // print the current transformation with a tree-like structure
            writeln!(
                f,
                "{}{}{}",
                indent,
                if is_last { "└── " } else { "├── " },
                transformation
            )?;

            visited.insert(transformation.clone());

            // recursively print the children with further indentation
            if let Some(downstream) = transformation_tree.get(transformation) {
                let new_indent = format!("{}{}", indent, if is_last { "    " } else { "│   " });

                // Check if this is a unary transformation (no second child)
                if downstream.1.is_none() {
                    // Unary transformation - only print one child
                    print_transformation(
                        f,
                        transformation_tree,
                        &downstream.0,
                        visited,
                        &new_indent,
                        true,
                    )?;
                } else {
                    // Binary transformation - print both children
                    // print the right child
                    print_transformation(
                        f,
                        transformation_tree,
                        downstream.1.as_ref().unwrap(),
                        visited,
                        &new_indent,
                        false,
                    )?;

                    // print the left child
                    print_transformation(
                        f,
                        transformation_tree,
                        &downstream.0,
                        visited,
                        &new_indent,
                        true,
                    )?;
                }
            }

            Ok(())
        }

        // print the rule
        writeln!(f, "{}", self.rule)?;
        // print the plan tree
        writeln!(f, "tw ({})\n{}", self.plan.tree_width(), self.plan)?;

        // start printing from the last_transformation with 0 indentation level
        writeln!(f, "plan")?;
        print_transformation(
            f,
            &self.transformation_tree,
            &self.last_transformation,
            &mut visited,
            "",
            true,
        )
    }
}
