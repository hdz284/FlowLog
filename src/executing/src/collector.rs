use crate::aggregation::*;
use catalog::head::HeadIDB;
use macros::{codegen_aggregation};
use planning::collections::CollectionSignature;
use reading::inspect::printsize_generic;
use reading::rel::{row_chop, Rel};

use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::reduce::ReduceCore;
use differential_dataflow::trace::implementations::{ValBuilder, ValSpine};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use timely::order::TotalOrder;

// Import for MIN optimization when using isize feature
#[cfg(feature = "isize-type")]
use macros::codegen_min_optimize;
#[cfg(feature = "isize-type")]
use parsing::aggregation::AggregationOperator;
#[cfg(feature = "isize-type")]
use reading::row::*;
#[cfg(feature = "isize-type")]
use differential_dataflow::operators::ThresholdTotal;
#[cfg(feature = "isize-type")]
use differential_dataflow::difference::IsZero;

pub fn non_recursive_collector<G>(
    last_signatures_map: &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,
    row_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    idb_catalogs: &HashMap<String, HeadIDB>,
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

        let concat_rels = last_signatures
            .iter()
            .skip(1)
            .map(|signature| { 
                Arc::clone(row_map.get(signature).expect("last signature missing when concatenate"))
            });

        let input_rel = match row_map.get(head_signature) {
            Some(head_rel) => {
                let full = std::iter::once(Arc::clone(init_rel)).chain(concat_rels);
                Arc::new(head_rel.concatenate(full))
            },
            None => {
                if last_signatures.len() == 1 {
                    Arc::clone(init_rel)
                } else {
                    Arc::new(init_rel.concatenate(concat_rels))
                }
            }
        };

        let idb_catalog = idb_catalogs
            .get(head_signature.name())
            .expect("couldn't find catalog metadata for idb head");
        if idb_catalog.is_aggregation() {
            let aggregation = idb_catalog.aggregation();
            
            // Check if we can use the optimized MIN aggregation path
            #[cfg(feature = "isize-type")]
            {
                if matches!(aggregation.operator(), AggregationOperator::Min) {
                    let output_rel = Arc::new(codegen_min_optimize!());
                    row_map.insert(Arc::clone(head_signature), output_rel);
                } else {
                    let output_rel = Arc::new(codegen_aggregation!());
                    row_map.insert(Arc::clone(head_signature), output_rel);
                }
            }
            
            // For non-isize features, use the standard aggregation path
            #[cfg(not(feature = "isize-type"))]
            {
                let output_rel = Arc::new(codegen_aggregation!());
                row_map.insert(Arc::clone(head_signature), output_rel);
            }
        } else {
            row_map.insert(Arc::clone(head_signature), input_rel);
        }
    }
}

pub fn recursive_collector<G>(
    last_signatures_map: &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,
    nest_row_map: &HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    variables_next_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    idb_catalogs: &HashMap<String, HeadIDB>,
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

        let concat_rels = last_signatures
            .iter()
            .skip(1)
            .map(|signature| { 
                Arc::clone(nest_row_map.get(signature).expect("last signature missing when concatenate"))
            });

        let input_rel = match variables_next_map.get(head_signature) {
            Some(head_rel) => {
                let full = std::iter::once(Arc::clone(init_rel)).chain(concat_rels);
                head_rel.concatenate(full).threshold()
            },
            None => {
                if last_signatures.len() == 1 {
                    init_rel.threshold()
                } else {
                    init_rel.concatenate(concat_rels).threshold()
                }
            }
        };

        let idb_catalog = idb_catalogs
            .get(head_signature.name())
            .expect("couldn't find catalog metadata for idb head");

        if idb_catalog.is_aggregation() {
            let aggregation = idb_catalog.aggregation();
            
            // Check if we can use the optimized MIN aggregation path
            #[cfg(feature = "isize-type")]
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
            
            // For non-isize features, use the standard aggregation path
            #[cfg(not(feature = "isize-type"))]
            {
                let output_rel = Arc::new(codegen_aggregation!());
                variables_next_map.insert(Arc::clone(head_signature), output_rel);
            }
        } else {
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
