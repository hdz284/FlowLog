use parsing::aggregation::Aggregation;
use parsing::head::{Head, HeadArg};
use parsing::parser::Program;

use std::collections::HashMap;

/// Represents the structural analysis of an aggregation rule head in a logic programming context.
///
/// This structure specifically handles rule heads that contain aggregation operations.
#[derive(Debug)]
pub struct AggregationHeadIDB {
    /// The name of the predicate (e.g., "total_salary", "count_employees", "max_age")
    name: String,

    /// The aggregation argument (e.g., SUM, COUNT, MAX, etc.).
    /// This is always present since this struct only handles aggregation heads.
    aggregation_argument: Aggregation,

    /// Flag indicating whether this head represents a group-by aggregation.
    /// True when there are non-aggregation arguments present alongside the aggregation.
    is_group_by: bool,

    /// The arity (total number of arguments) of the head expression.
    /// This includes both regular arguments and the aggregation argument.
    arity: usize,
}

impl AggregationHeadIDB {
    /// Constructs an AggregationHeadIDB from a Head expression by analyzing its aggregation structure.
    ///
    /// This method examines the head arguments to identify aggregation operations
    /// and determines whether the expression represents a group-by pattern.
    /// The analysis assumes aggregation arguments appear as the last argument.
    ///
    /// # Arguments
    /// * `head` - The Head expression to analyze (must contain an aggregation)
    ///
    /// # Returns
    /// A new AggregationHeadIDB instance with structural analysis results
    ///
    /// # Panics
    /// Panics if the head does not contain an aggregation argument
    pub fn from_aggregation_rule(head: &Head) -> Self {
        let head_args = head.head_arguments();

        // Extract the aggregation argument (must be the last argument)
        let aggregation_argument = head_args
            .last()
            .and_then(|arg| match arg {
                HeadArg::Aggregation(agg) => Some(agg.clone()),
                _ => None,
            })
            .expect("Head must contain an aggregation argument");

        // Determine if this is a group-by operation:
        // - Has an aggregation argument (guaranteed by above)
        // - Has at least one non-aggregation argument (for grouping)
        let is_group_by = head_args.len() > 1;

        Self {
            name: head.name().clone(),
            aggregation_argument,
            is_group_by,
            arity: head_args.len(),
        }
    }

    /// Returns the aggregation argument.
    ///
    /// # Returns
    /// The aggregation operation (always present)
    pub fn aggregation(&self) -> &Aggregation {
        &self.aggregation_argument
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
    /// This includes both regular arguments and the aggregation argument that is always
    /// present in aggregation head expressions.
    ///
    /// # Returns
    /// The total number of arguments as a usize
    pub fn arity(&self) -> usize {
        self.arity
    }
}

/// Creates a catalog mapping predicate names to their aggregation IDB metadata.
///
/// This function analyzes a logic program and builds a catalog that maps each predicate name
/// to its corresponding AggregationHeadIDB analysis, but only for rules that contain aggregations.
/// Non-aggregation rules are ignored. This catalog is useful for query planning and
/// optimization of aggregation queries specifically.
///
/// # Arguments
/// * `program` - The logic program to analyze
///
/// # Returns
/// A HashMap mapping predicate names to their AggregationHeadIDB analysis. Only predicates
/// with aggregation operations are included. If multiple aggregation rules define the same
/// predicate, only the first encountered rule's analysis is stored.
pub fn aggregation_catalog_from_program(program: &Program) -> HashMap<String, AggregationHeadIDB> {
    let mut aggregation_catalog: HashMap<String, AggregationHeadIDB> = HashMap::new();

    for rule in program.rules() {
        let head = rule.head();
        let predicate_name = head.name();

        // Check if this head contains an aggregation
        let has_aggregation = head
            .head_arguments()
            .last()
            .map(|arg| matches!(arg, HeadArg::Aggregation(_)))
            .unwrap_or(false);

        // Only process rules with aggregation and only if we haven't seen this predicate before
        if has_aggregation && !aggregation_catalog.contains_key(predicate_name) {
            let aggregation_head_idb = AggregationHeadIDB::from_aggregation_rule(head);
            aggregation_catalog.insert(predicate_name.clone(), aggregation_head_idb);
        }
    }

    aggregation_catalog
}
