use paste::paste;
use std::sync::Arc;

use timely::dataflow::operators::Concatenate;
use timely::dataflow::scopes::Child;
use timely::dataflow::Scope;
use timely::dataflow::ScopeParent;
use timely::order::TotalOrder;
use timely::progress::timestamp::Refines;

use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::arrange::ArrangeByKey;
use differential_dataflow::operators::arrange::ArrangeBySelf;
use differential_dataflow::operators::iterate::SemigroupVariable;
use differential_dataflow::operators::ThresholdTotal;
use differential_dataflow::AsCollection;
use differential_dataflow::Collection;
use differential_dataflow::Data;

use crate::arrangements::ArrangedDict;
use crate::semiring_one;
use crate::Semiring;

/* ------------------------------------------------------------------------------------ */
/* Fat support for fallback when arity exceeds MAX */
/* ------------------------------------------------------------------------------------ */
use crate::row::FatRow;

/* ------------------------------------------------------------------------------------ */
/* Rel (wrap over collections) */
/* ------------------------------------------------------------------------------------ */
use crate::arrangements::ArrangedSet;
use crate::row::Array;
use crate::row::Row;

#[inline(always)]
pub fn row_chop<const M: usize, const K: usize, const V: usize>(
) -> impl FnMut(Row<M>) -> (Row<K>, Row<V>) {
    move |v| {
        let mut key = Row::<K>::new();
        let mut value = Row::<V>::new();

        for i in 0..K {
            key.push(v.column(i));
        }
        for i in K..M {
            value.push(v.column(i));
        }
        (key, value)
    }
}

#[inline(always)]
fn fat_row_chop(k_arity: usize, total_arity: usize) -> impl FnMut(FatRow) -> (FatRow, FatRow) {
    move |v| {
        let mut key = FatRow::new();
        let mut value = FatRow::new();

        for i in 0..k_arity {
            key.push(v.column(i));
        }
        for i in k_arity..total_arity {
            value.push(v.column(i));
        }

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
                // fallback for large arities that store true arity
                CollectionFat(Collection<G, FatRow, Semiring>, usize),
                VariableFat(SemigroupVariable<G, FatRow, Semiring>, usize),
            }

            impl<G: Scope> Rel<G>
            where
                G: timely::dataflow::scopes::Scope,
                G::Timestamp: Data+Lattice+TotalOrder,
            {
                pub fn arity(&self) -> usize {
                    match self {
                        $( Rel::[<Collection $arity>](_) | Rel::[<Variable $arity>](_) => $arity, )*
                        Rel::CollectionFat(_, arity) => *arity,
                        Rel::VariableFat(_, arity) => *arity,
                    }
                }

                /// Check if this Rel is fat
                pub fn is_fat(&self) -> bool {
                    matches!(self, Rel::CollectionFat(_, _) | Rel::VariableFat(_, _))
                }

                pub fn is_thin(&self) -> bool {
                    !self.is_fat()
                }

                $(
                    // deref for rel_1, rel_2, ...,
                    pub fn [<rel_ $arity>](&self) -> &Collection<G, Row<$arity>, Semiring> {
                        match self {
                            Rel::[<Collection $arity>](rel) => rel,
                            Rel::[<Variable $arity>](var) => &*var,
                            _ => panic!("panic access to rel of arity {}", $arity),
                        }
                    }
                )*

                // deref for Fat rel
                pub fn rel_fat(&self) -> &Collection<G, FatRow, Semiring> {
                    match self {
                        Rel::CollectionFat(rel, _) => rel,
                        Rel::VariableFat(var, _) => &*var,
                        _ => panic!("cannot access fat rel on fixed-arity collection"),
                    }
                }

                pub fn arrange_set(&self) -> ArrangedSet<G> {
                    if self.is_fat() {
                        // fat case
                        ArrangedSet::ArrangedSetFat(
                            self.rel_fat().arrange_by_self(),
                            self.arity()
                        )
                    } else {
                        let arity = self.arity();
                        match arity {
                            $(
                                $arity => ArrangedSet::[<ArrangedSet $arity>](self.[<rel_ $arity>]().arrange_by_self()),
                            )*
                            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
                        }
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

                    assert_eq!(
                        self.is_fat(),
                        other.is_fat(),
                        "concat: both rels must have the same row type (fat or thin)"
                    );

                    if self.is_fat() {
                        Rel::CollectionFat(
                            self.rel_fat().concat(other.rel_fat()),
                            self.arity()
                        )
                    } else {
                        match self.arity() {
                            $(
                                $arity => Rel::[<Collection $arity>](
                                    self.[<rel_ $arity>]().concat(other.[<rel_ $arity>]())
                                ),
                            )*
                            _ => unreachable!("concat: arity {} overflow", self.arity()),
                        }
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

                    assert_eq!(
                        self.is_fat(),
                        other.is_fat(),
                        "subtract: both rels must have the same row type (fat or thin)"
                    );

                    if self.is_fat() {
                        Rel::CollectionFat(
                            self.rel_fat()
                                .lift(|x| Some((x, 1 as i32)))
                                .concat(&other.rel_fat().lift(|x| Some((x, -1 as i32))))
                                .threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one())),
                            self.arity()
                        )
                    } else {
                        match self.arity() {
                            $(
                                $arity => Rel::[<Collection $arity>](
                                    self.[<rel_ $arity>]()
                                        .lift(|x| Some((x, 1 as i32)))
                                        .concat(&other.[<rel_ $arity>]().lift(|x| Some((x, -1 as i32))))
                                        .threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
                                ),
                            )*
                            _ => unreachable!("subtract: arity {} overflow", self.arity()),
                        }
                    }
                }

                pub fn concatenate<I>(&self, others: I) -> Rel<G>
                where
                    I: Iterator<Item = Arc<Rel<G>>>,
                {
                    if self.is_fat() {
                        let streams = others.into_iter().map(|other| match &*other {
                            Rel::CollectionFat(rel, _) => rel.inner.clone(),
                            Rel::VariableFat(var, _) => var.inner.clone(),
                            _ => panic!("`others` must have the identical row type as `self` when concatenate"),
                        });

                        Rel::CollectionFat(
                            self.rel_fat().inner.concatenate(streams).as_collection(),
                            self.arity()
                        )
                    } else {
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
                            _ => unreachable!("concatenate: arity {} overflows", self.arity()),
                        }
                    }
                }

                pub fn threshold(&self) -> Rel<G> {
                    if self.is_fat() {
                        Rel::CollectionFat(
                            self.rel_fat().threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one())),
                            self.arity()
                        )
                    } else {
                        match self.arity() {
                            $(
                                $arity => Rel::[<Collection $arity>](
                                    self.[<rel_ $arity>]().threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
                                ),
                            )*
                            _ => unreachable!("threshold: arity {} should be handled by fixed-size variants", self.arity()),
                        }
                    }
                }

                pub fn enter<'a, T>(&self, child: &Child<'a, G, T>) -> Rel<Child<'a, G, T>>
                where
                    T: Refines<<G as ScopeParent>::Timestamp>+Lattice+TotalOrder,
                {
                    if self.is_fat() {
                        Rel::CollectionFat(
                            self.rel_fat().enter(child),
                            self.arity()
                        )
                    } else {
                        match self.arity() {
                            $(
                                $arity => Rel::[<Collection $arity>](
                                    self.[<rel_ $arity>]().enter(child)
                                ),
                            )*
                            _ => unreachable!("enter: arity {} overflows", self.arity()),
                        }
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

                    assert_eq!(
                        self.is_fat(),
                        result.is_fat(),
                        "set: both rels must have the same row type (fat or thin)"
                    );

                    if self.is_fat() {
                        match self {
                            Rel::VariableFat(var, arity) => {
                                Rel::CollectionFat(
                                    var.set(result.rel_fat()),
                                    arity
                                )
                            },
                            _ => panic!("set: self must be a Variable for fat case"),
                        }
                    } else {
                        match self {
                            $(
                                Rel::[<Variable $arity>](var) => {
                                    Rel::[<Collection $arity>](
                                        var.set(result.[<rel_ $arity>]()),
                                    )
                                },
                            )*
                            _ => panic!("set: self must be a Variable for thin case"),
                        }
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
                    if self.is_fat() {
                        let arity = self.arity();
                        Rel::CollectionFat(
                            self.rel_fat().leave(),
                            arity
                        )
                    } else {
                        match self.arity() {
                            $(
                                $arity => Rel::[<Collection $arity>](
                                    self.[<rel_ $arity>]().leave()
                                ),
                            )*
                            _ => unreachable!("leave: arity {} overflows", self.arity()),
                        }
                    }
                }
            }
        }
    };
}

impl_leave!(1, 2, 3, 4, 5, 6, 7, 8);
impl_rels!(0, 1, 2, 3, 4, 5, 6, 7, 8);

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
                    if self.is_fat() {
                        DoubleRel::DoubleRelFat(
                            self.rel_fat().map(fat_row_chop(at, self.arity())),
                            at,
                            self.arity() - at
                        )
                    } else {
                        match (at, self.arity()) {
                            $(
                                ($K, $M) => {
                                    DoubleRel::[<DoubleRel $K _ $V>](
                                        self.[<rel_ $M>]().map(row_chop::<$M, $K, $V>())
                                    )
                                },
                            )*
                            _ => panic!("arrange_double: unsupported arity combination (at: {}, arity: {}) for fixed-size variants", at, self.arity()),
                        }
                    }
                }
            }
        }
    }
}

impl_arranged_double!(
    (1, 1, 2), //  2
    (1, 2, 3),
    (2, 1, 3), //  3
    (1, 3, 4),
    (2, 2, 4),
    (3, 1, 4), //  4
    (1, 4, 5),
    (2, 3, 5),
    (3, 2, 5),
    (4, 1, 5), //  5
    (1, 5, 6),
    (2, 4, 6),
    (3, 3, 6),
    (4, 2, 6),
    (5, 1, 6), //  6
    (1, 6, 7),
    (2, 5, 7),
    (3, 4, 7),
    (4, 3, 7),
    (5, 2, 7),
    (6, 1, 7), //  7
    (1, 7, 8),
    (2, 6, 8),
    (3, 5, 8),
    (4, 4, 8),
    (5, 3, 8),
    (6, 2, 8),
    (7, 1, 8) //  8
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
                DoubleRelFat(Collection<G, (FatRow, FatRow), Semiring>, usize, usize), // (collection, key_arity, value_arity)
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
                        DoubleRel::DoubleRelFat(_, k_arity, v_arity) => (*k_arity, *v_arity),
                    }
                }

                /// Concatenate two DoubleRels with the same arity
                pub fn concatenate<I>(&self, others: I) -> DoubleRel<G>
                where
                    I: Iterator<Item = Arc<DoubleRel<G>>>,
                {
                    match self.arity() {
                        $(
                            ($K, $V) => {
                                let streams = others.into_iter().map(|other| match &*other {
                                    DoubleRel::[<DoubleRel $K _ $V>](rel) => rel.inner.clone(),
                                    _ => panic!("`others` must have the identical arity as `self` when concatenate"),
                                });

                                DoubleRel::[<DoubleRel $K _ $V>](self.[<rel_ $K _ $V>]().inner.concatenate(streams).as_collection())
                            },
                        )*
                        _ => panic!("concatenate must have identical arity"),
                    }
                }

                /// Check if this DoubleRel uses FatRow (heap-allocated)
                pub fn is_fat(&self) -> bool {
                    matches!(self, DoubleRel::DoubleRelFat(_, _, _))
                }

                pub fn thin(&self) -> bool {
                    !self.is_fat()
                }

                pub fn arrange_dict(&self) -> ArrangedDict<G> {
                    if self.is_fat() {
                        let (k_arity, v_arity) = self.arity();
                        ArrangedDict::ArrangedDictFat(self.rel_fat().arrange_by_key(), k_arity, v_arity)
                    } else {
                        match self {
                            $(
                                DoubleRel::[<DoubleRel $K _ $V>](rel) => ArrangedDict::[<ArrangedDict $K _ $V>](rel.arrange_by_key()),
                            )*
                            DoubleRel::DoubleRelFat(_, _, _) => unreachable!("arrange_dict: fat case should be handled elsewhere"),
                        }
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

                // fat accessor
                pub fn rel_fat(&self) -> &Collection<G, (FatRow, FatRow), Semiring> {
                    match self {
                        DoubleRel::DoubleRelFat(rel, _, _) => rel,
                        _ => panic!("panic access to fat rel from non-fat DoubleRel"),
                    }
                }

                pub fn enter<'a, T>(&self, child: &Child<'a, G, T>) -> DoubleRel<Child<'a, G, T>>
                where
                    T: Refines<<G as ScopeParent>::Timestamp>+Lattice+TotalOrder,
                {
                    if self.is_fat() {
                        let (k_arity, v_arity) = self.arity();
                        DoubleRel::DoubleRelFat(self.rel_fat().enter(child), k_arity, v_arity)
                    } else {
                        match self {
                            $(
                                DoubleRel::[<DoubleRel $K _ $V>](rel) => DoubleRel::[<DoubleRel $K _ $V>](rel.enter(child)),
                            )*
                            DoubleRel::DoubleRelFat(_, _, _) => unreachable!("Fat case should be handled above"),
                        }
                    }
                }
            }
        }
    };
}

impl_double_rels!(
    (1, 1),
    (1, 2),
    (1, 3),
    (1, 4),
    (1, 5),
    (1, 6),
    (1, 7),
    (1, 8),
    (2, 1),
    (2, 2),
    (2, 3),
    (2, 4),
    (2, 5),
    (2, 6),
    (2, 7),
    (2, 8),
    (3, 1),
    (3, 2),
    (3, 3),
    (3, 4),
    (3, 5),
    (3, 6),
    (3, 7),
    (3, 8),
    (4, 1),
    (4, 2),
    (4, 3),
    (4, 4),
    (4, 5),
    (4, 6),
    (4, 7),
    (4, 8),
    (5, 1),
    (5, 2),
    (5, 3),
    (5, 4),
    (5, 5),
    (5, 6),
    (5, 7),
    (5, 8),
    (6, 1),
    (6, 2),
    (6, 3),
    (6, 4),
    (6, 5),
    (6, 6),
    (6, 7),
    (6, 8),
    (7, 1),
    (7, 2),
    (7, 3),
    (7, 4),
    (7, 5),
    (7, 6),
    (7, 7),
    (7, 8),
    (8, 1),
    (8, 2),
    (8, 3),
    (8, 4),
    (8, 5),
    (8, 6),
    (8, 7),
    (8, 8)
);
