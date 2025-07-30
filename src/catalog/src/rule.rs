use std::collections::{HashMap, HashSet};
use crate::arithmetic::ArithmeticPos;
use parsing::compare::ComparisonExpr;
use parsing::rule::{Atom, AtomArg, Const, FLRule, Predicate};
use parsing::head::{Head, HeadArg};
use parsing::arithmetic::{Factor, Arithmetic};
use crate::atoms::{AtomSignature, AtomArgumentSignature};
use crate::filters::BaseFilters;
use tracing::debug;

/* per-rule catalog */
#[derive(Debug)]
pub struct Catalog {
    rule: FLRule,                                                                         // rule

    signature_to_argument_str_map: HashMap<AtomArgumentSignature, String>,                // map each (core, negative or subatom) signature to the argument_str (reverse map)
    argument_presence_map: HashMap<String, Vec<Option<AtomArgumentSignature>>>,           // map each argument_str to the first presence of the argument in every core atom (None if absent)
    
    head_arithmetic_exprs: Vec<Arithmetic>,

    head_variables_list: Vec<String>,

    atom_names: Vec<String>,                                                              // list of rhs atom names               
    atom_argument_signatures: Vec<Vec<AtomArgumentSignature>>,                            // for each rhs positive atom, a vector of argument signatures
    is_core_atom_bitmap: Vec<bool>,                                                       // bitmap of core atoms

    negated_atom_names: Vec<String>,                                                      // list of rhs negated atom names
    negated_atom_argument_signatures: Vec<Vec<AtomArgumentSignature>>,                    // for each rhs negated atom, a vector of argument signatures

    base_filters: BaseFilters,                                                            // (local filters) variable equality constraints, constant equality constraints, placeholder set
    
    comparison_predicates: Vec<ComparisonExpr>,                                           // comparison predicates
    comparison_predicates_vars_set: Vec<HashSet<String>>,                                 // comparison predicates vars set

    head_arguments_map: HashMap<String, HeadArg>,                                        // head arguments map (for each head argument as a string, map to itself)
}
 
/* getters */
impl Catalog {
    pub fn rule(&self) -> &FLRule {
        &self.rule
    }

    pub fn head_arguments_map(&self) -> &HashMap<String, HeadArg> {
        &self.head_arguments_map
    }

    pub fn dependent_atom_names(&self) -> HashSet<String> {
        self.atom_names
            .iter()
            .chain(self.negated_atom_names.iter())
            .cloned()
            .collect::<HashSet<String>>()
    }
    
    pub fn signature_to_argument_str_map(&self) -> &HashMap<AtomArgumentSignature, String> {
        &self.signature_to_argument_str_map
    }

    pub fn signature_to_argument_strs(&self, argument_signatures: &Vec<AtomArgumentSignature>) -> Vec<String> {
        argument_signatures
            .iter()
            .filter_map(|signature| 
                self.signature_to_argument_str_map.get(signature).cloned() // skip if the signature is not in the map
            )
            .collect::<Vec<String>>()
    }

    /* get a list of subatoms w.r.t. to the given signature arguments */
    pub fn sub_atoms(&self, signature_arguments: &Vec<AtomArgumentSignature>) -> Vec<AtomSignature> {
        let signature_arguments_str_set = self.signature_to_argument_strs(signature_arguments).into_iter().collect::<HashSet<String>>();
        self.is_core_atom_bitmap()
            .iter()
            .enumerate()
            .filter_map(|(i, &is_core)| {
                if !is_core {
                    let atom_argument_strs_set = self.signature_to_argument_strs(&self.atom_argument_signatures()[i]).into_iter().collect::<HashSet<String>>();
                    if atom_argument_strs_set.is_subset(&signature_arguments_str_set) {
                        Some(AtomSignature::new(true, i))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<AtomSignature>>()
    }

    /* get a list of active negated atoms w.r.t. to the given signature arguments */
    pub fn sub_negated_atoms(&self, signature_arguments: &Vec<AtomArgumentSignature>) -> Vec<AtomSignature> {
        let signature_arguments_str_set = self.signature_to_argument_strs(signature_arguments).into_iter().collect::<HashSet<String>>();
        self.negated_atom_argument_signatures
            .iter()
            .enumerate()
            .filter_map(|(i, negated_atom_argument_signatures)| {
                let negated_atom_argument_strs_set = self.signature_to_argument_strs(negated_atom_argument_signatures).into_iter().collect::<HashSet<String>>();
                if negated_atom_argument_strs_set.is_subset(&signature_arguments_str_set) {
                    Some(AtomSignature::new(false, i))
                } else {
                    None
                }
            })
            .collect::<Vec<AtomSignature>>()
    }

    pub fn argument_presence_map(&self, argument_str: &String) -> &Vec<Option<AtomArgumentSignature>> {
        &self.argument_presence_map[argument_str]
    }
    
    pub fn atom_names(&self) -> &Vec<String> {
        &self.atom_names
    }

    pub fn atom_argument_signatures(&self) -> &Vec<Vec<AtomArgumentSignature>> {
        &self.atom_argument_signatures
    }
    
    pub fn is_core_atom_bitmap(&self) -> &Vec<bool> {
        &self.is_core_atom_bitmap
    }

    pub fn negated_atom_names(&self) -> &Vec<String> {
        &self.negated_atom_names
    }

    pub fn negated_atom_argument_signatures(&self) -> &Vec<Vec<AtomArgumentSignature>> {
        &self.negated_atom_argument_signatures
    }

    pub fn head_name(&self) -> &String {
        self.rule.head().name()
    }

    pub fn head_variables_list(&self) -> Vec<String> {
        self.head_variables_list.clone()
    }

    pub fn head_arithmetic_exprs(&self) -> &Vec<Arithmetic> {
        &self.head_arithmetic_exprs
    }

    pub fn is_const_or_var_eq_or_placeholder(&self, signature: &AtomArgumentSignature) -> bool {
        self.base_filters.is_const_or_var_eq_or_placeholder(signature) 
    }

    pub fn const_signatures(&self, signature_arguments: &Vec<AtomArgumentSignature>) -> Vec<(AtomArgumentSignature, Const)> {
        signature_arguments
            .iter()
            .filter_map(|signature| {
                if let Some(constant) = self.base_filters.const_map().get(signature) {
                    Some((signature.clone(), constant.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn var_eq_signatures(&self, signature_arguments: &Vec<AtomArgumentSignature>) -> Vec<(AtomArgumentSignature, AtomArgumentSignature)> {
        signature_arguments
            .iter()
            .filter_map(|alias| {
                if let Some(signature) = self.base_filters.var_eq_map().get(alias) {
                    Some((signature.clone(), alias.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn vars_set(&self, rhs_ids: &Vec<usize>) -> HashSet<&String> {
        // get all argument_strs of the given list of rhs_ids (for positive atoms)
        rhs_ids
            .iter()
            .flat_map(|&rhs_id| self.atom_argument_signatures[rhs_id].iter())
            .filter_map(|signature| 
                // skip const, var_eq, and placeholder signatures
                if self.is_const_or_var_eq_or_placeholder(signature) {
                    None
                } else {
                    Some(&self.signature_to_argument_str_map()[signature])
                    // or self.signature_to_argument_str_map.get(signature)
                }
            )   
            .collect()
    }

    pub fn negated_vars_set(&self, neg_rhs_ids: &Vec<usize>) -> HashSet<&String> {
        // get all argument_strs of the given list of neg_rhs_ids (for negated atoms)
        Self::negated_vars(&self, neg_rhs_ids)
            .into_iter()
            .collect()
    }

    pub fn negated_vars(&self, neg_rhs_ids: &Vec<usize>) -> Vec<&String> {
        // get all argument_strs of the given list of neg_rhs_ids (for negated atoms)
        neg_rhs_ids
            .iter()
            .flat_map(|&neg_rhs_id| self.negated_atom_argument_signatures[neg_rhs_id].iter())
            .filter_map(|signature| 
                // skip const, var_eq, and placeholder signatures
                if self.is_const_or_var_eq_or_placeholder(signature) {
                    None
                } else {
                    Some(&self.signature_to_argument_str_map()[signature])
                    // or self.signature_to_argument_str_map.get(signature)
                }
            )   
            .collect()
    }

    pub fn comparison_predicates(&self) -> &Vec<ComparisonExpr> {
        &self.comparison_predicates
    }

    pub fn comparison_predicates_vars_set(&self, comp_ids: &Vec<usize>) -> Vec<&String> {
        comp_ids
            .iter()
            .flat_map(|&comp_id| self.comparison_predicates_vars_set[comp_id].iter())
            .collect()
    }

    /* partition the comparison predicates into join, left and right (left and right are not necessarily disjoint to each other) */
    pub fn partition_comparison_predicates(
        &self, 
        left_vars_set: &HashSet<&String>, 
        right_vars_set: &HashSet<&String>,
        active_comparison_predicates: &[usize]
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        /*
            first, every comparison predicate must be a subset of the union of left_vars_set and right_vars_set
            if comparison is subset of left_vars_set => left
            if comparison is subset of right_vars_set => right
            else (a comparison is neither) => join
         */
        let mut join_predicates = Vec::new();
        let mut left_predicates = Vec::new();
        let mut right_predicates = Vec::new();

        for &i in active_comparison_predicates {
            let comparison_predicate = &self.comparison_predicates[i];
            let vars_set = comparison_predicate.vars_set();
            let union_vars_set: HashSet<&String> = left_vars_set.union(right_vars_set).cloned().collect();

            assert!(vars_set.is_subset(&union_vars_set), "comp vars {:?} not a subset of the subtree vars {:?}", vars_set, union_vars_set);
            if vars_set.is_subset(left_vars_set) {
                left_predicates.push(i);
            } 

            if vars_set.is_subset(right_vars_set) {
                right_predicates.push(i);
            }

            if !vars_set.is_subset(left_vars_set) && !vars_set.is_subset(right_vars_set) {
                join_predicates.push(i);
            }
        }

        (join_predicates, left_predicates, right_predicates)
    }

    /* isolate out the negated atoms over join (mark those as not active) */
    pub fn attach_negated_atoms_on_joins(
        &self, 
        left_vars_set: &HashSet<&String>, 
        right_vars_set: &HashSet<&String>,
        is_active_negation_bitmap: &mut Vec<bool>
    ) -> Vec<AtomSignature> {
        let mut isolated_negated_atoms = Vec::new();

        for (i, negated_atom_argument_signatures) in self.negated_atom_argument_signatures.iter().enumerate() {
            // if it is not active, skip
            if !is_active_negation_bitmap[i] {
                continue;
            }

            let negated_atom_argument_strs = self.signature_to_argument_strs(negated_atom_argument_signatures);
            let negated_atom_argument_strs_set = negated_atom_argument_strs.iter().collect::<HashSet<&String>>();

            if !negated_atom_argument_strs_set.is_subset(left_vars_set) && !negated_atom_argument_strs_set.is_subset(right_vars_set) 
                && negated_atom_argument_strs_set.is_subset(&left_vars_set.union(right_vars_set).cloned().collect())
            {
                // if the negated atom (1) is not a subset of both left and right, and (2) is a subset of the union of left and right, it should be attached on the join
                isolated_negated_atoms.push(AtomSignature::new(false, i));
                is_active_negation_bitmap[i] = false;
            }
        }

        isolated_negated_atoms
    }
}


impl Catalog {
    /* main constructor */
    pub fn from_strata(rule: &FLRule) -> Self {
        let (signature_to_argument_str_map, 
                atom_names, 
                atom_argument_signatures, // all non-repeating variables
                negated_atom_names, 
                negated_atom_argument_signatures,
                base_filters,
                comparison_predicates
            ) = Self::populate_argument_signatures(rule);

        let argument_presence_map = Self::populate_argument_presence_map(&signature_to_argument_str_map, &atom_argument_signatures, &base_filters);
        let is_core_atom_bitmap = Self::populate_is_core_atom_bitmap(&signature_to_argument_str_map, &atom_argument_signatures);

        let comparison_predicates_vars_set = Self::populate_comparison_predicates_vars_set(&comparison_predicates);

        // populate the head map (i.e. for each head arguments as a str, map to itself), e.g. x -> x, x + y -> ArthmicPos(x, [+ y])
        let head_arguments_map: HashMap<String, HeadArg> = rule
            .head()
            .head_arguments()
            .iter()
            .map(|head_arg| {
                (head_arg.to_string(), head_arg.clone()) // `to_string` needs the `Display` trait
            })
            .collect();

        let (head_variables_list, head_arithmetic_exprs) =
            Self::extract_head_arguments(rule.head().head_arguments());

        Self { rule: rule.clone(),
               signature_to_argument_str_map,
               argument_presence_map,
               head_arithmetic_exprs,
               head_variables_list,
               atom_names,
               atom_argument_signatures,
               is_core_atom_bitmap,
               negated_atom_names,
               negated_atom_argument_signatures,
               base_filters,
               comparison_predicates,
               comparison_predicates_vars_set,
               head_arguments_map,
            }
    }


    /* TODO! test sip w/ negation, arithmics, filters */
    /* sideways info passing (rewrite one rule into a set of rules) */
    /* e.g. 
          Assign(actual, formal) :-
            CallGraphEdge(invocation, method),
            ActualParam(index, invocation, actual),
            FormalParam(index, method, formal).
        goes into 
        (0) CallGraphEdge_sip0(invocation, method) :- CallGraphEdge(invocation, method).
        (1) ActualParam_sip1(index, invocation, actual) :- ActualParam(index, invocation, actual), CallGraphEdge_sip0(invocation, _).
        (2) FormalParam_sip2(index, method, formal) :- FormalParam(index, method, formal), CallGraphEdge_sip0(_, method), ActualParam_sip1(index, _, _).
        (3) Assign(actual, formal) :-
            CallGraphEdge_sip0(invocation, method),
            ActualParam_sip1(index, invocation, actual),
            FormalParam_sip2(index, method, formal).
     */
    fn reducer(
        &self, 
        suffix: &str,
        base_rule: &FLRule,
        core_ids: &Vec<usize>,                                    // sideway passing order
        atoms: &mut Vec<Predicate>,
        negated_atoms: &Vec<Predicate>,
        cmprs: &Vec<Predicate>,
        is_active_non_core_atom_bitmap: &mut Vec<bool>,
        is_active_negation_bitmap: &mut Vec<bool>
    ) -> Vec<FLRule> {
        let mut sideway_rules = Vec::new();         // sideway rules, one for each core atom of the rule

        for (i, core_id) in core_ids.iter().enumerate() {
            let base_argument_signatures = &self.atom_argument_signatures()[*core_id];
            let base_arguments = self.signature_to_argument_strs(base_argument_signatures);
            let mut base_arguments_set = HashSet::new();
            let sideway_vars = base_arguments
                .iter()
                .filter_map(|arg| {
                    if base_arguments_set.insert(arg) { Some(arg) } else { None }
                })
                .collect::<Vec<&String>>();
            
            let subatom_ids = self.sub_atoms(base_argument_signatures)
                .into_iter()
                .filter_map(|subatom_signature| {
                    let subatom_id = subatom_signature.rhs_id();
                    if is_active_non_core_atom_bitmap[subatom_id] {
                        is_active_non_core_atom_bitmap[subatom_id] = false;
                        Some(subatom_id) // retain the subatom
                    } else {
                        None // skip the subatom
                    }
                })
                .collect::<Vec<usize>>();
            
            let negated_atom_ids = self.sub_negated_atoms(base_argument_signatures)
                .into_iter()
                .filter_map(|negated_atom_signature| {
                    let negated_id = negated_atom_signature.rhs_id();
                    if is_active_negation_bitmap[negated_id] {
                        is_active_negation_bitmap[negated_id] = false;
                        Some(negated_id) // retain the negated atom
                    } else {
                        None // skip the negated atom
                    }
                })
                .collect::<Vec<usize>>();
                
            let comparison_ids = self.comparison_predicates_vars_set.iter().enumerate()
                .filter_map(|(i, vars_set)| {
                    if vars_set.iter().collect::<HashSet<&String>>().is_subset(&base_arguments_set) { Some(i) } else { None }
                })
                .collect::<Vec<usize>>();

            // construct the sideway rule for the core atom 
            // (head) the atom itself modulo const / var eq, e.g. FormalParam_sip2(index, method, formal) :- ...
            let sideway_name = format!("{}_{}{}", atoms[*core_id].name(), suffix, i);
            let sideway_head = Head::new(
                sideway_name.clone(), 
                sideway_vars.iter().map(|&arg| HeadArg::Var(arg.clone())).collect()
            );

            //  (semijoins) e.g. CallGraphEdge_sip0(_, method) goes into
            //        FormalParam_sip2(index, method, formal) :- FormalParam(index, method, formal), CallGraphEdge_sip0(_, method), ActualParam_sip1(index, _, _).
            // (1) init w/ the base atom
            let mut sideway_rhs = vec![atoms[*core_id].clone()]; 

            // (2) stitch sub_atoms, sub_negated_atoms and comparisons (if it is subset of the base arguments)
            sideway_rhs.extend(
                subatom_ids.iter().map(|&subatom_id| atoms[subatom_id].clone())
                    .chain(
                        negated_atom_ids.iter().map(|&negated_atom_id| negated_atoms[negated_atom_id].clone())
                    )
                    .chain(
                        comparison_ids.iter().map(|&comp_id| cmprs[comp_id].clone())
                    )
            );

            // (3) for each non-disjoint core atoms prior to i^th, insert w/ only join vars, others sets to _
            for jn_id in core_ids[..i].iter() {
                let atom_argument_signatures = &self.atom_argument_signatures()[*jn_id];
                let atom_arguments = self.signature_to_argument_strs(atom_argument_signatures);
                let atom_arguments_set = atom_arguments.iter().collect::<HashSet<&String>>();
                if !base_arguments_set.is_disjoint(&atom_arguments_set) {
                    let mut new_atom_arguments = Vec::new();
                    for atom_arg in atoms[*jn_id].arguments() {
                        if atom_arg.is_var() && !base_arguments_set.contains(atom_arg.as_var()) {
                            new_atom_arguments.push(AtomArg::Placeholder);
                        } else {
                            new_atom_arguments.push(atom_arg.clone());
                        }
                    }

                    let intersect_atom = Atom::from_str(atoms[*jn_id].name(), new_atom_arguments);
                    sideway_rhs.push(Predicate::AtomPredicate(intersect_atom));
                }
            }

            if sideway_rhs.len() == 1 { 
                // skip trivial rules (only contains the base atom)
                continue;
            }

            // in-place change the atom to its reduced version
            atoms[*core_id] = 
                Predicate::AtomPredicate(
                    Atom::from_str(
                        &sideway_name.clone(), 
                        sideway_vars.iter().map(|&arg| AtomArg::Var(arg.clone())).collect()
                    )
                );

            // construct final rule
            sideway_rules.push(
                FLRule::new(sideway_head, sideway_rhs, base_rule.is_planning(), base_rule.is_sip())
            );
        }

        for r in &sideway_rules { debug!("{}", r); }
        sideway_rules
    }


    pub fn sideways(&self, rule_loc: usize) -> Vec<Catalog> {
        /* basics */
        let base_rule = self.rule();
        let (mut atoms, negated_atoms, cmprs): (Vec<_>, Vec<_>, Vec<_>) = {
            let mut atoms = Vec::new();
            let mut negated_atoms = Vec::new();
            let mut cmprs = Vec::new();
        
            for predicate in base_rule.rhs() {
                match predicate {
                    Predicate::AtomPredicate(_) => atoms.push(predicate.clone()),
                    Predicate::NegatedAtomPredicate(_) => negated_atoms.push(predicate.clone()),
                    Predicate::ComparePredicate(_) => cmprs.push(predicate.clone()),
                }
            }
        
            (atoms, negated_atoms, cmprs)
        };

        /* targets */
        let mut is_active_non_core_atom_bitmap = self.is_core_atom_bitmap().iter().map(|&is_core| !is_core).collect::<Vec<bool>>(); // for non-core atoms, initialize to true (active)
        let mut is_active_negation_bitmap = vec![true; self.negated_atom_names().len()];
        let core_ids = self.is_core_atom_bitmap().iter().enumerate().filter_map(|(i, &is_core)| if is_core { Some(i) } else { None }).collect::<Vec<usize>>();

        /* reduce the rule */
        let forward = self.reducer(
                &format!("sip{}f", rule_loc),
                base_rule, 
                &core_ids, 
                &mut atoms, 
                &negated_atoms, 
                &cmprs, 
                &mut is_active_non_core_atom_bitmap, 
                &mut is_active_negation_bitmap
            )
            .into_iter()
            .map(|rule| Catalog::from_strata(&rule))
            .collect::<Vec<Catalog>>();

        /* backward */
        let backward = self.reducer(
                &format!("sip{}b", rule_loc),
                base_rule, 
                &core_ids.into_iter().rev().collect(), 
                &mut atoms, 
                &negated_atoms, 
                &cmprs, 
                &mut is_active_non_core_atom_bitmap, 
                &mut is_active_negation_bitmap
            )
            .into_iter()
            .map(|rule| Catalog::from_strata(&rule))
            .collect::<Vec<Catalog>>();

        // construct the final rule (head :- core atoms, negated (active) atoms, comparisons)
        assert!(is_active_non_core_atom_bitmap.iter().all(|&x| !x));
        let final_head = base_rule.head().clone();
        let cores = atoms.into_iter().enumerate().filter_map(|(i, atom)| if self.is_core_atom_bitmap()[i] { Some(atom) } else { None }).collect::<Vec<Predicate>>();
        let active_neg = negated_atoms.into_iter().enumerate().filter_map(|(i, neg_atom)| if is_active_negation_bitmap[i] { Some(neg_atom) } else { None }).collect::<Vec<Predicate>>();
        let final_rhs = cores.into_iter().chain(active_neg.into_iter()).chain(cmprs.into_iter()).collect::<Vec<Predicate>>();

        let final_rule = FLRule::new(final_head, final_rhs, base_rule.is_planning(), base_rule.is_sip());
        debug!("\nfinal: {}", final_rule);

        // construct the final catalog by chaining forward, backward, and the final rule Catalog::from_strata(&final_rule)
        let mut sideways = forward;
        sideways.extend(backward);
        sideways.push(Catalog::from_strata(&final_rule));

        sideways
    }



    /* get vars_set for each comparison predicate */
    fn populate_comparison_predicates_vars_set(comparison_predicates: &Vec<ComparisonExpr>) -> Vec<HashSet<String>> {
        comparison_predicates
            .iter()
            .map(|comparison_expr| {
                comparison_expr
                    .vars_set()
                    .into_iter()
                    .cloned()
                    .collect::<HashSet<String>>()
            })
            .collect()
    }

    fn populate_argument_signatures(r: &FLRule) -> 
        (   HashMap<AtomArgumentSignature, String>, 
            Vec<String>, 
            Vec<Vec<AtomArgumentSignature>>, 
            Vec<String>, 
            Vec<Vec<AtomArgumentSignature>>,
            BaseFilters,
            Vec<ComparisonExpr>
        ) {
        let mut is_safe_set = HashSet::new();                                                         // verify if every argument_str is safe
        let mut signature_to_argument_str_map = HashMap::new();                      // map each rule atom signature to the variable string
        
        let mut atom_names = Vec::new();
        let mut atom_argument_signatures = Vec::new();                                 // vector of atoms, for each atom, a vector of rule argument signatures
        
        let mut negated_atom_names = Vec::new();
        let mut negated_atom_argument_signatures = Vec::new();

        // to filter and truncate for the atom in the plan
        let mut local_var_eq_map = HashMap::new(); // (1) arc(x, x), or x = y
        let mut local_var_first_occurence_map: HashMap<String, AtomArgumentSignature> = HashMap::new(); // to help construct the local_var_eq_map (map each var to the first occurrence)
        let mut local_const_map = HashMap::new(); // (2) arc(x, 1), or x = 1
        let mut local_placeholder_set = HashSet::new(); // (3) arc(x, _), or x = _

        let (positive_atoms, negated_atoms, comparison_predicates): (Vec<_>, Vec<_>, Vec<_>) = 
            r.rhs().iter().fold((Vec::new(), Vec::new(), Vec::new()), |(mut pos, mut neg, mut comp), p| {
                match p {
                    Predicate::AtomPredicate(atom) => pos.push(atom),
                    Predicate::NegatedAtomPredicate(atom) => neg.push(atom),
                    Predicate::ComparePredicate(expr) => comp.push(expr.clone()),
                }
                (pos, neg, comp)
            });

        // (i) populate the signatures of positive atoms
        for (rhs_id, atom) in positive_atoms.iter().enumerate() {
            atom_names.push(atom.name().to_owned());
            let mut atom_signatures = Vec::new();
            for (argument_id, argument) in atom.arguments().iter().enumerate() {
                let rule_argument_signature = 
                    AtomArgumentSignature::new(
                        AtomSignature::new(true, rhs_id),
                        argument_id,
                    );
                atom_signatures.push(rule_argument_signature);

                match argument {
                    AtomArg::Var(var) => {
                        is_safe_set.insert(var);
                        signature_to_argument_str_map
                            .insert(rule_argument_signature.clone(), var.to_string());
                        
                        if let Some(first_occurence) = local_var_first_occurence_map.get(var) {
                            // if the var is in the map, it is a local variable equality constraint
                            local_var_eq_map.insert(rule_argument_signature.clone(), first_occurence.clone());
                        } else {
                            // if the var is not in the map, it is the first occurrence
                            local_var_first_occurence_map.insert(var.to_string(), rule_argument_signature.clone());
                        }
                    },

                    AtomArg::Const(constant) => {
                        local_const_map.insert(rule_argument_signature, constant.to_owned());
                    },

                    AtomArg::Placeholder => {
                        local_placeholder_set.insert(rule_argument_signature.clone());
                    }
                }
            }
            atom_argument_signatures.push(atom_signatures);
            local_var_first_occurence_map.clear();
        }
        
        // (ii) populate the signatures of negated atoms
        for (neg_rhs_id, atom) in negated_atoms.iter().enumerate() {
            negated_atom_names.push(atom.name().to_owned());
            let mut negated_atom_signatures = Vec::new();
            for (argument_id, argument) in atom.arguments().iter().enumerate() {
                let rule_argument_signature = 
                    AtomArgumentSignature::new(
                        AtomSignature::new(false, neg_rhs_id),
                        argument_id,
                    );
                negated_atom_signatures.push(rule_argument_signature);

                match argument {
                    AtomArg::Var(var) => {
                        if is_safe_set.contains(var) {
                            signature_to_argument_str_map
                                .insert(rule_argument_signature.clone(), var.to_string());
                            
                            if let Some(first_occurence) = local_var_first_occurence_map.get(var) {
                                // if the var is in the map, it is a local variable equality constraint
                                local_var_eq_map.insert(rule_argument_signature.clone(), first_occurence.clone());
                            } else {
                                // if the var is not in the map, it is the first occurrence
                                local_var_first_occurence_map.insert(var.to_string(), rule_argument_signature.clone());
                                
                            }
                        } else {
                            panic!("unsafe var detected at negation !{} of rule {}", atom, r);
                        }
                    },

                    AtomArg::Const(constant) => {
                        local_const_map.insert(rule_argument_signature, constant.to_owned());
                    },

                    AtomArg::Placeholder => {
                        local_placeholder_set.insert(rule_argument_signature.clone());
                    }
                }
            }
            negated_atom_argument_signatures.push(negated_atom_signatures);
            local_var_first_occurence_map.clear();
        }

        (signature_to_argument_str_map, 
            atom_names, 
            atom_argument_signatures, 
            negated_atom_names, 
            negated_atom_argument_signatures,
            BaseFilters::new(local_var_eq_map, local_const_map, local_placeholder_set),
            comparison_predicates
        )
    }


    fn populate_is_core_atom_bitmap(
        signature_to_argument_str_map: &HashMap<AtomArgumentSignature, String>,
        atom_argument_signatures: &Vec<Vec<AtomArgumentSignature>>, /* only positive atoms */
    ) -> Vec<bool> {
        let mut is_core_atom_bitmap = vec![true; atom_argument_signatures.len()];

        let core_atom_argument_strs_set = atom_argument_signatures
            .iter()
            .map(|atom_argument_signatures| {
                atom_argument_signatures
                    .iter()
                    .filter_map(|signature| 
                        signature_to_argument_str_map.get(signature).cloned()
                    )
                    .collect::<HashSet<String>>()
            })
            .collect::<Vec<HashSet<String>>>();

        for (i, core_atom_argument_strs) in core_atom_argument_strs_set.iter().enumerate() {
            for (j, other_core_atom_argument_strs) in core_atom_argument_strs_set.iter().enumerate() {
                if i != j && core_atom_argument_strs.is_subset(other_core_atom_argument_strs) {
                    // if they are strictly equal, the larger index is not a core atom
                    if core_atom_argument_strs.len() < other_core_atom_argument_strs.len() {
                        // if other_core_atom_argument_strs is a strict superset
                        is_core_atom_bitmap[i] = false;
                    } else {
                        // if they are identical, the larger one is not a core atom
                        let larger = if i > j { i } else { j };
                        is_core_atom_bitmap[larger] = false;
                    }
                }
            }
        }
        is_core_atom_bitmap
    }
                
    fn populate_argument_presence_map(
        signature_to_argument_str_map: &HashMap<AtomArgumentSignature, String>,
        atom_argument_signatures: &Vec<Vec<AtomArgumentSignature>>, /* only positive atoms */
        base_filters: &BaseFilters,
    ) -> HashMap<String, Vec<Option<AtomArgumentSignature>>> {
        // map each argument_str to the first presence of the argument per atom (None if absent)
        // e.g. for rule tc(x, z) :- arc(x, y), tc(y, z), the map would be { x: [Some(0.0), None], y: [Some(0.1), Some(1.0)], z: [None, Some(1.1)] }
        let mut argument_presence_map: HashMap<String, Vec<Option<AtomArgumentSignature>>> = HashMap::new();

        for (rhs_id, argument_signatures) in atom_argument_signatures.iter().enumerate() {
            for argument_signature in argument_signatures {
                // skip if it is a base filter
                if base_filters.is_const_or_var_eq_or_placeholder(argument_signature) {
                    continue;
                }

                if let Some(variable) = signature_to_argument_str_map.get(argument_signature) {
                    // init the entry with a vector of None
                    let entry = argument_presence_map
                        .entry(variable.clone())
                        .or_insert(vec![None; atom_argument_signatures.len()]);

                    // if the rhs_id is None, set it to Some(argument_signature)
                    if entry[rhs_id].is_none() {
                        entry[rhs_id] = Some(argument_signature.clone());
                    }
                } else {
                    panic!("populate_argument_presence_map: argument signature {:?} absent from the signature map", argument_signature);
                }
            }
        }

        argument_presence_map
    }


    /* for each trace_argument_str, search for the first argument signature of it from the ordered list of positive atom_signatures (panic if not found) */
    pub fn top_down_trace(
        &self,
        trace_argument_strs: &[String],
        atom_signatures: &Vec<AtomSignature>,
    ) -> Vec<AtomArgumentSignature> {
        if let Some(non_positive) = atom_signatures.iter().find(|atom_signature| !atom_signature.is_positive()) {
            panic!("negated atom for top_down_trace: {:?}", non_positive);
        }

        let mut result = Vec::with_capacity(trace_argument_strs.len());

        let positive_rhs_ids = atom_signatures
            .iter()
            .map(|atom_signature| atom_signature.rhs_id())
            .collect::<Vec<usize>>();

        for trace_argument_str in trace_argument_strs {
            if let Some(presence_bitmap) = self.argument_presence_map.get(trace_argument_str) {
                if let Some(signature) = positive_rhs_ids
                    .iter()
                    .filter_map(|&positive_rhs_id| presence_bitmap.get(positive_rhs_id).and_then(|&opt| opt.clone()))
                    .next()
                {
                    result.push(signature);
                    continue;
                }
            } 
            
            panic!(
                "top_down_trace: argument_str {:?} absent from the presence map for positive atoms {:?}",
                trace_argument_strs, atom_signatures
            );
        }
        result
    }

    /* for each trace_expr (Arithmetic), convert to ArithmeticPos by looking up argument signatures */
    pub fn top_down_trace_exprs(
        &self,
        trace_exprs: &[Arithmetic],
        atom_signatures: &Vec<AtomSignature>,
    ) -> Vec<ArithmeticPos> {
        if let Some(non_positive) = atom_signatures.iter().find(|atom_signature| !atom_signature.is_positive()) {
            panic!("negated atom for top_down_trace_exprs: {:?}", non_positive);
        }

        trace_exprs
            .iter()
            .map(|arithmetic| {
                // Collect all variables in the arithmetic expression in order
                let var_names = arithmetic.variables_list_in_order();
                
                // Look up argument signatures for each variable using top_down_trace
                let var_signatures = self.top_down_trace(&var_names, atom_signatures);
                
                // Convert Arithmetic to ArithmeticPos using the signatures
                ArithmeticPos::from_arithmetic(arithmetic, &var_signatures)
            })
            .collect()
    }
    
    /* for each trace_argument_str, search for the first argument signature of it from a negative atom_signature (panic if not found) */
    pub fn top_down_trace_negated(
        &self,
        trace_argument_strs: &[String],
        negated_atom_signatures: &Vec<AtomSignature>, // only expect one negated atom
    ) -> Vec<AtomArgumentSignature> {
        assert_eq!(negated_atom_signatures.len(), 1);

        let negated_atom_signature = &negated_atom_signatures[0];
        if negated_atom_signature.is_positive() {
            panic!("positive atom for top_down_trace_negated: {:?}", negated_atom_signature);
        }

        let mut result = Vec::with_capacity(trace_argument_strs.len());

        let negated_atom_argument_signatures = &self.negated_atom_argument_signatures[negated_atom_signature.rhs_id()];

        for trace_argument_str in trace_argument_strs {
            for signature in negated_atom_argument_signatures {
                if let Some(argument_str) = self.signature_to_argument_str_map.get(signature) {
                    if argument_str == trace_argument_str {
                        result.push(signature.clone());
                        break;
                    }
                } else {
                    panic!("top_down_trace_negated: argument signature {:?} absent from the signature map", signature);
                }
            }
        }
        result
    }

    pub fn extract_head_arguments(
        head_args: &[HeadArg],
    ) -> (Vec<String>, Vec<Arithmetic>) {
        let mut head_vars_set = HashSet::new();
        let mut head_vars_list = Vec::new();
        let mut head_exprs = Vec::new();

        for head_arg in head_args {
            let arithmetic = match head_arg {
                HeadArg::Var(v) => Arithmetic::new(Factor::Var(v.clone()), vec![]),
                HeadArg::Arith(a) => a.clone(),
                HeadArg::Aggregation(a) => a.arithmetic().clone(),
            };

            // Collect variables in order (with duplicates)
            let var_names = arithmetic.variables_list_in_order();

            // Add unique variables to the head variables list
            for v in var_names {
                if head_vars_set.insert(v.clone()) {
                    head_vars_list.push(v);
                }
            }

            // Store the arithmetic expression directly
            head_exprs.push(arithmetic);
        }

        (head_vars_list, head_exprs)
    }
}


use std::fmt;

impl fmt::Display for Catalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Catalog for Rule: {}", self.rule())?;

        // Atom Names
        writeln!(f, "\nAtom Names (Core/Non-Core):")?;
        for (i, atom_name) in self.atom_names.iter().enumerate() {
            let status = if self.is_core_atom_bitmap[i] { "Core" } else { "Non-Core" };
            writeln!(f, "  - {} ({})", atom_name, status)?;
        }

        // Atom Argument Signatures
        writeln!(f, "\nAtom Argument Signatures:")?;
        for atom_signatures in &self.atom_argument_signatures {
            writeln!(f, "  Atom:")?;
            for signature in atom_signatures {
                writeln!(f, "    Argument Signature: {}", signature)?;
            }
        }

        // Negated Atom Names
        writeln!(f, "\nNegated Atom Names:")?;
        for negated_atom_name in &self.negated_atom_names {
            writeln!(f, "  - {}", negated_atom_name)?;
        }

        // Negated Atom Argument Signatures
        writeln!(f, "\nNegated Atom Argument Signatures:")?;
        for negated_signatures in &self.negated_atom_argument_signatures {
            writeln!(f, "  Negated Atom:")?;
            for signature in negated_signatures {
                writeln!(f, "    Argument Signature: {:?}", signature)?;
            }
        }

        // Signature to Argument String Map
        writeln!(f, "\nSignature to Argument String Map:")?;
        for (signature, arg_str) in &self.signature_to_argument_str_map {
            writeln!(f, "  {} -> {}", signature, arg_str)?;
        }

        // Argument Presence Map
        writeln!(f, "\nArgument Presence Map:")?;
        for (arg_str, presence_vec) in &self.argument_presence_map {
            write!(f, "  Argument {}: [", arg_str)?;
            let mut first = true;
            for presence in presence_vec {
                if !first {
                    write!(f, ", ")?;
                }
                match presence {
                    Some(signature) => write!(f, "{}", signature)?,
                    None => write!(f, "None")?,
                }
                first = false;
            }
            writeln!(f, "]")?;
        }

        // Base Filters
        writeln!(f, "\nBase Filters:")?;
        writeln!(f, "  {}", self.base_filters)?;

        Ok(())
    }
}
