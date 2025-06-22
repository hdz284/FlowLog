/* -----------------------------------------------------------------------------------------------
 * printing methods
 * -----------------------------------------------------------------------------------------------
 */
// use differential_dataflow::difference::Abelian;
use differential_dataflow::difference::Semigroup;

use differential_dataflow::operators::threshold::ThresholdTotal;
use differential_dataflow::lattice::Lattice;
use timely::order::TotalOrder;
use timely::dataflow::Scope; 
use differential_dataflow::{Collection, ExchangeData, Hashable};

use crate::rel::Rel;
use crate::semiring_one;

fn printsize<G, D, R>(rel: &Collection<G, D, R>, name: &str, is_recursive: bool)
where
    G: Scope,
    G::Timestamp: Lattice+TotalOrder,
    D: ExchangeData+Hashable,
    R: Semigroup+ExchangeData 
{
    let prefix = if is_recursive {
        format!("Delta of (recursive) {}", name)
    } else {
        format!("Size of (non-recursive) {}", name)
    };

    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
        .expand(|_| Some(((), 1 as i32)))
        .map(|_| ())
        .consolidate()
        .inspect(move |x| 
            println!("{}: {:?}", prefix, x) // use std::fmt::Display for D (i.e. Row)
        );
}


fn print<G, D, R>(rel: &Collection<G, D, R>, name: &str)
where
    G: Scope,
    G::Timestamp: Lattice+TotalOrder,
    D: ExchangeData+Hashable+std::fmt::Display,
    R: Semigroup+ExchangeData {
        
    let name = name.to_owned();
    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
        .expand(|x| Some((x, 1 as i32)))
        .inspect(move |(data, time, delta)| 
            println!("{}: ({}, {:?}, {})", name, data, time, delta) // use std::fmt::Display for D (i.e. Row)
        ); 
}


pub fn printsize_generic<G>(rel: &Rel<G>, name: &str, is_recursive: bool)
where
    G: Scope,
    G::Timestamp: Lattice+TotalOrder 
{
    if rel.is_fat() {
        printsize(rel.rel_fat(), name, is_recursive)
    } else {
        let arity = rel.arity();
        match arity {
            1 => printsize(rel.rel_1(), name, is_recursive),
            2 => printsize(rel.rel_2(), name, is_recursive),
            3 => printsize(rel.rel_3(), name, is_recursive),
            4 => printsize(rel.rel_4(), name, is_recursive),
            5 => printsize(rel.rel_5(), name, is_recursive),
            6 => printsize(rel.rel_6(), name, is_recursive),
            7 => printsize(rel.rel_7(), name, is_recursive),
            8 => printsize(rel.rel_8(), name, is_recursive),
            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
        }
    }
}


pub fn print_generic<G>(rel: &Rel<G>, name: &str)
where
    G: Scope,
    G::Timestamp: Lattice+TotalOrder 
{
    if rel.is_fat() {
        print(rel.rel_fat(), name)
    } else {
        let arity = rel.arity();
        match arity {
            1 => print(rel.rel_1(), name),
            2 => print(rel.rel_2(), name),
            3 => print(rel.rel_3(), name),
            4 => print(rel.rel_4(), name),
            5 => print(rel.rel_5(), name),
            6 => print(rel.rel_6(), name),
            7 => print(rel.rel_7(), name),
            8 => print(rel.rel_8(), name),
            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
        }
    }
}