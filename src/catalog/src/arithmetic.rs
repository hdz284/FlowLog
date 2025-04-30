use std::fmt;
use parsing::rule::Const;
use parsing::arithmetic::ArithmeticOperator;
use parsing::arithmetic::Arithmetic;
use parsing::arithmetic::Factor;
use crate::atoms::AtomArgumentSignature;

// move from a factor enum (when parsing) -- if it is a variable, we get its argument signature
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactorPos {
    Var(AtomArgumentSignature),
    Const(Const),
}

impl FactorPos {            
    pub fn signatures(&self) -> Vec<&AtomArgumentSignature> {
        match self {
            FactorPos::Var(atom_arg_signature) => vec![atom_arg_signature],
            FactorPos::Const(_) => vec![],
        }
    }
}   

impl fmt::Display for FactorPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FactorPos::Var(atom_arg_signature) => write!(f, "{}", atom_arg_signature),
            FactorPos::Const(constant) => write!(f, "{}", constant),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ArithmeticPos {
    init: FactorPos,
    rest: Vec<(ArithmeticOperator, FactorPos)>,
}

impl ArithmeticPos {
    pub fn from_arithmetic(arithmetic: &Arithmetic, var_signatures: &[AtomArgumentSignature]) -> Self {
        let mut var_id = 0;

        let init = match arithmetic.init() {
            Factor::Var(_) => {
                let var_signature = &var_signatures[var_id];
                var_id += 1;
                FactorPos::Var(var_signature.clone())
            },
            Factor::Const(constant) => FactorPos::Const(constant.clone()),
        };

        let rest = arithmetic.rest().iter().map(|(op, factor)| {
            let factor = match factor {
                Factor::Var(_) => {
                    let var_signature = &var_signatures[var_id];
                    var_id += 1;
                    FactorPos::Var(var_signature.clone())
                },
                Factor::Const(constant) => FactorPos::Const(constant.clone()),
            };
            (op.clone(), factor)
        }).collect();

        ArithmeticPos { init, rest }
    }

    pub fn init(&self) -> &FactorPos {
        &self.init
    }

    pub fn rest(&self) -> &Vec<(ArithmeticOperator, FactorPos)> {
        &self.rest
    }

    pub fn is_literal(&self) -> bool {
        self.rest.is_empty()
    }

    pub fn is_var(&self) -> bool {
        self.is_literal() && matches!(self.init, FactorPos::Var(_))
    }

    pub fn signatures(&self) -> Vec<&AtomArgumentSignature> {
        let mut signatures = self.init.signatures();
        for (_, factor) in &self.rest {
            signatures.extend(factor.signatures());
        }
        signatures
    }
}

impl fmt::Display for ArithmeticPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.init)?;
        for (op, factor) in &self.rest {
            write!(f, " {} {}", op, factor)?;
        }
        Ok(())
    }
}





