use std::fmt;
use std::{collections::{BinaryHeap, HashMap, HashSet}, vec};
use std::cmp::Reverse;
use catalog::rule::Catalog;
use tracing::debug;

// assuming no cross products for the joins

#[derive(Debug, Clone)]
pub struct PlanTree {
    root: usize,
    tree: HashMap<usize, Vec<usize>>,   // for each parent, return a vector of its children
    overlap: usize,
    max_overlap: usize,
    sub_trees: HashMap<usize, Vec<usize>>, // for each subroot, return a vector of the sub_tree in pre-order traversal order
    tree_width: usize,
}

impl PlanTree {
    pub fn root(&self) -> usize {
        self.root
    }

    pub fn tree(&self) -> &HashMap<usize, Vec<usize>> {
        &self.tree
    }

    pub fn sub_trees(&self) -> &HashMap<usize, Vec<usize>> {
        &self.sub_trees
    }

    pub fn is_acyclic(&self) -> bool {
        assert!(self.overlap <= self.max_overlap); // sanity check (cyclic queries have strictly smaller overlap than max_overlap)
        self.overlap == self.max_overlap
    }

    pub fn is_leaf(&self, x: usize) -> bool {
        self.tree.get(&x).unwrap().is_empty()
    }

    pub fn children(&self, x: usize) -> &Vec<usize> {
        self.tree.get(&x).unwrap()
    }

    pub fn tree_width(&self) -> usize {
        self.tree_width
    }
}

impl PlanTree {
    fn populate_subtree(
        subroot: usize,
        tree: &HashMap<usize, Vec<usize>>,
        sub_trees: &mut HashMap<usize, Vec<usize>>
    ) -> Vec<usize> {
        if let Some(subtree) = sub_trees.get(&subroot) {
            // return the already established subtree
            return subtree.clone();
        }
    
        let mut sub_tree = vec![subroot];
    
        // recurse and merge the subtrees of the children
        if let Some(children) = tree.get(&subroot) {
            for &child in children {
                let child_subtree = Self::populate_subtree(child, tree, sub_trees);
                sub_tree.extend(child_subtree); 
            }
        }

        sub_trees.insert(subroot, sub_tree.clone()); // cache the result in sub_trees
        
        sub_tree
    }

    pub fn from_catalog(catalog: &Catalog, is_optimized: bool) -> Self { 
        let atom_variable_sets = catalog
            .atom_argument_signatures()
            .iter()
            .zip(catalog.is_core_atom_bitmap().iter())
            .map(|(signatures, is_core)| {
                if *is_core {
                    catalog
                        .signature_to_argument_strs(signatures)
                        .iter()
                        .cloned()
                        .collect::<HashSet<_>>()
                } else {
                    HashSet::new() // we won't consider non-core atoms for the tree
                }
            })
            .collect::<Vec<_>>();

        // define a lambda that takes two usizes and returns the overlap of the corresponding atom_variable_sets
        let lambda_overlap = |from: usize, to: usize| -> usize {
            atom_variable_sets[from]
                .intersection(&atom_variable_sets[to])
                .count()
        };

        // return an optimized tree (using tree width)
        let head_variable_set = catalog
            .head_arguments_strs()
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        // return a default tree (essentially a chain) with the last atom as the root (say, 3 -> 2 -> 1 -> 0 -> [])
        // (first to last join order w/ only core atoms
        let core_atoms: Vec<usize> = catalog
            .is_core_atom_bitmap()
            .iter()
            .enumerate()
            .filter_map(|(i, &is_core)| if is_core { Some(i) } else { None })
            .collect();

        if core_atoms.is_empty() {
            panic!("No core atoms for the rule {}", catalog.rule());
        }
        
        let mut root = *core_atoms.last().unwrap();
        let mut tree: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut overlap = 0;
        for pair in core_atoms.windows(2).rev() {
            let parent = pair[1];
            let child = pair[0];
            // insert edge (parent -> child)
            tree.insert(parent, vec![child]);
            overlap += lambda_overlap(parent, child);
        }
    
        // the leaf in the chain -> []
        if let Some(&first) = core_atoms.first() {
            tree.insert(first, vec![]);
        }

        let mut width = Self::populate_tree_width(
            catalog,
            root,
            &tree,
            &atom_variable_sets,
            &head_variable_set,
        );

        let mut depth = Self::populate_tree_depth(root, &tree);

        if is_optimized {
            // iterate over each core atoms as roots                
            for candidate_root in 0..atom_variable_sets.len() {
                if !catalog.is_core_atom_bitmap()[candidate_root] {
                    continue; // skip non-core atoms
                }

                // Prim's algorithm with the candidate_root (for the maximum spanning tree)
                let mut visited = catalog.is_core_atom_bitmap().iter().map(|&is_core| !is_core).collect::<Vec<bool>>();

                // a map to trace nodes in the tree for easy access
                let mut candidate_tree: HashMap<usize, Vec<usize>> = HashMap::new();

                // a priority queue to trace the maximum overlap with the growing tree
                let mut max_heap = BinaryHeap::new();

                // root at depth 0, child of root will be depth - 1, and so on (so that the spanning tree can be bushy)
                max_heap.push((0, Reverse(0), usize::MAX, candidate_root)); 
                
                let mut candidate_overlap = 0;

                while let Some((prev_overlap, Reverse(child_depth), parent_id, child_id)) = max_heap.pop() {
                    if visited[child_id] {
                        // skip visited nodes (and non-core atoms)
                        continue; 
                    }

                    visited[child_id] = true;

                    if parent_id == usize::MAX {
                        // root node
                        candidate_tree.insert(child_id, vec![]);
                    } else {
                        // add the child to the parent (there must be one)
                        candidate_tree.get_mut(&parent_id).unwrap().push(child_id);
                        candidate_tree.insert(child_id, vec![]);
                        // debug!("insert edge ({} -> {})", parent_id, child_id);
                        candidate_overlap += prev_overlap;
                    }

                    // add all unvisited neighbors of the child to the priority queue
                    for neighbor_id in 0..atom_variable_sets.len() {
                        if !visited[neighbor_id] {
                            let next_overlap = lambda_overlap(child_id, neighbor_id);
                            if next_overlap > 0 {
                                max_heap.push((next_overlap, Reverse(child_depth + 1), child_id, neighbor_id)); // neighbor_depth = child_depth - 1
                            } else {
                                // no overlap between the root and the neighbor
                                max_heap.push((0, Reverse(1), candidate_root, neighbor_id)); 
                            }
                        }
                    }
                } // end of prim's algorithm

                // normalization via sorting to respect the default order
                for (_, children) in candidate_tree.iter_mut() {
                    children.sort_unstable();
                }

                // consider every permutation of the tree
                for candidate_tree in Self::tree_permutations(&candidate_tree) {
                    let candidate_width = Self::populate_tree_width(
                        catalog,
                        candidate_root,
                        &candidate_tree,
                        &atom_variable_sets,
                        &head_variable_set,
                    );

                    let candidate_depth = Self::populate_tree_depth(candidate_root, &candidate_tree);

                    if candidate_width < width || (candidate_width == width && candidate_depth < depth) {
                        debug!("newly optimized tree found w/ width {} and depth {}", candidate_width, candidate_depth);
                        tree = candidate_tree;
                        width = candidate_width;
                        depth = candidate_depth;
                        overlap = candidate_overlap;
                        root = candidate_root;
                    }
                }
            };
        }

        // populate the sub_trees HashMap for core atoms only
        let mut sub_trees: HashMap<usize, Vec<usize>> = HashMap::new();
        for x in 0..atom_variable_sets.len() {
            if catalog.is_core_atom_bitmap()[x] {
                Self::populate_subtree(x, &tree, &mut sub_trees);
            }
        }

        // max_overlap = sum of arities - number of distinct variables
        let num_distinct_variables = atom_variable_sets.iter().flatten().collect::<HashSet<_>>().len();

        Self {
            root,
            tree,
            overlap,
            max_overlap: atom_variable_sets.iter().map(|set| set.len()).sum::<usize>() - num_distinct_variables,
            sub_trees,
            tree_width: width,
        }
    }

    /// enumerate equivalent permutations of the tree
    fn tree_permutations(
        tree: &HashMap<usize, Vec<usize>>,
    ) -> Vec<HashMap<usize, Vec<usize>>> {
        use itertools::Itertools;
        let mut tree_permutations = vec![tree.clone()];

        for (parent, children) in tree {
            let mut new_permutations = vec![];
            for permuted_children in children.iter().copied().permutations(children.len()) {
                for tree in &tree_permutations {
                    let mut new_tree = tree.clone();
                    new_tree.insert(*parent, permuted_children.clone());
                    new_permutations.push(new_tree);
                }
            }
            tree_permutations = new_permutations;
        }

        tree_permutations
    }

    /// find the tree depth of the tree
    fn populate_tree_depth(
        root: usize,
        tree: &HashMap<usize, Vec<usize>>,
    ) -> usize {
        // define lambda for tree depth rooted at the parent
        fn subtree_depth(
            parent: usize,      // subtree root
            tree: &HashMap<usize, Vec<usize>>,  // ground truth
        ) -> usize {
            let children = tree.get(&parent).unwrap();
            if children.is_empty() {  return 0; } // base case (leaf)
            1 + children.iter().map(|&child| subtree_depth(child, tree)).max().unwrap()
        }

        // find the tree depth of the tree
        subtree_depth(root, tree)
    }


    /// find the tree width of the tree
    fn populate_tree_width(
        catalog: &Catalog,
        root: usize,
        tree: &HashMap<usize, Vec<usize>>,
        atom_variable_sets: &Vec<HashSet<String>>, 
        head_variables: &HashSet<String>,
    ) -> usize {
        // populate the sub_trees for core atoms only
        let mut sub_trees: HashMap<usize, Vec<usize>> = HashMap::new();
        for x in 0..atom_variable_sets.len() {
            if catalog.is_core_atom_bitmap()[x] {
                Self::populate_subtree(x, &tree, &mut sub_trees);
            }
        }

        // define lambda for tree width rooted at the parent
        fn subtree_width(
            catalog: &Catalog,
            parent: usize,      // subtree root
            tree: &HashMap<usize, Vec<usize>>,  // ground truth
            sub_trees: &HashMap<usize, Vec<usize>>, // ground truth for the tree
            head_variables: &HashSet<String>,
            _head_arity: usize,
        ) -> usize {
            let children = tree.get(&parent).unwrap();
            if children.is_empty() {  return 0; } // base case (leaf) instead of head_arity

            let planning_child = children.last().unwrap().clone();
            let planning_subtree = sub_trees.get(&planning_child).unwrap().clone();
            let leftover_subtrees = 
                std::iter::once(parent)
                    .chain(
                        children[..children.len() - 1]
                            .iter()
                            .flat_map(|&child| sub_trees.get(&child).unwrap().clone())
                    )
                    .collect::<Vec<usize>>();

            /* variables arguments for the both sides */
            let planning_vars_set = catalog.vars_set(&planning_subtree).into_iter().cloned().collect::<HashSet<_>>();
            let leftover_vars_set = catalog.vars_set(&leftover_subtrees).into_iter().cloned().collect::<HashSet<_>>();
            
            // planning_vars_set intersects (leftover_variables union head_variables)
            let planning_head_variables = 
                planning_vars_set
                    .intersection(
                        &leftover_vars_set.union(head_variables).cloned().collect()
                    )
                    .cloned()
                    .collect::<HashSet<String>>();
            
            // leftover_vars_set intersects (planning_variables union head_variables)
            let leftover_head_variables = 
                leftover_vars_set
                    .intersection(
                        &planning_vars_set.union(head_variables).cloned().collect()
                    )
                    .cloned()
                    .collect::<HashSet<String>>();

            // truncate the planning child from the tree 
            let mut truncated_tree = tree.clone();
            truncated_tree.get_mut(&parent).unwrap().pop(); 

            let planning_width = subtree_width(catalog, planning_child, &truncated_tree, sub_trees, &planning_head_variables, planning_head_variables.len());
            let leftover_width = subtree_width(catalog, parent, &truncated_tree, sub_trees, &leftover_head_variables, leftover_head_variables.len());

            std::cmp::max(
                std::cmp::max(planning_width, leftover_width),
                // head_arity // max intermediate arity
                // (alternative: max intermediate jn arity) planning_head_variables.union(&leftover_head_variables).count()
                planning_head_variables.union(&leftover_head_variables).count()
            )
        }

        // find the tree width of the tree
        subtree_width(catalog, root, tree, &sub_trees, head_variables, 0)
    }
}

// display the tree in a human-readable format
impl fmt::Display for PlanTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_tree(
            f: &mut fmt::Formatter<'_>,
            tree: &HashMap<usize, Vec<usize>>,
            current_node: usize,
            prefix: &str,
            is_last: bool,
        ) -> fmt::Result {
            // print the current node
            writeln!(
                f,
                "{}{}{}",
                prefix,
                if is_last { "└── " } else { "├── " },
                current_node
            )?;
            
            // determine the new prefix for child nodes
            let new_prefix = format!(
                "{}{}",
                prefix,
                if is_last { "    " } else { "│   " }
            );
            
            // retrieve and iterate over child nodes
            if let Some(children) = tree.get(&current_node) {
                let len = children.len();
                for (i, child) in children.iter().enumerate() {
                    let is_last_child = i == len - 1;
                    print_tree(f, tree, *child, &new_prefix, is_last_child)?;
                }
            }

            Ok(())
        }

        // print metadata
        // writeln!(f, "Plan Tree w/ root {}", self.root)?;
        // writeln!(f, "Overlap / Max Overlap: {} / {}", self.overlap, self.max_overlap)?;
        // start printing the tree from the root node
        print_tree(f, &self.tree, self.root, "", true)
    }
}