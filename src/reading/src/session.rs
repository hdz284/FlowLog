use paste::paste;
use timely::progress::Timestamp;
use differential_dataflow::input::InputSession;

use crate::Time;
use crate::Semiring;
use crate::row::Row;

/* ------------------------------------------------------------------------------------ */
/* session generics */
/* ------------------------------------------------------------------------------------ */
macro_rules! impl_input_sessions {
    ($($arity:literal),*) => {
        paste! {
            pub enum InputSessionGeneric<T: Timestamp + Clone> {
                $( [<InputSession $arity>](InputSession<T, Row<$arity>, Semiring>), )*
            }

            impl InputSessionGeneric<Time> {
                pub fn arity(&self) -> usize {
                    match self {
                        $( InputSessionGeneric::[<InputSession $arity>](_) => $arity, )*
                    }
                }

                pub fn close(self) {
                    match self {
                        $( InputSessionGeneric::[<InputSession $arity>](session) => session.close(), )*
                    }
                }

                $(
                    pub fn [<listen_ $arity>](&mut self) -> &mut InputSession<Time, Row<$arity>, Semiring> {
                        match self {
                            InputSessionGeneric::[<InputSession $arity>](session) => session,
                            _ => panic!("panic access to listen of arity {}", $arity),
                        }
                    }
                )*
            }
        }
    };
}

impl_input_sessions!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);