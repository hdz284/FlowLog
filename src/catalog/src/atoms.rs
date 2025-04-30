use std::fmt;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct AtomSignature {
    is_positive: bool,
    rhs_id: usize,
}

impl AtomSignature {
    pub fn new(is_positive: bool, rhs_id: usize) -> Self {
        Self { is_positive, rhs_id }
    }

    pub fn is_positive(&self) -> bool {
        self.is_positive
    }

    pub fn rhs_id(&self) -> usize {
        self.rhs_id
    }
}

impl fmt::Display for AtomSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", if self.is_positive { "" } else { "!" }, self.rhs_id) // e.g. !1
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AtomArgumentSignature {
    atom_signature: AtomSignature,
    argument_id: usize,
}

impl AtomArgumentSignature {
    pub fn new(atom_signature: AtomSignature, argument_id: usize) -> Self {
        Self { atom_signature, argument_id }
    }

    pub fn is_positive(&self) -> bool {
        self.atom_signature.is_positive()
    }

    pub fn atom_signature(&self) -> &AtomSignature {
        &self.atom_signature
    }
}

impl fmt::Display for AtomArgumentSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.atom_signature, self.argument_id) // e.g. !1.0
    }
}

