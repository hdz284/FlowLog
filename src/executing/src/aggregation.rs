use parsing::aggregation::{Aggregation, AggregationOperator};
use reading::row::{Array, FatRow, Row};
use reading::{semiring_one, Semiring};

/// Aggregates a collection of integer values using the specified aggregation operator.
///
/// # Arguments
/// * `input` - Vector of integers to aggregate
/// * `op` - The aggregation operation to perform
///
/// # Returns
/// * `Some(result)` - The aggregation result, or the count for Count operations
/// * `None` - If the input is empty and the operation cannot produce a meaningful result
fn aggregate_ints(input: &[i32], op: &AggregationOperator) -> Option<i32> {
    match op {
        AggregationOperator::Count => Some(input.len() as i32),
        AggregationOperator::Sum => Some(input.iter().sum()),
        AggregationOperator::Min => input.iter().min().copied(),
        AggregationOperator::Max => input.iter().max().copied(),
    }
}

/// Creates the reduction logic for differential dataflow aggregation operations.
///
/// This function returns a closure that can be used with differential dataflow's
/// reduce operator to perform aggregations on grouped data.
///
/// # Type Parameters
/// * `N_GB` - Number of columns in the group-by key
/// * `N_TOT` - Total number of columns in the relation
///
/// # Arguments
/// * `aggregation` - The aggregation specification containing the operator
///
/// # Returns
/// A closure that implements the aggregation logic for differential dataflow
pub fn aggregation_reduce_logic<const N_GB: usize, const N_TOT: usize>(
    aggregation: &Aggregation,
) -> impl FnMut(
    &Row<N_GB>,
    &[(&i32, Semiring)],
    &mut Vec<(i32, Semiring)>,
    &mut Vec<(i32, Semiring)>,
) {
    let operator = aggregation.operator().clone();

    move |_key, input, output, _fuel| {
        // Extract values from input rows for aggregation
        let values: Vec<i32> = input.iter().map(|(value, _)| **value).collect();

        if let Some(result) = aggregate_ints(&values, &operator) {
            output.push((result, semiring_one()));
        }
    }
}

/// Creates a mapping function to merge key-value pairs back into a relation after aggregation.
///
/// This function reconstructs the full relation by combining the group-by key with
/// the aggregated value. The aggregated value is placed as the last column.
///
/// # Type Parameters
/// * `N_GB` - Number of columns in the group-by key
/// * `N_TOT` - Total number of columns in the output relation (should equal N_GB + 1)
///
/// # Returns
/// A closure that merges key-value pairs into complete rows
pub fn aggregation_merge_kv<const N_GB: usize, const N_TOT: usize>(
) -> impl Fn((Row<N_GB>, i32)) -> Row<N_TOT> {
    move |(key, value)| {
        let mut out_row = Row::<N_TOT>::new();

        // First, add all columns from the group-by key
        for i in 0..N_GB {
            out_row.push(key.column(i).clone());
        }

        // Then, add the aggregated value as the last column
        out_row.push(value);

        out_row
    }
}

/// Creates a mapping function to separate a relation into key-value pairs before aggregation.
///
/// This function splits each row into a group-by key (all columns except the last)
/// and the value to be aggregated (the last column).
///
/// # Type Parameters
/// * `N_GB` - Number of columns in the group-by key (N_TOT - 1)
/// * `N_TOT` - Total number of columns in the input relation
///
/// # Returns
/// A closure that separates rows into key-value pairs for aggregation
pub fn aggregation_separate_kv<const N_GB: usize, const N_TOT: usize>(
) -> impl Fn(Row<N_TOT>) -> (Row<N_GB>, i32) {
    move |row| {
        let mut group_by_row = Row::<N_GB>::new();

        // Extract the first N_GB columns as the group-by key
        for i in 0..N_GB {
            group_by_row.push(row.column(i).clone());
        }

        // Extract the last column as the value to aggregate
        let aggregate_value = row.column(N_GB).clone();

        (group_by_row, aggregate_value)
    }
}

// ============================================================================
// Fat Row Variants
// ============================================================================
// These functions provide the same aggregation logic but work with FatRow,
// which has dynamic arity instead of compile-time fixed arity.

/// Fat row version of aggregation reduce logic.
///
/// Similar to `aggregation_reduce_logic` but works with `FatRow` which supports
/// dynamic column counts determined at runtime.
///
/// # Arguments
/// * `aggregation` - The aggregation specification containing the operator
///
/// # Returns
/// A closure that implements the aggregation logic for differential dataflow
pub fn aggregation_reduce_logic_fat(
    aggregation: &Aggregation,
) -> impl FnMut(
    &FatRow,
    &[(&i32, Semiring)],
    &mut Vec<(i32, Semiring)>,
    &mut Vec<(i32, Semiring)>,
) {
    let operator = aggregation.operator().clone();

    move |_key, input, output, _fuel| {
        let values: Vec<i32> = input.iter().map(|(value, _)| **value).collect();

        if let Some(result) = aggregate_ints(&values, &operator) {
            output.push((result, semiring_one()));
        }
    }
}

/// Fat row version of key-value merging after aggregation.
///
/// Reconstructs a `FatRow` by appending the aggregated value to the group-by key.
/// The aggregated value is always placed as the last column.
///
/// # Returns
/// A closure that merges key-value pairs into complete fat rows
pub fn aggregation_merge_kv_fat() -> impl Fn((FatRow, i32)) -> FatRow {
    move |(key, value)| {
        let mut out_row = FatRow::new();

        // Copy all columns from the group-by key
        for i in 0..key.arity() {
            out_row.push(key.column(i).clone());
        }

        // Append the aggregated value as the last column
        out_row.push(value.clone());

        out_row
    }
}

/// Fat row version of relation separation into key-value pairs.
///
/// Splits a `FatRow` into group-by key (all columns except the last) and
/// the aggregation value (the last column).
///
/// # Returns
/// A closure that separates fat rows into key-value pairs for aggregation
pub fn aggregation_separate_kv_fat() -> impl Fn(FatRow) -> (FatRow, i32) {
    move |row| {
        let mut group_by_row = FatRow::new();

        let arity = row.arity();

        // Extract all columns except the last as the group-by key
        for i in 0..arity - 1 {
            group_by_row.push(row.column(i).clone());
        }

        // Extract the last column as the value to aggregate
        let aggregate_value = row.column(arity - 1).clone();

        (group_by_row, aggregate_value)
    }
}
