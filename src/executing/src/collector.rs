use crate::aggregation::*;
use catalog::head::HeadIDB;
use macros::codegen_aggregation;
use planning::collections::CollectionSignature;
use reading::inspect::printsize_generic;
use reading::rel::{row_chop, Rel};
use reading::row::*;

use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::reduce::ReduceCore;
use differential_dataflow::trace::implementations::{ValBuilder, ValSpine};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use timely::order::TotalOrder;

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
        let idb_catalog = idb_catalogs
            .get(head_signature.name())
            .expect("couldn't find catalog metadata for idb head");
        if idb_catalog.is_aggregation() {
            let input_rel = build_concat_rel(head_signature, last_signatures, row_map, row_map);
            let aggregation = idb_catalog.aggregation();
            let output_rel = Arc::new(codegen_aggregation!());
            row_map.insert(Arc::clone(head_signature), output_rel);
        } else {
            let rel = build_concat_rel(head_signature, last_signatures, row_map, row_map);
            row_map.insert(Arc::clone(head_signature), rel);
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
        let idb_catalog = idb_catalogs
            .get(head_signature.name())
            .expect("couldn't find catalog metadata for idb head");
        if idb_catalog.is_aggregation() {
            let input_rel = build_concat_rel(
                head_signature,
                last_signatures,
                nest_row_map,
                variables_next_map,
            );
            let aggregation = idb_catalog.aggregation();
            let output_rel = Arc::new(codegen_aggregation!());
            variables_next_map.insert(Arc::clone(head_signature), output_rel);
        } else {
            let rel = build_concat_rel(
                head_signature,
                last_signatures,
                nest_row_map,
                variables_next_map,
            );
            variables_next_map.insert(Arc::clone(head_signature), rel);
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

/// Helper function: Given a head signature, create the relation that is the concatenation
/// of previous and newly discovered
fn build_concat_rel<G>(
    head_signature: &Arc<CollectionSignature>,
    last_signatures: &Vec<Arc<CollectionSignature>>,
    current_map: &HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    head_map: &HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
) -> Arc<Rel<G>>
where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    let relations = last_signatures
        .iter()
        .map(|signature| {
            Arc::clone(
                current_map
                    .get(signature)
                    .expect("last signature missing when concatenate"),
            )
        })
        .collect::<Vec<Arc<Rel<G>>>>();
    let rel = match head_map.get(head_signature) {
        Some(head_rel) => Arc::new(head_rel.concatenate(relations.into_iter())),
        None => {
            let mut iter = relations.into_iter();
            let first = iter.next().unwrap();
            Arc::new(first.concatenate(iter))
        }
    };
    rel
}
