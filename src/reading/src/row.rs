use std::fmt;
use arrayvec::ArrayVec;
use std::fmt::Debug;
use std::hash::Hash;
use serde::{Deserialize, Serialize};

/* ------------------------------------------------------------------------------------ */
/* Array */
/* ------------------------------------------------------------------------------------ */

/// 
/// a trait to abstract ops over array implementations
pub trait Array: Debug + Send + Sync {
    /// insert a value
    fn push(&mut self, v: i32);
    /// return the number of columns
    fn arity(&self) -> usize;
    /// return the value of a column
    fn column(&self, id: usize) -> i32;
}

/// stack-allocated row for small arities using const generics
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row<const N: usize> {
    values: ArrayVec<i32, N>,
}

impl<const N: usize> Row<N> {
    pub fn new() -> Self {
        Self {
            values: ArrayVec::new(),
        }
    }

    // pub fn extend(&mut self, slice: &[i32]) {
    //     self.values.extend(slice.iter().cloned());
    // }
}

impl<const N: usize> Array for Row<N> {
    fn push(&mut self, v: i32) {
        self.values.push(v);
    }

    fn arity(&self) -> usize {
        self.values.len()
    }

    fn column(&self, id: usize) -> i32 {
        unsafe {
            *self.values.get_unchecked(id)
        }
    }
}

impl<const N: usize> fmt::Display for Row<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "{}",
            self.values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

