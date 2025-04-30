use std::fmt;
use parsing::rule::Const;
use parsing::arithmetic::ArithmeticOperator;
use catalog::arithmetic::ArithmeticPos;
use catalog::arithmetic::FactorPos;

use crate::arguments::TransformationArgument;

// move from a factor signature (pos) enum (when parsing) -- if it is a signature, we replace by the transformation argument
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactorArgument {
    Var(TransformationArgument),
    Const(Const),
}

impl FactorArgument {            
    pub fn transformation_arguments(&self) -> Vec<&TransformationArgument> {
        match self {
            FactorArgument::Var(transformation_arg) => vec![transformation_arg],
            FactorArgument::Const(_) => vec![],
        }
    }
}   

impl fmt::Display for FactorArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FactorArgument::Var(transformation_arg) => write!(f, "{}", transformation_arg),
            FactorArgument::Const(constant) => write!(f, "{}", constant),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ArithmeticArgument {
    init: FactorArgument,
    rest: Vec<(ArithmeticOperator, FactorArgument)>,
}

impl ArithmeticArgument {
    pub fn from_arithmetic(arithmetic: &ArithmeticPos, var_arguments: &[TransformationArgument]) -> Self {
        let mut var_id = 0;

        let init = match arithmetic.init() {
            FactorPos::Var(_) => {
                let var_signature = &var_arguments[var_id];
                var_id += 1;
                FactorArgument::Var(var_signature.clone())
            },
            FactorPos::Const(constant) => FactorArgument::Const(constant.clone()),
        };

        let rest = arithmetic.rest().iter().map(|(op, factor)| {
            let factor = match factor {
                FactorPos::Var(_) => {
                    let var_signature = &var_arguments[var_id];
                    var_id += 1;
                    FactorArgument::Var(var_signature.clone())
                },
                FactorPos::Const(constant) => FactorArgument::Const(constant.clone()),
            };
            (op.clone(), factor)
        }).collect();

        ArithmeticArgument { init, rest }
    }

    pub fn init(&self) -> &FactorArgument {
        &self.init
    }

    pub fn rest(&self) -> &Vec<(ArithmeticOperator, FactorArgument)> {
        &self.rest
    }

    pub fn is_literal(&self) -> bool {
        self.rest.is_empty()
    }

    pub fn transformation_arguments(&self) -> Vec<&TransformationArgument> {
        let mut transformation_arguments = self.init.transformation_arguments();
        for (_, factor) in &self.rest {
            transformation_arguments.extend(factor.transformation_arguments());
        }
        transformation_arguments
    }

    // flip the underlying transformation arguments 
    pub fn jn_flip(&self) -> Self {
        let init = match &self.init {
            FactorArgument::Var(arg) => FactorArgument::Var(arg.jn_flip()),
            FactorArgument::Const(constant) => FactorArgument::Const(constant.clone()),
        };

        let rest = self.rest.iter().map(|(op, factor)| {
            let factor = match factor {
                FactorArgument::Var(arg) => FactorArgument::Var(arg.jn_flip()),
                FactorArgument::Const(constant) => FactorArgument::Const(constant.clone()),
            };
            (op.clone(), factor)
        }).collect();

        ArithmeticArgument { init, rest }
    }
}

impl fmt::Display for ArithmeticArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.init)?;
        for (op, factor) in &self.rest {
            write!(f, " {} {}", op, factor)?;
        }
        Ok(())
    }
}





