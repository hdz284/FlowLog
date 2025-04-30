use std::fmt;
use std::collections::{HashMap, HashSet};
use parsing::rule::Const;

use crate::atoms::AtomArgumentSignature;

#[derive(Debug)]
pub struct BaseFilters {
    // variable equality constraints (e.g., arc(x, y), x = y), the key is the alias variable
    var_eq_map: HashMap<AtomArgumentSignature, AtomArgumentSignature>,

    // constant equality constraints (e.g., arc(x, 5) or arc(x, y), y = 6)
    const_map: HashMap<AtomArgumentSignature, Const>, 

    // placeholder set for redundant variables
    placeholder_set: HashSet<AtomArgumentSignature>,
}

impl BaseFilters {
    pub fn new(var_eq_map: HashMap<AtomArgumentSignature, AtomArgumentSignature>, const_map: HashMap<AtomArgumentSignature, Const>, placeholder_set: HashSet<AtomArgumentSignature>) -> Self {
        Self {
            var_eq_map,
            const_map,
            placeholder_set,
        }
    }

    pub fn var_eq_map(&self) -> &HashMap<AtomArgumentSignature, AtomArgumentSignature> {
        &self.var_eq_map
    }

    pub fn const_map(&self) -> &HashMap<AtomArgumentSignature, Const> {
        &self.const_map
    }

    pub fn placeholder_set(&self) -> &HashSet<AtomArgumentSignature> {
        &self.placeholder_set
    }

    pub fn is_const_or_var_eq_or_placeholder(&self, arg: &AtomArgumentSignature) -> bool {
        self.var_eq_map.contains_key(arg) || self.const_map.contains_key(arg) || self.placeholder_set.contains(arg)
    }

    pub fn is_empty(&self) -> bool {
        self.var_eq_map.is_empty() && self.const_map.is_empty() && self.placeholder_set.is_empty()
    }
}




impl fmt::Display for BaseFilters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // variable equality constraints
        writeln!(f, "Variable Eq Constraints Map:")?;
        for (arg1, arg2) in &self.var_eq_map {
            writeln!(f, "  {} -> {}", arg1, arg2)?;
        }

        // constant equality constraints
        writeln!(f, "\nConstant Map:")?;
        for (arg, constant) in &self.const_map {
            writeln!(f, "  {} -> {}", arg, constant)?;
        }

        // placeholder set
        writeln!(f, "\nPlaceholder Set:")?;
        for placeholder in &self.placeholder_set {
            writeln!(f, "  {}", placeholder)?;
        }

        Ok(())
    }
}