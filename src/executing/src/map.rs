use planning::constraints::BaseConstraints;
use std::sync::Arc;
use arrayvec::ArrayVec;
use planning::arguments::TransformationArgument;
use planning::flow::TransformationFlow;
use reading::row::Row;
use reading::row::Array;
use crate::compare::*;
use planning::compare::ComparisonExprArgument;


/* ------------------------------------------------------------------------ */
/* renders for map from row to row */
/* ------------------------------------------------------------------------ */
fn map_deconstructor<const N: usize>(args: &Arc<Vec<TransformationArgument>>) -> ArrayVec<usize, N> {
    args.iter().filter_map(|arg| match arg {
        TransformationArgument::KV((true, id)) => Some(*id),
        _ => None,
    }).collect::<ArrayVec<_, N>>()
}

fn const_eq_deconstructor(constraints: &BaseConstraints) -> Vec<(usize, i32)> {
    constraints.constant_eq_constraints().iter().filter_map(|(arg, constant)| match arg {
        TransformationArgument::KV((true, id)) => Some((*id, constant.integer())),
        _ => None,
    }).collect::<Vec<_>>()
}

fn var_eq_deconstructor(constraints: &BaseConstraints) -> Vec<(usize, usize)> {
    constraints.variable_eq_constraints().iter().filter_map(|(left, right)| match (left, right) {
        (TransformationArgument::KV((true, lid)), TransformationArgument::KV((true, rid))) => Some((*lid, *rid)),
        _ => None,
    }).collect::<Vec<_>>()
}

#[inline(always)]
fn is_filtered<const M: usize>(v: &Row<M>, const_eqs: &[(usize, i32)], var_eqs: &[(usize, usize)], compares: &Vec<ComparisonExprArgument>) -> bool {
    const_eqs.iter().all(|(i, constant)| v.column(*i) == *constant) && 
    var_eqs.iter().all(|(i, j)| v.column(*i) == v.column(*j)) &&
    compares.iter().all(|compare| compare_row(v, compare))
}

pub fn row_row<const M: usize, const N: usize>(flow: &TransformationFlow) -> impl FnMut(Row<M>) -> Option<Row<N>> {
    // for the single atom rule
    // assert!(!flow.is_constrainted());
    let k_or_v_ids = if let TransformationFlow::KVToKV { key, value, .. } = flow {
        assert!(key.is_empty() || value.is_empty());
        map_deconstructor::<N>(if key.is_empty() { value } else { key })
    } else {
        panic!("row_row: must be kv flow arguments");
    };

    assert_eq!(k_or_v_ids.len(), N, "vids arity ≠ row stack arity");

    let constraints = flow.constraints();
    let const_eqs = const_eq_deconstructor(constraints);
    let var_eqs = var_eq_deconstructor(constraints);
    let compares = flow.compares().clone();

    #[inline(always)]
    move |v| 
    if is_filtered(&v, &const_eqs, &var_eqs, &compares) {
        let mut row = Row::<N>::new();
        for id in &k_or_v_ids { row.push(v.column(*id)); }
        Some(row)
    } else {
        None
    }
}


/* ------------------------------------------------------------------------ */
/* renders for map from row to kv */
/* ------------------------------------------------------------------------ */
pub fn row_kv<const M: usize, const K: usize, const V: usize>(flow: &TransformationFlow) -> impl FnMut(Row<M>) -> Option<(Row<K>, Row<V>)> {
    // assert!(!flow.is_constrainted());
    let (kids, vids) = 
        if let TransformationFlow::KVToKV { key, value, .. } = flow {
            (map_deconstructor::<K>(key), map_deconstructor::<V>(value))
        } else {
            panic!("row_kv: must be a kv flow");
        };

    assert_eq!(kids.len(), K, "kids arity ≠ row stack arity");
    assert_eq!(vids.len(), V, "vids arity ≠ row stack arity");

    let constraints = flow.constraints();
    let const_eqs = const_eq_deconstructor(constraints);
    let var_eqs = var_eq_deconstructor(constraints);
    let compares = flow.compares().clone();

    #[inline(always)]
    move |v| 
    if is_filtered(&v, &const_eqs, &var_eqs, &compares) {
        let mut key = Row::<K>::new();
        let mut value = Row::<V>::new();
        for id in &kids { key.push(v.column(*id)); }
        for id in &vids { value.push(v.column(*id)); }

        Some((key, value))
    } else {
        None
    }
}


