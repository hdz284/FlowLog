/* ------------------------------------------------------------------------------------ */
/* I/O methods - Macro-based implementation for arities 1 through MAX_ARITY */
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
use crate::row::FatRow;
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


/* ------------------------------------------------------------------------------------ */
/* read row for thin relations */
/* ------------------------------------------------------------------------------------ */
macro_rules! generate_read_row_functions {
    ($($n:expr),*) => {
        $(
            paste::paste! {
                pub fn [<read_row_ $n>](
                    rel_decl: &RelDecl,
                    rel_path: &str,
                    delimiter: &u8,
                    session: &mut InputSession<Time, Row<$n>, Semiring>,
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

                                let mut row = Row::<$n>::new();
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
            }
        )*
    };
}

// `read_row_i(rel_decl: &RelDecl, rel_path: &str, delimiter: &u8, session: &mut InputSession<Time, Row<i>, Semiring>, id: usize, peers: usize)` for i from 1 to 8
generate_read_row_functions!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);




/* ------------------------------------------------------------------------------------ */
/* read row for fat relations */
/* ------------------------------------------------------------------------------------ */

pub fn read_row_fat(
    rel_decl: &RelDecl,
    rel_path: &str,
    delimiter: &u8,
    session: &mut InputSession<Time, FatRow, Semiring>,
    id: usize,
    peers: usize,
) {
    let rel_arity = rel_decl.arity();
    
    let ingest = 
        reader(rel_path)
            .filter_map(move |line| {
                let mut tuple = line.split(|&bt| bt == *delimiter);

                let first_value = std::str::from_utf8(tuple.next()?).ok()?.parse::<i32>().ok()?;
                if (first_value as usize) % peers != id {
                    return None;
                }

                let mut row = FatRow::new();
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





/* ------------------------------------------------------------------------------------ */
/* construct session and table of some arity */
/* ------------------------------------------------------------------------------------ */

macro_rules! generate_construct_session_and_table {
    ($($n:expr),*) => {
        pub fn construct_session_and_table<G: Scope<Timestamp=Time>>(
            scope: &mut G,
            arity: usize,
            fat_mode: bool,
        ) -> (InputSessionGeneric<Time>, Rel<G>) {
            if !fat_mode {
                match arity {
                    $(
                        $n => {
                            let (session, input_rel) = scope.new_collection::<Row<$n>, Semiring>();
                            paste::paste! {
                                (InputSessionGeneric::[<InputSession $n>](session), Rel::[<Collection $n>](input_rel))
                            }
                        }
                    )*
                    _ => unreachable!("construct_session_and_table: arity {} overflows", arity),
                }
            } else {
                let (session, input_rel) = scope.new_collection::<FatRow, Semiring>();
                (
                    InputSessionGeneric::InputSessionFat(session, arity),
                    Rel::CollectionFat(input_rel, arity)
                )
            }
        }
    };
}

// `construct_session_and_table_i(scope: &mut G, arity: usize) -> (InputSessionGeneric<Time>, Rel<G>)` for i from 1 to 8
generate_construct_session_and_table!(1, 2, 3, 4, 5, 6, 7, 8);




/* ------------------------------------------------------------------------------------ */
/* read and insert row of some arity */
/* ------------------------------------------------------------------------------------ */

macro_rules! generate_read_row_generic {
    ($($n:expr),*) => {
        pub fn read_row_generic(
            rel_decl: &RelDecl,
            rel_path: &str,
            delimiter: &u8,
            session_generic: &mut InputSessionGeneric<Time>,
            id: usize,
            peers: usize,
            fat_mode: bool,
        ) {
            let arity = rel_decl.arity();
            if !fat_mode {
                match arity {
                    $(
                        $n => paste::paste! {
                            [<read_row_ $n>](rel_decl, rel_path, delimiter, &mut session_generic.[<listen_ $n>](), id, peers)
                        },
                    )*
                    _ => unreachable!("arity {} should be handled by match arms if <= MAX_FALLBACK_ARITY", arity),
                }
            } else {
                // fat mode
                read_row_fat(rel_decl, rel_path, delimiter, &mut session_generic.listen_fat(), id, peers)
            }
        }
    };
}

// `read_row_generic(rel_decl: &RelDecl, rel_path: &str, delimiter: &u8, session_generic: &mut InputSessionGeneric<Time>, id: usize, peers: usize)` for i from 1 to 8
generate_read_row_generic!(1, 2, 3, 4, 5, 6, 7, 8);



/* ------------------------------------------------------------------------------------ */
/* construct semigroup variable of some arity */
/* ------------------------------------------------------------------------------------ */

macro_rules! generate_construct_var {
    ($($n:expr),*) => {
        pub fn construct_var<G: Scope<Timestamp=Product<Time, Iter>>>(
            scope: &mut G,
            arity: usize,
            fat_mode: bool,
        ) -> Rel<G> {
            if !fat_mode {
                match arity {
                    $(
                        $n => paste::paste! {
                            Rel::[<Variable $n>](SemigroupVariable::<_, Row<$n>, Semiring>::new(scope, Product::new(Default::default(), 1)))
                        },
                    )*
                    _ => unreachable!("arity {} should be handled by match arms if <= MAX_FALLBACK_ARITY", arity),
                }
            } else {
                // fat mode
                Rel::VariableFat(
                    SemigroupVariable::<_, FatRow, Semiring>::new(scope, Product::new(Default::default(), 1)),
                    arity
                )
            }
        }
    };
}

// `construct_var_i(scope: &mut G, arity: usize) -> Rel<G>` for i from 1 to 8
generate_construct_var!(1, 2, 3, 4, 5, 6, 7, 8);

