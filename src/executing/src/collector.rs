use crate::aggregation::*;
use catalog::head::AggregationHeadIDB;
use macros::codegen_aggregation;
use macros::codegen_min_optimize;
use parsing::aggregation::AggregationOperator;
use planning::collections::CollectionSignature;
use reading::inspect::printsize_generic;
use reading::rel::{row_chop, Rel};
use reading::row::*;

use differential_dataflow::difference::IsZero;
use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::reduce::ReduceCore;
use differential_dataflow::operators::ThresholdTotal;
use differential_dataflow::trace::implementations::{ValBuilder, ValSpine};
use differential_dataflow::AsCollection;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use timely::dataflow::operators::Map;
use timely::order::TotalOrder;

pub fn non_recursive_collector<G>(
    last_signatures_map: &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,
    row_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    idb_catalogs: &HashMap<String, AggregationHeadIDB>,
) where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    for (head_signature, last_signatures) in last_signatures_map
        .iter()
        .sorted_by_key(|(signature, _)| signature.name())
    {
        let init_rel = last_signatures
            .first()
            .and_then(|signature| row_map.get(signature))
            .expect("Init relation missing");

        let concat_rels = last_signatures.iter().skip(1).map(|signature| {
            Arc::clone(
                row_map
                    .get(signature)
                    .expect("last signature missing when concatenate"),
            )
        });

        let input_rel = match row_map.get(head_signature) {
            Some(head_rel) => {
                let full = std::iter::once(Arc::clone(init_rel)).chain(concat_rels);
                Arc::new(head_rel.concatenate(full))
            }
            None => {
                if last_signatures.len() == 1 {
                    Arc::clone(init_rel)
                } else {
                    Arc::new(init_rel.concatenate(concat_rels))
                }
            }
        };

        // Check if this is an aggregation rule by looking it up in the aggregation catalog
        if let Some(idb_catalog) = idb_catalogs.get(head_signature.name()) {
            // This is an aggregation rule - use aggregation macros
            let aggregation = idb_catalog.aggregation();

            // MIN Semiring Optimization:
            // =========================
            // For MIN aggregations, we use a specialized semiring-based approach:
            //
            // Standard Aggregation Approach:
            // - Groups tuples by key using reduce_core
            // - Collects all values for each key into a vector
            // - Computes min across the entire vector
            //
            // MIN Semiring Approach:
            // - Uses differential dataflow's threshold_semigroup operator
            // - Leverages the Min semiring where:
            //   * Addition operation is min(a, b)
            //   * Zero element is infinity (u32::MAX)
            //   * Idempotent: min(a, a) = a
            // - Incrementally maintains minimum value per key in the difference
            //
            // Key Benefits: Leverages DD's built-in semiring support for efficiency
            // This optimization is only available in Present semiring.

            // Check if we can use the optimized MIN aggregation path
            #[cfg(not(feature = "isize-type"))]
            {
                if matches!(aggregation.operator(), AggregationOperator::Min) {
                    let output_rel = Arc::new(codegen_min_optimize!());
                    row_map.insert(Arc::clone(head_signature), output_rel);
                } else {
                    let output_rel = Arc::new(codegen_aggregation!());
                    row_map.insert(Arc::clone(head_signature), output_rel);
                }
            }

            // For isize-type feature, use the standard aggregation path
            // (Min semiring optimization requires Present-type semiring)
            #[cfg(feature = "isize-type")]
            {
                let output_rel = Arc::new(codegen_aggregation!());
                row_map.insert(Arc::clone(head_signature), output_rel);
            }
        } else {
            // This is a normal (non-aggregation) rule - use standard handling
            row_map.insert(Arc::clone(head_signature), input_rel);
        }
    }
}

pub fn recursive_collector<G>(
    last_signatures_map: &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,
    nest_row_map: &HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    variables_next_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    idb_catalogs: &HashMap<String, AggregationHeadIDB>,
) where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    for (head_signature, last_signatures) in last_signatures_map
        .iter()
        .sorted_by_key(|(signature, _)| signature.name())
    {
        // (sideways) jump over sip rules
        // We do not collect sip rules in the collector, we store them in the next row map
        // TODO: temporarily way to avoid sip rule, need carefully refactor
        // to avoid this in the future
        if head_signature.name().contains("_sip") {
            continue;
        }

        let init_rel = last_signatures
            .first()
            .and_then(|signature| nest_row_map.get(signature))
            .expect("init relation missing");

        let concat_rels = last_signatures.iter().skip(1).map(|signature| {
            Arc::clone(
                nest_row_map
                    .get(signature)
                    .expect("last signature missing when concatenate"),
            )
        });

        let input_rel = match variables_next_map.get(head_signature) {
            Some(head_rel) => {
                let full = std::iter::once(Arc::clone(init_rel)).chain(concat_rels);
                head_rel.concatenate(full).threshold()
            }
            None => {
                if last_signatures.len() == 1 {
                    init_rel.threshold()
                } else {
                    init_rel.concatenate(concat_rels).threshold()
                }
            }
        };

        // Check if this is an aggregation rule by looking it up in the aggregation catalog
        if let Some(idb_catalog) = idb_catalogs.get(head_signature.name()) {
            // This is an aggregation rule - use aggregation macros
            let aggregation = idb_catalog.aggregation();

            // Check if we can use the optimized MIN aggregation path
            #[cfg(not(feature = "isize-type"))]
            {
                if matches!(aggregation.operator(), AggregationOperator::Min) {
                    let input_rel = Arc::new(input_rel);
                    let output_rel = Arc::new(codegen_min_optimize!());
                    variables_next_map.insert(Arc::clone(head_signature), output_rel);
                } else {
                    let output_rel = Arc::new(codegen_aggregation!());
                    variables_next_map.insert(Arc::clone(head_signature), output_rel);
                }
            }

            // For isize-type feature, use the standard aggregation path
            #[cfg(feature = "isize-type")]
            {
                let output_rel = Arc::new(codegen_aggregation!());
                variables_next_map.insert(Arc::clone(head_signature), output_rel);
            }
        } else {
            // This is a normal (non-aggregation) rule - use standard handling
            variables_next_map.insert(Arc::clone(head_signature), Arc::new(input_rel));
        }
    }
}

pub fn inspector<G>(
    head_signatures_set: &HashSet<Arc<CollectionSignature>>,
    inspect_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    is_recursive: bool,
) where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    for head_signature in head_signatures_set
        .iter()
        .sorted_by_key(|signature| signature.name())
    {
        let entry = inspect_map.get_mut(head_signature).expect(if is_recursive {
            "recursive head signature absent"
        } else {
            "non-recursive head signature absent"
        });

        *entry = Arc::new(entry.threshold());
        printsize_generic(entry, head_signature.name(), is_recursive);
        // print_generic(entry, head_signature.name());
    }
}
