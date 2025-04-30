/* ------------------------------------------------------------------------------------ */
/* I/O methods */
/* ------------------------------------------------------------------------------------ */

use std::fs::File;
use std::io::{BufRead, BufReader};

use differential_dataflow::input::InputSession;
use differential_dataflow::input::Input;
use differential_dataflow::operators::iterate::SemigroupVariable;
use differential_dataflow::difference::Present;
use timely::dataflow::Scope;
use timely::order::Product;


use parsing::decl::RelDecl;
use crate::row::Row;
use crate::row::Array;
use crate::rel::Rel;
use crate::session::InputSessionGeneric;
use crate::Time;
use crate::Iter;
use crate::Semiring;

#[inline(always)]
pub fn reader(rel_path: &str) -> impl Iterator<Item = Vec<u8>> {
    let read_f = 
        BufReader::new(
            File::open(rel_path).unwrap_or_else(|_| panic!("can't read data from \"{}\"", rel_path)),
        );

    read_f
        .split(b'\n')
        .filter_map(Result::ok)  // filter out errors
        .filter(move |line| !line.is_empty()) // skip empty lines
} 


macro_rules! read_row_fn {
    ($name:ident, $row_type:ty) => {
        pub fn $name(
            rel_decl: &RelDecl,
            rel_path: &str,
            delimiter: &u8,
            session: &mut InputSession<Time, $row_type, Semiring>,
            id: usize,
            peers: usize
        ) {
            let rel_arity = rel_decl.arity();

            if id == 0 {
                println!("reading {} from {}", rel_decl, rel_path);
            }

            let ingest = 
                reader(rel_path)
                    .filter_map(move |line| {
                        let mut tuple = line.split(|&bt| bt == *delimiter);

                        let first_value = std::str::from_utf8(tuple.next()?).ok()?.parse::<i32>().ok()?;
                        if (first_value as usize) % peers != id {
                            return None;
                        }

                        let mut row = <$row_type>::new();
                        row.push(first_value);

                        for value in tuple {
                            let parsed_value = std::str::from_utf8(value).ok()?.parse::<i32>().ok()?;
                            row.push(parsed_value);
                        }

                        if row.arity() != rel_arity {
                            panic!("expected {} values, got {}", rel_arity, row.arity());
                        }

                        Some(row)
                    });
            
            ingest.for_each(|row| session.update(row, Present {}));
        }
    };
}

read_row_fn!(read_row_1, Row<1>);
read_row_fn!(read_row_2, Row<2>);
read_row_fn!(read_row_3, Row<3>);
read_row_fn!(read_row_4, Row<4>);
read_row_fn!(read_row_5, Row<5>);
read_row_fn!(read_row_6, Row<6>);
read_row_fn!(read_row_7, Row<7>);
read_row_fn!(read_row_8, Row<8>);
read_row_fn!(read_row_9, Row<9>);
read_row_fn!(read_row_10, Row<10>);

// ...






/* ------------------------------------------------------------------------------------ */
/* construct session and table of some arity */
/* ------------------------------------------------------------------------------------ */
pub fn construct_session_and_table<G: Scope<Timestamp=Time>>(
    scope: &mut G,
    arity: usize,
) -> (InputSessionGeneric<Time>, Rel<G>) {
    match arity {
        1 => {
            let (session, input_rel) = scope.new_collection::<Row<1>, Semiring>();
            (InputSessionGeneric::InputSession1(session), Rel::Collection1(input_rel))
        }
        2 => {
            let (session, input_rel) = scope.new_collection::<Row<2>, Semiring>();
            (InputSessionGeneric::InputSession2(session), Rel::Collection2(input_rel))
        }
        3 => {
            let (session, input_rel) = scope.new_collection::<Row<3>, Semiring>();
            (InputSessionGeneric::InputSession3(session), Rel::Collection3(input_rel))
        }
        4 => {
            let (session, input_rel) = scope.new_collection::<Row<4>, Semiring>();
            (InputSessionGeneric::InputSession4(session), Rel::Collection4(input_rel))
        }
        5 => {
            let (session, input_rel) = scope.new_collection::<Row<5>, Semiring>();
            (InputSessionGeneric::InputSession5(session), Rel::Collection5(input_rel))
        }
        6 => {
            let (session, input_rel) = scope.new_collection::<Row<6>, Semiring>();
            (InputSessionGeneric::InputSession6(session), Rel::Collection6(input_rel))
        }
        7 => {
            let (session, input_rel) = scope.new_collection::<Row<7>, Semiring>();
            (InputSessionGeneric::InputSession7(session), Rel::Collection7(input_rel))
        }
        8 => {
            let (session, input_rel) = scope.new_collection::<Row<8>, Semiring>();
            (InputSessionGeneric::InputSession8(session), Rel::Collection8(input_rel))
        }
        9 => {
            let (session, input_rel) = scope.new_collection::<Row<9>, Semiring>();
            (InputSessionGeneric::InputSession9(session), Rel::Collection9(input_rel))
        }
        10 => {
            let (session, input_rel) = scope.new_collection::<Row<10>, Semiring>();
            (InputSessionGeneric::InputSession10(session), Rel::Collection10(input_rel))
        }
        _ => panic!("arity too large: {}", arity),
    }
}




/* ------------------------------------------------------------------------------------ */
/* read and insert row of some arity */
/* ------------------------------------------------------------------------------------ */
pub fn read_row_generic(
    rel_decl: &RelDecl,
    rel_path: &str,
    delimiter: &u8,
    session_generic: &mut InputSessionGeneric<Time>,
    id: usize,
    peers: usize,
) {
    match rel_decl.arity() {
        1 => read_row_1(rel_decl, rel_path, delimiter, &mut session_generic.listen_1(), id, peers),
        2 => read_row_2(rel_decl, rel_path, delimiter, &mut session_generic.listen_2(), id, peers),
        3 => read_row_3(rel_decl, rel_path, delimiter, &mut session_generic.listen_3(), id, peers),
        4 => read_row_4(rel_decl, rel_path, delimiter, &mut session_generic.listen_4(), id, peers),
        5 => read_row_5(rel_decl, rel_path, delimiter, &mut session_generic.listen_5(), id, peers),
        6 => read_row_6(rel_decl, rel_path, delimiter, &mut session_generic.listen_6(), id, peers),
        7 => read_row_7(rel_decl, rel_path, delimiter, &mut session_generic.listen_7(), id, peers),
        8 => read_row_8(rel_decl, rel_path, delimiter, &mut session_generic.listen_8(), id, peers),
        9 => read_row_9(rel_decl, rel_path, delimiter, &mut session_generic.listen_9(), id, peers),
        10 => read_row_10(rel_decl, rel_path, delimiter, &mut session_generic.listen_10(), id, peers),
        // ...
        _ => panic!("arity too large: {}", rel_decl.arity()),
    }
}



/* ------------------------------------------------------------------------------------ */
/* construct semigroup variable of some arity */
/* ------------------------------------------------------------------------------------ */

pub fn construct_var<G: Scope<Timestamp=Product<Time, Iter>>>(
    scope: &mut G,
    arity: usize,
) -> Rel<G> {
    match arity {
        1 => Rel::Variable1(SemigroupVariable::<_, Row<1>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        2 => Rel::Variable2(SemigroupVariable::<_, Row<2>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        3 => Rel::Variable3(SemigroupVariable::<_, Row<3>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        4 => Rel::Variable4(SemigroupVariable::<_, Row<4>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        5 => Rel::Variable5(SemigroupVariable::<_, Row<5>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        6 => Rel::Variable6(SemigroupVariable::<_, Row<6>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        7 => Rel::Variable7(SemigroupVariable::<_, Row<7>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        8 => Rel::Variable8(SemigroupVariable::<_, Row<8>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        9 => Rel::Variable9(SemigroupVariable::<_, Row<9>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        10 => Rel::Variable10(SemigroupVariable::<_, Row<10>, Semiring>::new(scope, Product::new(Default::default(), 1))),
        _ => panic!("arity too large: {}", arity),
    }
}
