


use std::fmt;


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TransformationArgument {
    /* KV({k, v}, id) */
    KV((bool, usize)),

    /* Jn({left, right}, {k, v}, id) */
    Jn((bool, bool, usize)),
}

impl TransformationArgument {
    pub fn kv_indices(&self) -> (bool, usize) {
        match self {
            TransformationArgument::KV(indices) => *indices,
            _ => panic!("TransformationArgument::kv_indices expects KV: {:?}", self),
        }
    }

    pub fn jn_indices(&self) -> (bool, bool, usize) {
        match self {
            TransformationArgument::Jn(indices) => *indices,
            _ => panic!("TransformationArgument::jn_indices expects Jn: {:?}", self),
        }
    }

    pub fn jn_flip(&self) -> Self {
        match self {
            TransformationArgument::Jn((left_or_right, key_or_value, id)) => TransformationArgument::Jn((!left_or_right, *key_or_value, *id)),
            _ => panic!("TransformationArgument::jn_flip expects Jn: {:?}", self),
        }
    }
 }

impl fmt::Display for TransformationArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformationArgument::KV((key_or_value, id)) => match *key_or_value {
                false => write!(f, "[k, {}]", id),
                true => write!(f, "[v, {}]", id),
            },
            TransformationArgument::Jn((left_or_right, key_or_value, id)) => {
                match *key_or_value {
                    false => write!(f, "[{}, k, {}]", left_or_right, id),
                    true => write!(f, "[{}, v, {}]", left_or_right, id),
                }
            }
        }
    }
}



