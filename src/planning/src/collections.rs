use::std::fmt;
use std::sync::Arc;
use std::collections::HashMap;
use catalog::rule::Catalog;
use catalog::atoms::AtomArgumentSignature;

/*
    serialization identifies the collection and keep tracks of the lineage of the intermedidate transformation
*/
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CollectionSignature {
    Atom {  
        name: String,  
    },

    UnaryTransformationOutput { 
        name: String, 
    },

    JnOutput { 
        name: String, 
    },

    NegJnOutput { 
        name: String, 
    },
}

impl CollectionSignature {
    pub fn new_atom(name: &str) -> Self {
        Self::Atom { name: name.to_string() }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Atom { name } => name,
            Self::UnaryTransformationOutput { name } => name,
            Self::JnOutput { name } => name,
            Self::NegJnOutput { name } => name,
        }
    }

    pub fn debug_name(&self) -> String {
        let name = self.name();
        let mut debug_name = String::new();

        let mut skip = false;

        // for name, whenever a `|` is encountered, skip until the next `|` (skip the `|` as well) and keep on skipping until the end of the string
        for c in name.chars() {
            if c == '|' {
                skip = !skip;
            } else if !skip {
                debug_name.push(c);
            }
        }

        debug_name
    }

    pub fn is_atom(&self) -> bool {
        match self {
            Self::Atom { .. } => true,
            _ => false,
        }
    }
}

impl fmt::Display for CollectionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "{:?}", self)
        write!(f, "{}", self.name())
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Collection {
    signature: Arc<CollectionSignature>,
    key_argument_signatures: Vec<AtomArgumentSignature>,
    value_argument_signatures: Vec<AtomArgumentSignature>,
}

/* constructors */
impl Collection {
    pub fn new(signature: CollectionSignature, key_argument_signatures: &Vec<AtomArgumentSignature>, value_argument_signatures: &Vec<AtomArgumentSignature>) -> Self {
        Self {
            signature: Arc::new(signature),
            key_argument_signatures: key_argument_signatures.clone(),
            value_argument_signatures: value_argument_signatures.clone(),
        }
    }

    pub fn arity(&self) -> (usize, usize) {
        (self.key_argument_signatures.len(), self.value_argument_signatures.len())
    }

    pub fn is_kv(&self) -> bool {
        !self.key_argument_signatures.is_empty()
    }

    pub fn is_k_only(&self) -> bool {
        self.value_argument_signatures.is_empty()
    }

    pub fn signature(&self) -> &Arc<CollectionSignature> {
        &self.signature
    }

    pub fn kv_argument_signatures(&self) -> (&Vec<AtomArgumentSignature>, &Vec<AtomArgumentSignature>) {
        (&self.key_argument_signatures, &self.value_argument_signatures)
    }

    pub fn key_argument_signatures(&self) -> &Vec<AtomArgumentSignature> {
        &self.key_argument_signatures
    }

    pub fn value_argument_signatures(&self) -> &Vec<AtomArgumentSignature> {
        &self.value_argument_signatures
    }

    pub fn pprint(&self) -> String {
        if self.is_kv() {
            format!("{}({}: {})", self.signature.name(), 
                                  self.key_argument_signatures
                                    .iter()
                                    .map(|argument_signature| argument_signature.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", "),
                                  self.value_argument_signatures
                                    .iter()
                                    .map(|argument_signature| argument_signature.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                        )
        } else {
            /* row tuples */
            format!("{}({})", self.signature.name(), 
                                  self.value_argument_signatures
                                    .iter()
                                    .map(|argument_signature| argument_signature.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                        )
        }
    }   


    pub fn populate_argument_presence_map(&self, catalog: &Catalog) -> HashMap<String, AtomArgumentSignature> {
        let mut argument_presence_map = HashMap::new();
        for argument_signature in self.key_argument_signatures.iter().chain(self.value_argument_signatures.iter()) {
            let argument_str = &catalog.signature_to_argument_str_map()[argument_signature];
            /* only populate the entry if the argument is not already present */
            /* in other words, only the first occurrence of the argument is recorded */
            argument_presence_map.entry(argument_str.to_string()).or_insert_with(|| argument_signature.clone());
        }
        argument_presence_map
    }
}



