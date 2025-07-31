use crate::arguments::TransformationArgument;
use crate::arithmetic::ArithmeticArgument;
use crate::compare::ComparisonExprArgument;
use crate::constraints::BaseConstraints;
use catalog::arithmetic::ArithmeticPos;
use catalog::atoms::AtomArgumentSignature;
use catalog::compare::ComparisonExprPos;
use parsing::rule::Const;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum TransformationFlow {
    /* single atom rule, e.g. (y, z, x) -> (x, y, z), or equivalently, ((), y, z, x) -> ((), x, y, z) */
    /* lift base atoms, e.g. ((), x, y, z) -> ((x), y, z) */
    /* intermediate filters or truncates */
    KVToKV {
        key: Arc<Vec<TransformationArgument>>,
        value: Arc<Vec<TransformationArgument>>,
        constraints: BaseConstraints,          // local constraints
        compares: Vec<ComparisonExprArgument>, // local comparisons or filters
    },

    /* last join, e.g. ((x), y, z), ((x), w) -> ((), y, x, z, w) */
    /* intermediate join, e.g. ((x), y, z), ((x), w) -> ((x), y, z, w) */
    JnToKV {
        key: Arc<Vec<TransformationArgument>>,
        value: Arc<Vec<TransformationArgument>>,
        compares: Vec<ComparisonExprArgument>, // filters over joins
    },

    RowToHeadRow {
        value: Arc<Vec<ArithmeticArgument>>,
    },
}

impl TransformationFlow {
    // construct a new flow with the l or r flipped
    pub fn jn_flip(&self) -> Self {
        if let Self::JnToKV {
            key,
            value,
            compares,
        } = self
        {
            Self::JnToKV {
                key: Arc::new(key.iter().map(|arg| arg.jn_flip()).collect::<Vec<_>>()),
                value: Arc::new(value.iter().map(|arg| arg.jn_flip()).collect::<Vec<_>>()),
                compares: compares.iter().map(|comp| comp.jn_flip()).collect(),
            }
        } else {
            panic!("TransformationFlow::flip() called on kv")
        }
    }

    pub fn constraints(&self) -> &BaseConstraints {
        match self {
            Self::KVToKV { constraints, .. } => constraints,
            Self::JnToKV { .. } | Self::RowToHeadRow { .. } => {
                panic!("TransformationFlow::constraints() called on JnToKV or RowToHeadRow")
            }
        }
    }

    pub fn compares(&self) -> &Vec<ComparisonExprArgument> {
        match self {
            Self::KVToKV { compares, .. } => compares,
            Self::JnToKV { compares, .. } => compares,
            Self::RowToHeadRow { .. } => {
                panic!("TransformationFlow::compares() called on HeadRowToRow")
            }
        }
    }

    pub fn is_constrainted(&self) -> bool {
        match self {
            Self::KVToKV { constraints, .. } => {
                !(constraints.is_empty() && self.compares().is_empty())
            }
            Self::JnToKV { compares, .. } => !compares.is_empty(),
            Self::RowToHeadRow { .. } => false, // no constraints in HeadRowToRow
        }
    }

    /* check if key is empty */
    pub fn is_key_empty(&self) -> bool {
        match self {
            Self::KVToKV { key, .. } => key.is_empty(),
            Self::JnToKV { key, .. } => key.is_empty(),
            Self::RowToHeadRow { .. } => true, // no key in RowToHeadRow
        }
    }

    /* helper to get the input transformation arguments that flows over the transformation */
    fn flow_over_signatures(
        input_signature_map: &HashMap<AtomArgumentSignature, TransformationArgument>,
        output_signatures: &[AtomArgumentSignature],
        context: &str,
    ) -> Vec<TransformationArgument> {
        output_signatures
            .iter()
            .map(|signature| {
                *input_signature_map.get(signature).unwrap_or_else(|| {
                    panic!(
                        "{}: signature {:?} absent from the input signature map {:?}",
                        context, signature, input_signature_map
                    )
                })
            })
            .collect()
    }

    fn head_row_flow_over_signatures(
        input_signature_map: &HashMap<AtomArgumentSignature, TransformationArgument>,
        output_signatures: &[ArithmeticPos],
        context: &str,
    ) -> Vec<ArithmeticArgument> {
        output_signatures
            .iter()
            .map(|arithmetic_pos| {
                // Collect transformation arguments for all variables in this arithmetic expression
                let var_arguments: Vec<TransformationArgument> = arithmetic_pos
                    .signatures()
                    .iter()
                    .map(|signature| {
                        *input_signature_map.get(signature).unwrap_or_else(|| {
                            panic!(
                                "{}: signature {:?} absent from the input signature map {:?}",
                                context, signature, input_signature_map
                            )
                        })
                    })
                    .collect();

                // Use the existing from_arithmetic method to construct ArithmeticArgument
                ArithmeticArgument::from_arithmetic(arithmetic_pos, &var_arguments)
            })
            .collect()
    }

    /* helper to map kv argument signatures to transformation arguments in the flow */
    pub fn kv_argument_flow_map(
        key_signatures: &[AtomArgumentSignature],
        value_signatures: &[AtomArgumentSignature],
    ) -> HashMap<AtomArgumentSignature, TransformationArgument> {
        key_signatures
            .iter()
            .enumerate()
            .map(|(id, signature)| (signature.clone(), TransformationArgument::KV((false, id))))
            .chain(
                value_signatures.iter().enumerate().map(|(id, signature)| {
                    (signature.clone(), TransformationArgument::KV((true, id)))
                }),
            )
            .collect()
    }

    pub fn kv_to_kv(
        input_key_signatures: &Vec<AtomArgumentSignature>,
        input_value_signatures: &Vec<AtomArgumentSignature>,
        output_key_signatures: &Vec<AtomArgumentSignature>,
        output_value_signatures: &Vec<AtomArgumentSignature>,
        const_eq_constraints: &Vec<(AtomArgumentSignature, Const)>,
        var_eq_constraints: &Vec<(AtomArgumentSignature, AtomArgumentSignature)>,
        compare_exprs: &Vec<ComparisonExprPos>,
    ) -> Self {
        let input_signature_map =
            Self::kv_argument_flow_map(input_key_signatures, input_value_signatures);

        let flow_key_signatures = Self::flow_over_signatures(
            &input_signature_map,
            output_key_signatures,
            "(TransformationFlow::kv_to_kv) key",
        );
        let flow_value_signatures = Self::flow_over_signatures(
            &input_signature_map,
            output_value_signatures,
            "(TransformationFlow::kv_to_kv) value",
        );

        /* const constraints */
        let const_signatures: Vec<AtomArgumentSignature> = const_eq_constraints
            .iter()
            .map(|(signature, _)| signature.clone())
            .collect();

        let flow_const_signatures = Self::flow_over_signatures(
            &input_signature_map,
            &const_signatures,
            "(TransformationFlow::kv_to_kv) const",
        )
        .into_iter()
        .zip(
            const_eq_constraints
                .iter()
                .map(|(_, constant)| constant.clone()),
        )
        .collect::<Vec<(TransformationArgument, Const)>>();

        /* var eq constraints */
        let var_signatures: Vec<AtomArgumentSignature> = var_eq_constraints
            .iter()
            .map(|(left_signature, _)| left_signature.clone())
            .collect();

        let alias_signatures: Vec<AtomArgumentSignature> = var_eq_constraints
            .iter()
            .map(|(_, right_signature)| right_signature.clone())
            .collect();

        let flow_var_signatures = Self::flow_over_signatures(
            &input_signature_map,
            &var_signatures,
            "(TransformationFlow::kv_to_kv) var left",
        );
        let flow_alias_signatures = Self::flow_over_signatures(
            &input_signature_map,
            &alias_signatures,
            "(TransformationFlow::kv_to_kv) var right",
        );
        let flow_var_eq_signatures = flow_var_signatures
            .into_iter()
            .zip(flow_alias_signatures.into_iter())
            .collect::<Vec<(TransformationArgument, TransformationArgument)>>();

        /* comparison constraints */
        let flow_compare_signatures = compare_exprs
            .iter()
            .map(|comp| {
                let left_signatures = comp
                    .left()
                    .signatures()
                    .iter()
                    .map(|&signature| signature.clone())
                    .collect::<Vec<_>>();
                let right_signatures = comp
                    .right()
                    .signatures()
                    .iter()
                    .map(|&signature| signature.clone())
                    .collect::<Vec<_>>();

                /* move signatures into transformation arguments */
                ComparisonExprArgument::from_comparison_expr(
                    comp,
                    &Self::flow_over_signatures(
                        &input_signature_map,
                        &left_signatures,
                        "(TransformationFlow::kv_to_kv) compare left",
                    ),
                    &Self::flow_over_signatures(
                        &input_signature_map,
                        &right_signatures,
                        "(TransformationFlow::kv_to_kv) compare right",
                    ),
                )
            })
            .collect::<Vec<ComparisonExprArgument>>();

        Self::KVToKV {
            key: Arc::new(flow_key_signatures),
            value: Arc::new(flow_value_signatures),
            constraints: BaseConstraints::new(flow_const_signatures, flow_var_eq_signatures),
            compares: flow_compare_signatures,
        }
    }

    pub fn join_to_kv(
        input_left_key_signatures: &Vec<AtomArgumentSignature>,
        input_left_value_signatures: &Vec<AtomArgumentSignature>,
        _input_right_key_signatures: &Vec<AtomArgumentSignature>, // not necessary
        input_right_value_signatures: &Vec<AtomArgumentSignature>,
        output_key_signatures: &Vec<AtomArgumentSignature>,
        output_value_signatures: &Vec<AtomArgumentSignature>,
        compare_exprs: &Vec<ComparisonExprPos>,
    ) -> Self {
        // debug!("input_left_key_signatures: {:?}", input_left_key_signatures);
        // debug!("input_right_key_signatures: {:?}", input_right_key_signatures);

        let left_signature_map =
            Self::kv_argument_flow_map(input_left_key_signatures, input_left_value_signatures)
                .into_iter()
                .map(|(signature, trace)| {
                    let join_trace = match trace {
                        TransformationArgument::KV((key_or_value, id)) => {
                            TransformationArgument::Jn((false, key_or_value, id))
                        }
                        _ => panic!(
                            "TransformationFlow::Jn expects kv in left input: {:?}",
                            trace
                        ),
                    };
                    (signature, join_trace)
                });

        let right_signature_map = Self::kv_argument_flow_map(&vec![], input_right_value_signatures) // Self::kv_argument_flow_map(input_right_key_signatures, input_right_value_signatures)
            .into_iter()
            .map(|(signature, trace)| {
                let join_trace = match trace {
                    TransformationArgument::KV((key_or_value, id)) => {
                        TransformationArgument::Jn((true, key_or_value, id))
                    }
                    _ => panic!(
                        "TransformationFlow::join_to_kv expects kv in right input: {:?}",
                        trace
                    ),
                };
                (signature, join_trace)
            });

        let input_signature_map = left_signature_map.chain(right_signature_map).collect();
        let flow_key_signatures = Self::flow_over_signatures(
            &input_signature_map,
            output_key_signatures,
            "(TransformationFlow::join_to_kv) key",
        );
        let flow_value_signatures = Self::flow_over_signatures(
            &input_signature_map,
            output_value_signatures,
            "(TransformationFlow::join_to_kv) value",
        );

        /* comparison constraints */
        let flow_compare_signatures = compare_exprs
            .iter()
            .map(|comp| {
                let left_signatures = comp
                    .left()
                    .signatures()
                    .iter()
                    .map(|&signature| signature.clone())
                    .collect::<Vec<_>>();
                let right_signatures = comp
                    .right()
                    .signatures()
                    .iter()
                    .map(|&signature| signature.clone())
                    .collect::<Vec<_>>();

                /* move signatures into transformation arguments */
                ComparisonExprArgument::from_comparison_expr(
                    comp,
                    &Self::flow_over_signatures(
                        &input_signature_map,
                        &left_signatures,
                        "(TransformationFlow::join_to_kv) compare left",
                    ),
                    &Self::flow_over_signatures(
                        &input_signature_map,
                        &right_signatures,
                        "(TransformationFlow::join_to_kv) compare right",
                    ),
                )
            })
            .collect::<Vec<ComparisonExprArgument>>();

        Self::JnToKV {
            key: Arc::new(flow_key_signatures),
            value: Arc::new(flow_value_signatures),
            compares: flow_compare_signatures,
        }
    }

    pub fn head_row_to_row(
        input_value_signatures: &Vec<AtomArgumentSignature>,
        output_value_signatures: &Vec<ArithmeticPos>,
    ) -> Self {
        let input_signature_map = Self::kv_argument_flow_map(&vec![], &input_value_signatures);
        let flow_value_signatures = Self::head_row_flow_over_signatures(
            &input_signature_map,
            output_value_signatures,
            "(TransformationFlow::head_row_to_row) value",
        );

        Self::RowToHeadRow {
            value: Arc::new(flow_value_signatures),
        }
    }

    pub fn head_arity(&self) -> usize {
        match self {
            Self::RowToHeadRow { value } => value.len(),
            _ => panic!("head_arity called on non-head flow"),
        }
    }
}

impl fmt::Display for TransformationFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KVToKV {
                key,
                value,
                constraints,
                compares,
            } => {
                let filters_str = match (constraints.is_empty(), compares.is_empty()) {
                    (true, true) => String::new(),
                    (false, true) => format!(" if {}", constraints),
                    (true, false) => format!(
                        " if {}",
                        compares
                            .iter()
                            .map(|comp| format!("{}", comp))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    (false, false) => format!(
                        " if {} and {}",
                        constraints,
                        compares
                            .iter()
                            .map(|comp| format!("{}", comp))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                };

                if key.is_empty() {
                    write!(
                        f,
                        "|({}){}|",
                        value
                            .iter()
                            .map(|transformation_argument| format!("{}", transformation_argument))
                            .collect::<Vec<String>>()
                            .join(", "),
                        filters_str
                    )
                } else {
                    write!(
                        f,
                        "|({}: {}){}|",
                        key.iter()
                            .map(|transformation_argument| format!("{}", transformation_argument))
                            .collect::<Vec<String>>()
                            .join(", "),
                        value
                            .iter()
                            .map(|transformation_argument| format!("{}", transformation_argument))
                            .collect::<Vec<String>>()
                            .join(", "),
                        filters_str
                    )
                }
            }

            Self::JnToKV {
                key,
                value,
                compares,
            } => {
                let filters_str = if compares.is_empty() {
                    String::new()
                } else {
                    format!(
                        " if {}",
                        compares
                            .iter()
                            .map(|comp| format!("{}", comp))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                };

                if key.is_empty() {
                    write!(
                        f,
                        "|({}){}|",
                        value
                            .iter()
                            .map(|transformation_argument| format!("{}", transformation_argument))
                            .collect::<Vec<String>>()
                            .join(", "),
                        filters_str
                    )
                } else {
                    write!(
                        f,
                        "|({}: {}){}|",
                        key.iter()
                            .map(|transformation_argument| format!("{}", transformation_argument))
                            .collect::<Vec<String>>()
                            .join(", "),
                        value
                            .iter()
                            .map(|transformation_argument| format!("{}", transformation_argument))
                            .collect::<Vec<String>>()
                            .join(", "),
                        filters_str
                    )
                }
            }

            Self::RowToHeadRow { value } => {
                write!(
                    f,
                    "|({})|",
                    value
                        .iter()
                        .map(|transformation_argument| format!("{}", transformation_argument))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}
