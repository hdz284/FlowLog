use paste::paste;
use std::sync::Arc;

use timely::order::TotalOrder;
use timely::dataflow::Scope;
use timely::dataflow::operators::Concatenate;
use timely::dataflow::scopes::Child;
use timely::progress::timestamp::Refines;
use timely::dataflow::ScopeParent;

use differential_dataflow::difference::Present;
use differential_dataflow::Data;
use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::arrange::ArrangeBySelf;
use differential_dataflow::operators::arrange::ArrangeByKey;
use differential_dataflow::Collection;
use differential_dataflow::operators::ThresholdTotal;
use differential_dataflow::AsCollection;
use differential_dataflow::operators::iterate::SemigroupVariable;

use crate::Semiring;
use crate::arrangements::ArrangedDict;

/* ------------------------------------------------------------------------------------ */
/* Rel (wrap over collections) */
/* ------------------------------------------------------------------------------------ */
use crate::arrangements::ArrangedSet;
use crate::row::Row;
use crate::row::Array;

#[inline(always)]
fn row_chop<const M: usize, const K: usize, const V: usize>() -> impl FnMut(Row<M>) -> (Row<K>, Row<V>) {
    move |v| {
        let mut key = Row::<K>::new();
        let mut value = Row::<V>::new();

        for i in 0..K { key.push(v.column(i)); }
        for i in K..M { value.push(v.column(i)); }

        (key, value)
    }
}

macro_rules! impl_rels {
    ($($arity:literal),*) => {
        paste! {
            pub enum Rel<G: Scope>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                $(
                    [<Collection $arity>](Collection<G, Row<$arity>, Semiring>),
                    [<Variable $arity>](SemigroupVariable<G, Row<$arity>, Semiring>),
                )*
            }

            impl<G: Scope> Rel<G>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                pub fn arity(&self) -> usize {
                    match self {
                        $( Rel::[<Collection $arity>](_) | Rel::[<Variable $arity>](_) => $arity, )*
                    }
                }

                $(
                    // rel_1, rel_2, ...,
                    pub fn [<rel_ $arity>](&self) -> &Collection<G, Row<$arity>, Semiring> {
                        match self {
                            Rel::[<Collection $arity>](rel) => rel,
                            Rel::[<Variable $arity>](var) => &*var,
                            _ => panic!("panic access to rel of arity {}", $arity),
                        }
                    }
                )*

                pub fn arrange_set(&self) -> ArrangedSet<G> {
                    match self.arity() {
                        $(
                            $arity => ArrangedSet::[<ArrangedSet $arity>](self.[<rel_ $arity>]().arrange_by_self()),
                        )*
                        _ => panic!("arity unimplemented {}", self.arity()),
                    }
                }

                

                pub fn concat(&self, other: &Rel<G>) -> Rel<G> {
                    assert_eq!(
                        self.arity(),
                        other.arity(),
                        "concat: self.arity = {}, other.arity = {}",
                        self.arity(),
                        other.arity()
                    );
                    match self.arity() {
                        $(
                            $arity => Rel::[<Collection $arity>](
                                self.[<rel_ $arity>]().concat(other.[<rel_ $arity>]())
                            ),
                        )*
                        _ => panic!("concat must have identical arity"),
                    }
                }

                /* 
                    pub fn negate(&self) -> Rel<G> {
                        match self.arity() {
                            $(
                                $arity => Rel::[<Collection $arity>](self.[<rel_ $arity>]().negate()),
                            )*
                            _ => panic!("arity unimplemented {}", self.arity()),
                        }
                    }
                */
                
                pub fn subtract(&self, other: &Rel<G>) -> Rel<G> {
                    assert_eq!(
                        self.arity(),
                        other.arity(),
                        "subtract: self.arity = {}, other.arity = {}",
                        self.arity(),
                        other.arity()
                    );
                    match self.arity() {
                        $(
                            $arity => Rel::[<Collection $arity>](
                                self.[<rel_ $arity>]()
                                    .expand(|x| Some((x, 1 as i32)))
                                    .concat(&other.[<rel_ $arity>]().expand(|x| Some((x, -1 as i32))))
                                    .threshold_semigroup(move |_, _, old| old.is_none().then_some(Present {}))
                            ),
                        )*
                        _ => panic!("subtract must have identical arity"),
                    }
                }
                        
                pub fn concatenate<I>(&self, others: I) -> Rel<G> 
                where
                    I: Iterator<Item = Arc<Rel<G>>>,    
                {
                    match self.arity() {
                        $(
                            $arity => {
                                let streams = others.into_iter().map(|other| match &*other {
                                    Rel::[<Collection $arity>](rel) => rel.inner.clone(),
                                    Rel::[<Variable $arity>](var) => var.inner.clone(),
                                    _ => panic!("`others` must have the identical arity as `self` when concatenate"),
                                });
                
                                Rel::[<Collection $arity>](self.[<rel_ $arity>]().inner.concatenate(streams).as_collection())
                            },
                        )*
                        _ => panic!("concatenate must have identical arity"),
                    }
                }
                
                pub fn threshold(&self) -> Rel<G> {
                    match self.arity() {
                        $(
                            $arity => Rel::[<Collection $arity>](
                                self.[<rel_ $arity>]().threshold_semigroup(move |_, _, old| old.is_none().then_some(Present {}))
                            ),
                        )*
                        _ => panic!("threshold unimplemented"),
                    }
                }

                pub fn enter<'a, T>(&self, child: &Child<'a, G, T>) -> Rel<Child<'a, G, T>>
                where
                    T: Refines<<G as ScopeParent>::Timestamp>+Lattice+TotalOrder,
                {
                    match self.arity() {
                        $(
                            $arity => Rel::[<Collection $arity>](
                                self.[<rel_ $arity>]().enter(child)
                            ),
                        )*
                        _ => panic!("enter unimplemented for arity {}", self.arity()),
                    }
                }

                pub fn set(self, result: &Rel<G>) -> Rel<G> {
                    assert_eq!(
                        self.arity(),
                        result.arity(),
                        "set: self.arity = {}, other.arity = {}",
                        self.arity(),
                        result.arity()
                    );
                    match self {
                        // must be a SemigroupVariable
                        $(
                            Rel::[<Variable $arity>](var) => {
                                Rel::[<Collection $arity>](
                                    var.set(result.[<rel_ $arity>]()),
                                )
                            },
                        )*
                        _ => panic!("set unimplemented for arity {}", self.arity()),
                    }
                }
            }
        }
    };
}


macro_rules! impl_leave {
    ($($arity:literal),*) => {
        paste! {
            impl<'a, G: Scope, T> Rel<Child<'a, G, T>>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
                T: Refines<<G as ScopeParent>::Timestamp>+Lattice+TotalOrder,
            {
                pub fn leave(&self) -> Rel<G> {
                    match self.arity() {
                        $(
                            $arity => Rel::[<Collection $arity>](
                                self.[<rel_ $arity>]().leave()
                            ),
                        )*
                        _ => panic!("leave unimplemented for arity {}", self.arity()),
                    }
                }
            }
        }
    };
}




impl_leave!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_rels!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);







macro_rules! impl_arranged_double {
    ($(($K:literal, $V:literal, $M:literal)),*) => {
        paste! {
            impl<G: Scope> Rel<G>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                // chop a row rel to a (k, v) rel
                pub fn arrange_double(&self, at: usize) -> DoubleRel<G> {
                    match (at, self.arity()) {
                        $(
                            ($K, $M) => {
                                DoubleRel::[<DoubleRel $K _ $V>](
                                    self.[<rel_ $M>]().map(row_chop::<$M, $K, $V>())
                                )
                            },
                        )*
                        _ => panic!("unimplemented"),
                    }
                }
            }
        }
    }
}

impl_arranged_double!(
    (1, 1, 2), //  2
    (1, 2, 3), (2, 1, 3), //  3
    (1, 3, 4), (2, 2, 4), (3, 1, 4), //  4
    (1, 4, 5), (2, 3, 5), (3, 2, 5), (4, 1, 5), //  5
    (1, 5, 6), (2, 4, 6), (3, 3, 6), (4, 2, 6), (5, 1, 6), //  6
    (1, 6, 7), (2, 5, 7), (3, 4, 7), (4, 3, 7), (5, 2, 7), (6, 1, 7), //  7
    (1, 7, 8), (2, 6, 8), (3, 5, 8), (4, 4, 8), (5, 3, 8), (6, 2, 8), (7, 1, 8) //  8
);





/* ------------------------------------------------------------------------------------ */
/* DoubleRel */
/* ------------------------------------------------------------------------------------ */

macro_rules! impl_double_rels {
    ($(($K:literal, $V:literal)),*) => {
        paste! {
            pub enum DoubleRel<G: Scope>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                $(
                    [<DoubleRel $K _ $V>](Collection<G, (Row<$K>, Row<$V>), Semiring>),
                )*
            }

            impl<G: Scope> DoubleRel<G>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                pub fn arity(&self) -> (usize, usize) {
                    match self {
                        $(
                            DoubleRel::[<DoubleRel $K _ $V>](_) => ($K, $V),
                        )*
                    }
                }

                pub fn arrange_dict(&self) -> ArrangedDict<G> {
                    match self {
                        $(
                            DoubleRel::[<DoubleRel $K _ $V>](rel) => ArrangedDict::[<ArrangedDict $K _ $V>](rel.arrange_by_key()),
                        )*
                    }
                }

                $(
                    // rel_1_1, rel_1_2, ...,
                    pub fn [<rel_ $K _ $V>](&self) -> &Collection<G, (Row<$K>, Row<$V>), Semiring> {
                        match self {
                            DoubleRel::[<DoubleRel $K _ $V>](rel) => rel,
                            _ => panic!("panic access to rel of arity ({}, {})", $K, $V),
                        }
                    }
                )*

                pub fn enter<'a, T>(&self, child: &Child<'a, G, T>) -> DoubleRel<Child<'a, G, T>>
                where
                    T: Refines<<G as ScopeParent>::Timestamp>+Lattice+TotalOrder,
                {
                    match self.arity() {
                        $(
                            ($K, $V) => DoubleRel::[<DoubleRel $K _ $V>](
                                self.[<rel_ $K _ $V>]().enter(child)
                            ),
                        )*
                        _ => panic!("enter unimplemented for arity {:?}", self.arity()),
                    }
                }
            }
        }
    };
}


impl_double_rels!(
    (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8),
    (2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8),
    (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8),
    (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6), (4, 7), (4, 8),
    (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (5, 7), (5, 8),
    (6, 1), (6, 2), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8),
    (7, 1), (7, 2), (7, 3), (7, 4), (7, 5), (7, 6), (7, 7), (7, 8),
    (8, 1), (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8)
);

