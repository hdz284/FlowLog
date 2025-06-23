use paste::paste;
use std::rc::Rc; // reference counted pointer // differential dataflow trace implementation
use timely::order::TotalOrder;
use timely::dataflow::Scope;
use differential_dataflow::operators::arrange::Arranged;
use differential_dataflow::operators::arrange::TraceAgent;
use differential_dataflow::operators::ThresholdTotal;
use differential_dataflow::lattice::Lattice;
use differential_dataflow::Data;

use crate::Semiring;
use crate::semiring_one;
use crate::row::Row;
use crate::rel::Rel;
use crate::row::FatRow;


use differential_dataflow::trace::implementations::Vector;
use differential_dataflow::trace::implementations::ord_neu::OrdValBatch;
use differential_dataflow::trace::implementations::spine_fueled::Spine;
use timely::dataflow::ScopeParent;

/* ------------------------------------------------------------------------------------ */
/* Dict */
/* ------------------------------------------------------------------------------------ */
// Arranged<G, TraceAgent<Spine<Rc<OrdValBatch< Vector<((u32, u32), Product<(), u64>, Present)>> >>>>

pub type BatchDict<const K: usize, const V: usize, T, R> = ((Row<K>, Row<V>), T, R);
pub type VectorBatchDict<const K: usize, const V: usize, T, R> = Vector<BatchDict<K, V, T, R>>;
pub type DictTrace<const K: usize, const V: usize, T, R> = TraceAgent<Spine<Rc< OrdValBatch<VectorBatchDict<K, V, T, R>> >>>;

pub type ArrangedDictType<const K: usize, const V: usize, G, R> = Arranged<G, DictTrace<K, V, <G as ScopeParent>::Timestamp, R>>;

// Fat row arrangements for fallback
pub type BatchDictFat<T, R> = ((FatRow, FatRow), T, R);
pub type VectorBatchDictFat<T, R> = Vector<BatchDictFat<T, R>>;
pub type DictTraceFat<T, R> = TraceAgent<Spine<Rc<OrdValBatch<VectorBatchDictFat<T, R>>>>>;
pub type ArrangedDictTypeFat<G, R> = Arranged<G, DictTraceFat<<G as ScopeParent>::Timestamp, R>>;

macro_rules! impl_dicts {
    ($(($K:literal, $V:literal)),*) => {
        paste! {
            pub enum ArrangedDict<G: Scope> 
            where 
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                $(
                    [<ArrangedDict $K _ $V>](ArrangedDictType<$K, $V, G, Semiring>),
                )*
                // Fallback for large arities using FatRow
                ArrangedDictFat(ArrangedDictTypeFat<G, Semiring>, usize, usize), // Store K and V arities
            }

            impl<G: Scope> ArrangedDict<G>
            where
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                pub fn arity(&self) -> (usize, usize) {
                    match self {
                        $( ArrangedDict::[<ArrangedDict $K _ $V>](_) => ($K, $V), )*
                        ArrangedDict::ArrangedDictFat(_, k, v) => (*k, *v),
                    }
                }

                /// Check if this ArrangedDict uses FatRow (heap-allocated)
                pub fn is_fat(&self) -> bool {
                    matches!(self, ArrangedDict::ArrangedDictFat(_, _, _))
                }

                /// Check if this ArrangedDict uses fixed-size Row<N> (stack-allocated)
                pub fn is_thin(&self) -> bool {
                    !self.is_fat()
                }
            }

            impl<G: Scope> ArrangedDict<G>
            where
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                $(
                    pub fn [<dict_ $K _ $V>](&self) -> &ArrangedDictType<$K, $V, G, Semiring> {
                        match self {
                            ArrangedDict::[<ArrangedDict $K _ $V>](dict) => dict,
                            _ => panic!("panic access to dict of arity ({}, {})", $K, $V),
                        }
                    }
                )*

                pub fn dict_fat(&self) -> &ArrangedDictTypeFat<G, Semiring> {
                    match self {
                        ArrangedDict::ArrangedDictFat(dict, _, _) => dict,
                        _ => panic!("Cannot access fat dict on fixed-arity arrangement"),
                    }
                }
            }  
        }
    };
}

impl_dicts!(
    (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8),
    (2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8),
    (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8),
    (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6), (4, 7), (4, 8),
    (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (5, 7), (5, 8),
    (6, 1), (6, 2), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8),
    (7, 1), (7, 2), (7, 3), (7, 4), (7, 5), (7, 6), (7, 7), (7, 8),
    (8, 1), (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8)
);


// impl for 4 by 4
// impl_dicts!(
//     (1, 1), (1, 2), (1, 3), (1, 4),
//     (2, 1), (2, 2), (2, 3), (2, 4),
//     (3, 1), (3, 2), (3, 3), (3, 4),
//     (4, 1), (4, 2), (4, 3), (4, 4)
// );




/* ------------------------------------------------------------------------------------ */
/* Set */
/* ------------------------------------------------------------------------------------ */
// Arranged<G, TraceAgent<Spine<Rc<OrdKeyBatch< Vector<((Row<K>, ()), Product<(), u64>, Present)>> >>>>
use differential_dataflow::trace::implementations::ord_neu::OrdKeyBatch;
pub type BatchSet<const K: usize, T, R> = ((Row<K>, ()), T, R);
pub type VectorBatchSet<const K: usize, T, R> = Vector<BatchSet<K, T, R>>;
pub type SetTrace<const K: usize, T, R> = TraceAgent<Spine<Rc< OrdKeyBatch<VectorBatchSet<K, T, R>> >>>;
pub type ArrangedSetType<const K: usize, G, R> = Arranged<G, SetTrace<K, <G as ScopeParent>::Timestamp, R>>;

// Fat row set arrangements for fallback
pub type BatchSetFat<T, R> = ((FatRow, ()), T, R);
pub type VectorBatchSetFat<T, R> = Vector<BatchSetFat<T, R>>;
pub type SetTraceFat<T, R> = TraceAgent<Spine<Rc<OrdKeyBatch<VectorBatchSetFat<T, R>>>>>;
pub type ArrangedSetTypeFat<G, R> = Arranged<G, SetTraceFat<<G as ScopeParent>::Timestamp, R>>;

macro_rules! impl_sets {
    ($($K:literal),*) => {
        paste! {
            pub enum ArrangedSet<G: Scope> 
            where 
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                $( [<ArrangedSet $K>](ArrangedSetType<$K, G, Semiring>), )*
                // Fallback for large arities using FatRow
                ArrangedSetFat(ArrangedSetTypeFat<G, Semiring>, usize), // Store K arity
            }

            impl<G: Scope> ArrangedSet<G>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                pub fn arity(&self) -> usize {
                    match self {
                        $( ArrangedSet::[<ArrangedSet $K>](_) => $K, )*
                        ArrangedSet::ArrangedSetFat(_, k) => *k,
                    }
                }

                /// Check if this ArrangedSet uses FatRow (heap-allocated)
                pub fn is_fat(&self) -> bool {
                    matches!(self, ArrangedSet::ArrangedSetFat(_, _))
                }

                /// Check if this ArrangedSet uses fixed-size Row<N> (stack-allocated)
                pub fn is_thin(&self) -> bool {
                    !self.is_fat()
                }

                pub fn threshold(&self) -> Rel<G> {
                    if self.is_fat() {
                        // FatRow case
                        Rel::CollectionFat(
                            self.set_fat().threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one())),
                            self.arity()
                        )
                    } else {
                        // Fixed-size Row<N> case
                        match self {
                            $( ArrangedSet::[<ArrangedSet $K>](set) => Rel::[<Collection $K>](
                                set.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
                                ),
                            )*
                            ArrangedSet::ArrangedSetFat(_, _) => unreachable!("Fat case should be handled elsewhere"),
                        }
                    }
                }
            }

            impl<G: Scope> ArrangedSet<G>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                $(
                    pub fn [<set_ $K>](&self) -> &ArrangedSetType<$K, G, Semiring> {
                        match self {
                            ArrangedSet::[<ArrangedSet $K>](set) => set,
                            _ => panic!("panic access to set_{} of arity {}", $K, $K),
                        }
                    }
                )*

                pub fn set_fat(&self) -> &ArrangedSetTypeFat<G, Semiring> {
                    match self {
                        ArrangedSet::ArrangedSetFat(set, _) => set,
                        _ => panic!("Cannot access fat set on fixed-arity arrangement"),
                    }
                }
            }
        }
    };
}


impl_sets!(1, 2, 3, 4, 5, 6, 7, 8);