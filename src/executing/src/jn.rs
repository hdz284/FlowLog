use arrayvec::ArrayVec;
use std::sync::Arc;

// use parsing::rule::Const;
use planning::arguments::TransformationArgument;
// use planning::constraints::TransformationConstraints;
use planning::flow::TransformationFlow;
use reading::row::Array;
use reading::row::FatRow;
use reading::row::Row;

use crate::compare::jn_compare;

/* -------------------------------------------------------------------------------------------------------------------- */
/* renders for (k, v) jn (k, v) */
/* -------------------------------------------------------------------------------------------------------------------- */
fn jn_deconstructor<const N: usize>(
    args: &Arc<Vec<TransformationArgument>>,
) -> ArrayVec<(bool, bool, usize), N> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((l_or_r, k_or_v, id)) => Some((*l_or_r, *k_or_v, *id)),
            _ => None,
        })
        .collect::<ArrayVec<_, N>>()
}

#[inline(always)]
fn jn_extractor<const K: usize, const V: usize, const W: usize, const N: usize>(
    k: &Row<K>,
    v1: &Row<V>,
    v2: &Row<W>,
    extracts: &[(bool, bool, usize)],
) -> Row<N> {
    let mut row = Row::<N>::new();
    for &(l_or_r, k_or_v, id) in extracts {
        if !k_or_v {
            // from key
            row.push(k.column(id));
        } else {
            // from value
            if !l_or_r {
                row.push(v1.column(id)); // from left
            } else {
                row.push(v2.column(id)); // from right
            }
        }
    }
    row
}

/* (k, v) jn (k, v) → (k, v) */
/*                  → (k, ∅) */
/*                  → (∅, v) */
pub fn jn_logic<const K: usize, const V: usize, const W: usize, const N: usize>(
    flow: &TransformationFlow,
) -> impl FnMut(&Row<K>, &Row<V>, &Row<W>) -> Option<Row<N>> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        jn_deconstructor::<N>(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("jn row: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |k, v1, v2| {
        if compares
            .iter()
            .all(|compare| jn_compare(Some(k), Some(v1), Some(v2), compare))
        {
            Some(jn_extractor(k, v1, v2, &rids))
        } else {
            None
        }
    }
}

/* -------------------------------------------------------------------------------------------------------------------- */
/* renders for (∅, v) cartesian product (∅, v) */
/* -------------------------------------------------------------------------------------------------------------------- */
fn cartesian_deconstructor<const N: usize>(
    args: &Arc<Vec<TransformationArgument>>,
) -> ArrayVec<(bool, usize), N> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((l_or_r, true, id)) => Some((*l_or_r, *id)),
            _ => None,
        })
        .collect::<ArrayVec<_, N>>()
}

#[inline(always)]
fn cartesian_extractor<const V: usize, const W: usize, const N: usize>(
    v1: &Row<V>,
    v2: &Row<W>,
    extracts: &[(bool, usize)],
) -> Row<N> {
    let mut row = Row::<N>::new();
    for &(l_or_r, id) in extracts {
        // always from value
        if !l_or_r {
            row.push(v1.column(id)); // from left
        } else {
            row.push(v2.column(id)); // from right
        }
    }
    row
}

pub fn cartesian_logic<const V: usize, const W: usize, const N: usize>(
    flow: &TransformationFlow,
) -> impl FnMut(&(), &Row<V>, &Row<W>) -> Option<Row<N>> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        cartesian_deconstructor::<N>(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("cartesian: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |_, v1, v2| {
        if compares
            .iter()
            .all(|compare| jn_compare(None, Some(v1), Some(v2), compare))
        {
            Some(cartesian_extractor(v1, v2, &rids))
        } else {
            None
        }
    }
}

/* -------------------------------------------------------------------------------------------------------------------- */
/* renders for (k, v) jn (k, ∅) */
/* -------------------------------------------------------------------------------------------------------------------- */
fn v1_jn_deconstructor<const N: usize>(
    args: &Arc<Vec<TransformationArgument>>,
) -> ArrayVec<(bool, usize), N> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((l_or_r, k_or_v, id)) => {
                assert!((*l_or_r, *k_or_v) != (true, true)); // v2 = ∅
                Some((*k_or_v, *id))
            }
            _ => None,
        })
        .collect()
}

#[inline(always)]
fn v1_jn_extractor<const K: usize, const V: usize, const N: usize>(
    k: &Row<K>,
    v1: &Row<V>,
    extracts: &[(bool, usize)],
) -> Row<N> {
    // v1 -- always from left
    let mut row = Row::<N>::new();
    for &(k_or_v, id) in extracts {
        if !k_or_v {
            row.push(k.column(id)); // from key
        } else {
            row.push(v1.column(id)); // from value
        }
    }
    row
}

/* (k, v) jn (k, ∅) → (k, v) */
/*                  → (k, ∅) */
/*                  → (∅, v) */
pub fn v1_jn_logic<const K: usize, const V: usize, const N: usize>(
    flow: &TransformationFlow,
) -> impl FnMut(&Row<K>, &Row<V>, &()) -> Option<Row<N>> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v1_jn_deconstructor::<N>(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("jn_logic: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |k, v1, _| {
        if compares
            .iter()
            .all(|compare| jn_compare(Some(k), Some(v1), None, compare))
        {
            Some(v1_jn_extractor(k, v1, &rids))
        } else {
            None
        }
    }
}

/* -------------------------------------------------------------------------------------------------------------------- */
/* renders for (k, ∅) jn (k, ∅) */
/* -------------------------------------------------------------------------------------------------------------------- */
fn v2_jn_deconstructor<const N: usize>(
    args: &Arc<Vec<TransformationArgument>>,
) -> ArrayVec<usize, N> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((false, false, id)) => Some(*id), // must be a (left) key
            _ => None,
        })
        .collect()
}

#[inline(always)]
fn v2_jn_extractor<const K: usize, const N: usize>(k: &Row<K>, extracts: &[usize]) -> Row<N> {
    let mut row = Row::<N>::new();
    for &id in extracts {
        row.push(k.column(id));
    }
    row
}

/* (k, ∅) jn (k, ∅) → (k, v) */
/*                  → (k, ∅) */
/*                  → (∅, v) */
pub fn v2_jn_logic<const K: usize, const N: usize>(
    flow: &TransformationFlow,
) -> impl FnMut(&Row<K>, &(), &()) -> Option<Row<N>> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v2_jn_deconstructor::<N>(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("jn_logic: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |k, _, _| {
        if compares
            .iter()
            .all(|compare| jn_compare(Some(k), None, None, compare))
        {
            Some(v2_jn_extractor(k, &rids))
        } else {
            None
        }
    }
}

/* -------------------------------------------------------------------------------------------------------------------- */
/* negation as_collection */
pub fn aj_flatten<const K: usize, const V: usize, const N: usize>(
    flow: &TransformationFlow,
) -> impl FnMut(&Row<K>, &Row<V>) -> Row<N> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v1_jn_deconstructor::<N>(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("aj_flatten: must be a jn flow");
    };

    move |k, v| {
        let mut row = Row::<N>::new();
        for &(k_or_v, id) in &rids {
            if !k_or_v {
                row.push(k.column(id)); // from key
            } else {
                row.push(v.column(id)); // from value
            }
        }
        row
    }
}

pub fn v1_aj_flatten<const K: usize, const N: usize>(
    flow: &TransformationFlow,
) -> impl FnMut(&Row<K>, &()) -> Row<N> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v2_jn_deconstructor::<N>(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("v1_aj_flatten: must be a jn flow");
    };

    move |k, _| {
        let mut row = Row::<N>::new();
        for &id in &rids {
            row.push(k.column(id)); // from key
        }
        row
    }
}

/* -------------------------------------------------------------------------------------------------------------------- */
/* Fat mode versions for joins */
/* -------------------------------------------------------------------------------------------------------------------- */

fn jn_deconstructor_fat(args: &Arc<Vec<TransformationArgument>>) -> Vec<(bool, bool, usize)> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((l_or_r, k_or_v, id)) => Some((*l_or_r, *k_or_v, *id)),
            _ => None,
        })
        .collect::<Vec<_>>()
}

#[inline(always)]
fn jn_extractor_fat(
    k: &FatRow,
    v1: &FatRow,
    v2: &FatRow,
    extracts: &[(bool, bool, usize)],
) -> FatRow {
    let mut row = FatRow::new();
    for &(l_or_r, k_or_v, id) in extracts {
        if !k_or_v {
            // from key
            row.push(k.column(id));
        } else {
            // from value
            if !l_or_r {
                row.push(v1.column(id)); // from left
            } else {
                row.push(v2.column(id)); // from right
            }
        }
    }
    row
}

pub fn jn_logic_fat(
    flow: &TransformationFlow,
) -> impl FnMut(&FatRow, &FatRow, &FatRow) -> Option<FatRow> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        jn_deconstructor_fat(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("jn_logic_fat: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |k, v1, v2| {
        if compares
            .iter()
            .all(|compare| jn_compare(Some(k), Some(v1), Some(v2), compare))
        {
            Some(jn_extractor_fat(k, v1, v2, &rids))
        } else {
            None
        }
    }
}

/* Fat mode cartesian product */
fn cartesian_deconstructor_fat(args: &Arc<Vec<TransformationArgument>>) -> Vec<(bool, usize)> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((l_or_r, true, id)) => Some((*l_or_r, *id)),
            _ => None,
        })
        .collect::<Vec<_>>()
}

#[inline(always)]
fn cartesian_extractor_fat(v1: &FatRow, v2: &FatRow, extracts: &[(bool, usize)]) -> FatRow {
    let mut row = FatRow::new();
    for &(l_or_r, id) in extracts {
        // always from value
        if !l_or_r {
            row.push(v1.column(id)); // from left
        } else {
            row.push(v2.column(id)); // from right
        }
    }
    row
}

pub fn cartesian_logic_fat(
    flow: &TransformationFlow,
) -> impl FnMut(&(), &FatRow, &FatRow) -> Option<FatRow> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        cartesian_deconstructor_fat(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("cartesian_logic_fat: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |_, v1, v2| {
        if compares
            .iter()
            .all(|compare| jn_compare(None, Some(v1), Some(v2), compare))
        {
            Some(cartesian_extractor_fat(v1, v2, &rids))
        } else {
            None
        }
    }
}

/* Fat mode v1 join operations */
fn v1_jn_deconstructor_fat(args: &Arc<Vec<TransformationArgument>>) -> Vec<(bool, usize)> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((l_or_r, k_or_v, id)) => {
                assert!((*l_or_r, *k_or_v) != (true, true)); // v2 = ∅
                Some((*k_or_v, *id))
            }
            _ => None,
        })
        .collect()
}

#[inline(always)]
fn v1_jn_extractor_fat(k: &FatRow, v1: &FatRow, extracts: &[(bool, usize)]) -> FatRow {
    let mut row = FatRow::new();
    for &(k_or_v, id) in extracts {
        if !k_or_v {
            row.push(k.column(id)); // from key
        } else {
            row.push(v1.column(id)); // from value
        }
    }
    row
}

pub fn v1_jn_logic_fat(
    flow: &TransformationFlow,
) -> impl FnMut(&FatRow, &FatRow, &()) -> Option<FatRow> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v1_jn_deconstructor_fat(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("v1_jn_logic_fat: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |k, v1, _| {
        if compares
            .iter()
            .all(|compare| jn_compare(Some(k), Some(v1), None, compare))
        {
            Some(v1_jn_extractor_fat(k, v1, &rids))
        } else {
            None
        }
    }
}

/* Fat mode v2 join operations */
fn v2_jn_deconstructor_fat(args: &Arc<Vec<TransformationArgument>>) -> Vec<usize> {
    args.iter()
        .filter_map(|arg| match arg {
            TransformationArgument::Jn((false, false, id)) => Some(*id), // must be a (left) key
            _ => None,
        })
        .collect()
}

#[inline(always)]
fn v2_jn_extractor_fat(k: &FatRow, extracts: &[usize]) -> FatRow {
    let mut row = FatRow::new();
    for &id in extracts {
        row.push(k.column(id));
    }
    row
}

pub fn v2_jn_logic_fat(
    flow: &TransformationFlow,
) -> impl FnMut(&FatRow, &(), &()) -> Option<FatRow> {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v2_jn_deconstructor_fat(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("v2_jn_logic_fat: must be a jn flow");
    };
    let compares = flow.compares().clone();

    move |k, _, _| {
        if compares
            .iter()
            .all(|compare| jn_compare(Some(k), None, None, compare))
        {
            Some(v2_jn_extractor_fat(k, &rids))
        } else {
            None
        }
    }
}

/* Fat mode antijoin flatten operations */
pub fn aj_flatten_fat(flow: &TransformationFlow) -> impl FnMut(&FatRow, &FatRow) -> FatRow {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v1_jn_deconstructor_fat(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("aj_flatten_fat: must be a jn flow");
    };

    move |k, v| {
        let mut row = FatRow::new();
        for &(k_or_v, id) in &rids {
            if !k_or_v {
                row.push(k.column(id)); // from key
            } else {
                row.push(v.column(id)); // from value
            }
        }
        row
    }
}

pub fn v1_aj_flatten_fat(flow: &TransformationFlow) -> impl FnMut(&FatRow, &()) -> FatRow {
    let rids = if let TransformationFlow::JnToKV { key, value, .. } = flow {
        v2_jn_deconstructor_fat(&Arc::new(key.iter().chain(value.iter()).cloned().collect()))
    } else {
        panic!("v1_aj_flatten_fat: must be a jn flow");
    };

    move |k, _| {
        let mut row = FatRow::new();
        for &id in &rids {
            row.push(k.column(id)); // from key
        }
        row
    }
}
