use parsing::aggregation::Aggregation;
use parsing::head::{Head, HeadArg};
use parsing::parser::Program;

use std::collections::HashMap;

/// Represents the structural analysis of a rule head in a logic programming context.
///
/// This structure identifies whether a head expression contains aggregation operations
/// and determines the type of aggregation pattern (simple aggregation vs group-by).
/// This analysis is crucial for query optimization and execution planning.
#[derive(Debug)]
pub struct HeadIDB {
    /// The name of the predicate (e.g., "person", "salary", "result")
    name: String,

    /// Optional aggregation argument (e.g., SUM, COUNT, MAX, etc.).
    /// If present, this indicates the head expression involves an aggregation operation.
    aggregation_argument: Option<Aggregation>,

    /// Flag indicating whether this head represents a group-by aggregation.
    /// True when there are non-aggregation arguments present alongside an aggregation.
    is_group_by: bool,

    /// The arity (total number of arguments) of the head expression.
    /// This includes both regular arguments and any aggregation argument.
    arity: usize,
}

impl HeadIDB {
    /// Constructs a HeadIDB from a Head expression by analyzing its structure.
    ///
    /// This method examines the head arguments to identify aggregation operations
    /// and determines whether the expression represents a group-by pattern.
    /// The analysis assumes aggregation arguments appear as the last argument.
    ///
    /// # Arguments
    /// * `head` - The Head expression to analyze
    ///
    /// # Returns
    /// A new HeadIDB instance with structural analysis results
    pub fn from_rule(head: &Head) -> Self {
        let head_args = head.head_arguments();

        // Check if the last argument is an aggregation
        let aggregation_argument = head_args.last().and_then(|arg| match arg {
            HeadArg::Aggregation(agg) => Some(agg.clone()),
            _ => None,
        });

        // Determine if this is a group-by operation:
        // - Must have an aggregation argument
        // - Must have at least one non-aggregation argument (for grouping)
        let is_group_by = aggregation_argument.is_some() && head_args.len() > 1;

        Self {
            name: head.name().clone(),
            aggregation_argument,
            is_group_by,
            arity: head_args.len(),
        }
    }

    /// Checks if this head expression contains an aggregation operation.
    ///
    /// # Returns
    /// `true` if an aggregation argument is present, `false` otherwise
    pub fn is_aggregation(&self) -> bool {
        self.aggregation_argument.is_some()
    }

    /// Returns the aggregation argument.
    ///
    /// # Returns
    /// The aggregation operation
    ///
    /// # Panics
    /// Panics if no aggregation argument is present. Use `is_aggregation()` to check first.
    pub fn aggregation(&self) -> Aggregation {
        self.aggregation_argument
            .clone()
            .expect("No aggregation argument present")
    }

    /// Checks if this head expression represents a group-by aggregation operation.
    ///
    /// A group-by operation occurs when there are both grouping variables (non-aggregation
    /// arguments) and an aggregation operation. This distinguishes between simple aggregations
    /// like "COUNT(*)" and grouped aggregations like "COUNT(*) GROUP BY department".
    ///
    /// # Returns
    /// `true` if this represents a group-by operation, `false` otherwise
    pub fn is_group_by(&self) -> bool {
        self.is_group_by
    }

    /// Returns a reference to the predicate name.
    ///
    /// # Returns
    /// A reference to the string containing the predicate name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the arity (total number of arguments) of the head expression.
    ///
    /// This includes both regular arguments and any aggregation argument that was present
    /// in the original head expression.
    ///
    /// # Returns
    /// The total number of arguments as a usize
    pub fn arity(&self) -> usize {
        self.arity
    }
}

/// Creates a catalog mapping predicate names to their IDB (Intensional Database) metadata.
///
/// This function analyzes a logic program and builds a catalog that maps each predicate name
/// to its corresponding HeadIDB analysis. This catalog is useful for query planning and
/// optimization, as it provides quick access to structural information about each predicate.
///
/// # Arguments
/// * `program` - The logic program to analyze
///
/// # Returns
/// A HashMap mapping predicate names to their HeadIDB analysis. If multiple rules define
/// the same predicate, only the first encountered rule's analysis is stored.
pub fn idb_catalog_from_program(program: &Program) -> HashMap<String, HeadIDB> {
    let mut idb_catalog: HashMap<String, HeadIDB> = HashMap::new();

    for rule in program.rules() {
        let predicate_name = rule.head().name();

        // Only insert if we haven't seen this predicate before
        // This preserves the "first rule wins" semantics
        if !idb_catalog.contains_key(predicate_name) {
            let head_idb = HeadIDB::from_rule(rule.head());
            idb_catalog.insert(predicate_name.clone(), head_idb);
        }
    }

    idb_catalog
}
