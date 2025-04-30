use std::sync::Arc;
use std::collections::HashMap;
use planning::flow::TransformationFlow;
use planning::collections::CollectionSignature;
use reading::rel::DoubleRel;
use reading::arrangements::ArrangedDict;
use reading::arrangements::ArrangedSet;
use reading::rel::Rel::*;
use reading::rel::Rel;

use differential_dataflow::lattice::Lattice;
use timely::order::TotalOrder;
use differential_dataflow::Data;
use macros::*;
use crate::jn::*;

pub fn cartesian<G>(
    large: &Arc<CollectionSignature>,
    small: &Arc<CollectionSignature>,
    row_map: &HashMap<Arc<CollectionSignature>, Arc<Rel<G>>>,
    iv0: usize,
    iv1: usize,
    target: usize,
    flow: &TransformationFlow
) -> Arc<Rel<G>> 
where 
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Data+Lattice+TotalOrder,
{
    use differential_dataflow::operators::arrange::ArrangeByKey;
    let rel_0 = row_map.get(large).expect("0 for cartesian");
    let rel_1 = row_map.get(small).expect("1 for cartesian");
    Arc::new(codegen_cartesian!())
}


pub fn kv_jn_kv<G>(
    large: &Arc<CollectionSignature>, 
    small: &Arc<CollectionSignature>, 
    kv_map: &HashMap<Arc<CollectionSignature>, (Arc<DoubleRel<G>>, Arc<ArrangedDict<G>>)>,
    ik0: usize,
    iv0: usize,
    iv1: usize,
    target: usize,
    flow: &TransformationFlow
) -> Arc<Rel<G>> 
where 
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Data+Lattice+TotalOrder,
{
    let (_, dict_0) = kv_map.get(large).expect("0 for kv jn kv");
    let (_, dict_1) = kv_map.get(small).expect("1 for kv jn kv");
    Arc::new(codegen_jn!())
}    


pub fn kv_jn_k<G>(
    large: &Arc<CollectionSignature>, 
    small: &Arc<CollectionSignature>, 
    kv_map: &HashMap<Arc<CollectionSignature>, (Arc<DoubleRel<G>>, Arc<ArrangedDict<G>>)>,
    k_map: &HashMap<Arc<CollectionSignature>, (Arc<Rel<G>>, Arc<ArrangedSet<G>>)>,
    ik0: usize,
    iv0: usize,
    iv1: usize,
    target: usize,
    flow: &TransformationFlow
) -> Arc<Rel<G>> 
where 
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Data+Lattice+TotalOrder,
{
    assert!(iv1 == 0);
    let (_, dict_0) = kv_map.get(large).expect("dict for kv jn k");
    let (_, set_1) = k_map.get(small).expect("set for kv jn k");
    Arc::new(codegen_kv_k_jn!())
}


pub fn k_jn_k<G>(
    large: &Arc<CollectionSignature>, 
    small: &Arc<CollectionSignature>, 
    k_map: &HashMap<Arc<CollectionSignature>, (Arc<Rel<G>>, Arc<ArrangedSet<G>>)>,
    ik0: usize,
    iv0: usize,
    iv1: usize,
    target: usize,
    flow: &TransformationFlow
) -> Arc<Rel<G>> 
where 
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Data+Lattice+TotalOrder,
{
    assert!(iv0 == 0 && iv1 == 0);
    let (_, set_0) = k_map.get(large).expect("0 for k jn k");
    let (_, set_1) = k_map.get(small).expect("1 for k jn k");
    Arc::new(codegen_k_k_jn!())
}


pub fn kv_aj_k<G>(
    large: &Arc<CollectionSignature>, 
    small: &Arc<CollectionSignature>, 
    kv_map: &HashMap<Arc<CollectionSignature>, (Arc<DoubleRel<G>>, Arc<ArrangedDict<G>>)>,
    k_map: &mut HashMap<Arc<CollectionSignature>, (Arc<Rel<G>>, Arc<ArrangedSet<G>>)>,
    ik0: usize,
    iv0: usize,
    iv1: usize,
    target: usize,
    flow: &TransformationFlow
) -> Arc<Rel<G>> 
where 
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Data+Lattice+TotalOrder,
{
    assert!(iv1 == 0);
    if let Some((rel_1, set_1)) = k_map.get_mut(small) {
        *rel_1 = Arc::new(set_1.threshold()); 
        *set_1 = Arc::new(rel_1.arrange_set());
    } else { 
        panic!("threshold for kv aj k"); 
    }
    let (_, dict_0) = kv_map.get(large).expect("dict for kv aj k");
    let (_, set_1) = k_map.get(small).expect("set for kv aj k");

    /* i32 version 
        let substract_rel = codegen_kv_k_jn!().negate();
        Arc::new(codegen_kv_flatten!().concat(&substract_rel))
    */

    /* boolean version */
    Arc::new(codegen_kv_flatten!().subtract(&codegen_kv_k_jn!()))
}


pub fn k_aj_k<G>(
    large: &Arc<CollectionSignature>, 
    small: &Arc<CollectionSignature>, 
    k_map: &mut HashMap<Arc<CollectionSignature>, (Arc<Rel<G>>, Arc<ArrangedSet<G>>)>,
    ik0: usize,
    iv0: usize,
    iv1: usize,
    target: usize,
    flow: &TransformationFlow
) -> Arc<Rel<G>> 
where 
    G: timely::dataflow::scopes::Scope,
    G::Timestamp: Data+Lattice+TotalOrder,
{
    assert!(iv0 == 0 && iv1 == 0);
    if let Some((rel_1, set_1)) = k_map.get_mut(small) {
        *rel_1 = Arc::new(set_1.threshold()); 
        *set_1 = Arc::new(rel_1.arrange_set());
    } else { 
        panic!("threshold for k aj k"); 
    }
    let (_, set_0) = k_map.get(large).expect("0 for k aj k");
    let (_, set_1) = k_map.get(small).expect("1 for k aj k");

    /* i32 version 
        let substract_rel = codegen_k_k_jn!().negate();
        Arc::new(codegen_k_flatten!().concat(&substract_rel))
    */

    /* boolean version */
    Arc::new(codegen_k_flatten!().subtract(&codegen_k_k_jn!()))
}
                                        