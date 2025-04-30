use std::sync::Arc;
use std::collections::{HashSet, HashMap};
use planning::collections::CollectionSignature;
use differential_dataflow::lattice::Lattice;
use timely::order::TotalOrder;
use itertools::Itertools;

use reading::rel::Rel;
use reading::inspect::printsize_generic;
// use reading::inspect::print_generic;

pub fn non_recursive_collector<G>(
    last_signatures_map: &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,
    row_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>
) where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice+TotalOrder,
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

        let head_rel = match row_map.get(head_signature) {
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

        row_map.insert(Arc::clone(head_signature), head_rel);
    }
}




pub fn recursive_collector<G>(
    last_signatures_map: &HashMap<Arc<CollectionSignature>, Vec<Arc<CollectionSignature>>>,
    nest_row_map: &HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    variables_next_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>
) where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice+TotalOrder,
{   
    for (head_signature, last_signatures) in last_signatures_map
        .iter()
        .sorted_by_key(|(signature, _)| signature.name()) 
    {
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

        let head_rel = match variables_next_map.get(head_signature) {
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

        variables_next_map.insert(Arc::clone(head_signature), Arc::new(head_rel));
    }
}



pub fn inspector<G>(
    head_signatures_set: &HashSet<Arc<CollectionSignature>>, 
    inspect_map: &mut HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>, 
    is_recursive: bool
) where
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    for head_signature in head_signatures_set
        .iter()
        .sorted_by_key(|signature| signature.name()) 
    {
        let entry = inspect_map.get_mut(head_signature)
            .expect(if is_recursive {
                "recursive head signature absent"
            } else {
                "non-recursive head signature absent"
            });

        *entry = Arc::new(entry.threshold());
        printsize_generic(entry, head_signature.name(), is_recursive);
        // print_generic(entry, head_signature.name());
    }
}