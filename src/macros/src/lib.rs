use itertools::iproduct;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

// Import centralized configuration constants
use reading::config::{KV_MAX, PROD_MAX, ROW_MAX};

/* ------------------------------------------------------------------------ */
/* codegen for maps */
/* ------------------------------------------------------------------------ */

/* row → row */
#[proc_macro]
pub fn codegen_row_row(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=ROW_MAX, 1..=ROW_MAX);
    let mut arms = vec![];
    for (iv_, target_) in space {
        let base_type = Ident::new(&format!("rel_{}", iv_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#iv_, #target_) => #final_rel(
                input_rel.#base_type().flat_map(row_row::<#iv_, #target_>(flow)))
        });
    }

    let expanded = quote! {
        if input_rel.is_fat() {
            CollectionFat(
                input_rel.rel_fat().flat_map(row_row_fat(flow)),
                target
            )
        } else {
            match (iv, target) {
                #(#arms),*,
                _ => panic!("codegen_row_row unimplemented for {}, {}", iv, target),
            }
        }
    };

    TokenStream::from(expanded)
}


/* row → head (arithmetic transformation) */
#[proc_macro]
pub fn codegen_row_head(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=ROW_MAX, 1..=ROW_MAX);
    let mut arms = vec![];
    
    for (input_arity_, output_arity_) in space {
        let base_type = Ident::new(&format!("rel_{}", input_arity_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", output_arity_), Span::call_site());
        arms.push(quote! {
            (#input_arity_, #output_arity_) => #final_rel(
                input_rel.#base_type().flat_map(row_head::<#input_arity_, #output_arity_>(flow)))
        });
    }

    let expanded = quote! {
        if input_rel.is_fat() {
            CollectionFat(
                input_rel.rel_fat().flat_map(row_head_fat(flow)),
                target
            )
        } else {
            match (iv, target) {
                #(#arms),*,
                _ => panic!("codegen_row_head unimplemented for input arity {} and output arity {}", iv, target),
            }
        }
    };

    TokenStream::from(expanded)
}

/* row → kv */
#[proc_macro]
pub fn codegen_row_kv(_: TokenStream) -> TokenStream {
    let space =
        iproduct!(1..=ROW_MAX, 1..=KV_MAX, 1..=KV_MAX).filter(|&(iv, ok, ov)| iv >= ok + ov);
    let mut arms = vec![];

    for (iv_, ok_, ov_) in space {
        let base_type = Ident::new(&format!("rel_{}", iv_), Span::call_site());
        let final_double_rel = Ident::new(&format!("DoubleRel{}_{}", ok_, ov_), Span::call_site());
        arms.push(quote! {
            (#iv_, #ok_, #ov_) => #final_double_rel(
                input_rel.#base_type()
                         .flat_map(row_kv::<#iv_, #ok_, #ov_>(flow))
                        )
        });
    }

    let expanded = quote! {
        if input_rel.is_fat() {
            DoubleRelFat(
                input_rel.rel_fat().flat_map(row_kv_fat(flow)),
                ok, // key arity
                ov  // value arity
            )
        } else {
            match (iv, ok, ov) {
                #(#arms),*,
                _ => panic!("codegen_row_kv unimplemented for {}, {}, {}", iv, ok, ov),
            }
        }
    };

    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------ */
/* codegen for kv ⋈ kv */
/* ------------------------------------------------------------------------ */
#[proc_macro]
pub fn codegen_jn(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=KV_MAX, 1..=KV_MAX, 1..=KV_MAX, 1..=ROW_MAX)
        .filter(|&(_, iv0, iv1, _)| iv0 >= iv1);
    let mut arms = vec![];

    for (ik0_, iv0_, iv1_, target_) in space {
        let type_0 = Ident::new(&format!("dict_{}_{}", ik0_, iv0_), Span::call_site());
        let type_1 = Ident::new(&format!("dict_{}_{}", ik0_, iv1_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#ik0_, #iv0_, #iv1_, #target_) => {
                #final_rel(
                    dict_0.#type_0()
                    .join_core(
                        dict_1.#type_1(),
                        jn_logic::<#ik0_, #iv0_, #iv1_, #target_>(flow)
                    )
                )
            }
        });
    }

    let expanded = quote! {
        if dict_0.is_fat() && dict_1.is_fat() {
            CollectionFat(
                dict_0.dict_fat()
                    .join_core(
                        dict_1.dict_fat(),
                        jn_logic_fat(flow)
                    ),
                target
            )
        } else {
            match (ik0, iv0, iv1, target) {
                #(#arms),*,
                _ => panic!("codegen_jn unimplemented for {}, {}, {}, {}", ik0, iv0, iv1, target),
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn codegen_cartesian(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=PROD_MAX, 1..=PROD_MAX, 1..=PROD_MAX)
        .filter(|&(iv0, iv1, target)| iv0 + iv1 >= target);
    let mut arms = vec![];

    for (iv0_, iv1_, target_) in space {
        let type_0 = Ident::new(&format!("rel_{}", iv0_), Span::call_site());
        let type_1 = Ident::new(&format!("rel_{}", iv1_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#iv0_, #iv1_, #target_) => {
                #final_rel(
                    rel_0.#type_0()
                         .map(|x| ((), x))
                         .arrange_by_key()
                         .join_core(
                            &rel_1.#type_1()
                                  .map(|x| ((), x))
                                  .arrange_by_key(),
                                cartesian_logic::<#iv0_, #iv1_, #target_>(flow)
                         )
                )
            }
        });
    }

    let expanded = quote! {
        if rel_0.is_fat() && rel_1.is_fat() {
            CollectionFat(
                rel_0.rel_fat()
                     .map(|x| ((), x))
                     .arrange_by_key()
                     .join_core(
                        &rel_1.rel_fat()
                              .map(|x| ((), x))
                              .arrange_by_key(),
                            cartesian_logic_fat(flow)
                     ),
                target
            )
        } else {
            match (iv0, iv1, target) {
                #(#arms),*,
                _ => panic!("codegen_cartesian unimplemented for {}, {}, {}", iv0, iv1, target),
            }
        }
    };

    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------ */
/* codegen for kv ⋈ k */
/* ------------------------------------------------------------------------ */
#[proc_macro]
pub fn codegen_kv_k_jn(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=KV_MAX, 1..=KV_MAX, 1..=ROW_MAX);

    let mut arms = vec![];
    for (ik0_, iv0_, target_) in space {
        let type_0 = Ident::new(&format!("dict_{}_{}", ik0_, iv0_), Span::call_site());
        let type_1 = Ident::new(&format!("set_{}", ik0_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#ik0_, #iv0_, #target_) => {
                #final_rel(
                    dict_0.#type_0()
                    .join_core(
                        set_1.#type_1(),
                        v1_jn_logic::<#ik0_, #iv0_, #target_>(flow)
                    )
                )
            }
        });
    }

    let expanded = quote! {
        if dict_0.is_fat() && set_1.is_fat() {
            CollectionFat(
                dict_0.dict_fat()
                    .join_core(
                        set_1.set_fat(),
                        v1_jn_logic_fat(flow)
                    ),
                target
            )
        } else {
            match (ik0, iv0, target) {
                #(#arms),*,
                _ => panic!("cpdegen_kv_k_jn unimplemented for {}, {}, {}", ik0, iv0, target),
            }
        }
    };

    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------ */
/* codegen for k ⋈ k */
/* ------------------------------------------------------------------------ */
#[proc_macro]
pub fn codegen_k_k_jn(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=KV_MAX, 1..=ROW_MAX);

    let mut arms = vec![];
    for (ik0_, target_) in space {
        let type_0 = Ident::new(&format!("set_{}", ik0_), Span::call_site());
        let type_1 = Ident::new(&format!("set_{}", ik0_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#ik0_, #target_) => {
                #final_rel(
                    set_0.#type_0()
                    .join_core(
                        set_1.#type_1(),
                        v2_jn_logic::<#ik0_, #target_>(flow)
                    )
                )
            }
        });
    }

    let expanded = quote! {
        if set_0.is_fat() && set_1.is_fat() {
            CollectionFat(
                set_0.set_fat()
                    .join_core(
                        set_1.set_fat(),
                        v2_jn_logic_fat(flow)
                    ),
                target
            )
        } else {
            match (ik0, target) {
                #(#arms),*,
                _ => panic!("codegen_k_k_jn unimplemented for {}, {}", ik0, target),
            }
        }
    };

    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------ */
/* codegen for aj flatten */
/* ------------------------------------------------------------------------ */

#[proc_macro]
pub fn codegen_kv_flatten(_: TokenStream) -> TokenStream {
    let space =
        iproduct!(1..=KV_MAX, 1..=KV_MAX, 1..=KV_MAX).filter(|&(ik, iv, target)| ik + iv >= target);

    let mut arms = vec![];
    for (ik0_, iv0_, target_) in space {
        let type_0 = Ident::new(&format!("dict_{}_{}", ik0_, iv0_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#ik0_, #iv0_, #target_) => {
                #final_rel(
                    dict_0.#type_0().as_collection(aj_flatten::<#ik0_, #iv0_, #target_>(flow))
                )
            }
        });
    }

    let expanded = quote! {
        if dict_0.is_fat() {
            CollectionFat(
                dict_0.dict_fat().as_collection(aj_flatten_fat(flow)),
                target
            )
        } else {
            match (ik0, iv0, target) {
                #(#arms),*,
                _ => panic!("codegen_kv_flatten unimplemented for {}, {}, {}", ik0, iv0, target),
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn codegen_k_flatten(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=KV_MAX, 1..=KV_MAX).filter(|&(ik, target)| ik >= target);

    let mut arms = vec![];
    for (ik0_, target_) in space {
        let type_0 = Ident::new(&format!("set_{}", ik0_), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", target_), Span::call_site());
        arms.push(quote! {
            (#ik0_, #target_) => {
                #final_rel(
                    set_0.#type_0().as_collection(v1_aj_flatten::<#ik0_, #target_>(flow))
                )
            }
        });
    }

    let expanded = quote! {
        if set_0.is_fat() {
            CollectionFat(
                set_0.set_fat().as_collection(v1_aj_flatten_fat(flow)),
                target
            )
        } else {
            match (ik0, target) {
                #(#arms),*,
                _ => panic!("codegen_k_flatten unimplemented for {}, {}", ik0, target),
            }
        }
    };

    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------ */
/* codegen for aggregation */
/* ------------------------------------------------------------------------ */
#[proc_macro]
pub fn codegen_aggregation(_: TokenStream) -> TokenStream {
    let space = 0..=KV_MAX;
    let mut arms = vec![];

    for key_arity in space {
        let arity = key_arity + 1;
        let base_type = Ident::new(&format!("rel_{}", arity), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", arity), Span::call_site());

        arms.push(quote! {
            #arity => Rel::#final_rel(
                input_rel.#base_type()
                    .map(row_chop::<#arity, #key_arity, 1>())
                    .reduce_core::<_,ValBuilder<_,_,_,_>,ValSpine<_,_,_,_>>(
                        "aggregation",
                        aggregation_reduce_logic::<#key_arity>(&aggregation)
                    )
                    .as_collection(|k, v| aggregation_merge_kv::<#key_arity, #arity>()((k.clone(), v.clone())))
            )
        });
    }

    let expanded = quote! {
        if input_rel.is_fat() {
            Rel::CollectionFat(
                input_rel.rel_fat()
                    .map(aggregation_separate_kv_fat())
                    .reduce_core::<_,ValBuilder<_,_,_,_>,ValSpine<_,_,_,_>>(
                        "aggregation",
                        aggregation_reduce_logic_fat(&aggregation)
                    )
                    .as_collection(|k, v| aggregation_merge_kv_fat()((k.clone(), v.clone()))),
                idb_catalog.arity()
            )
        } else {
            match idb_catalog.arity() {
                #(#arms),*,
                _ => panic!("codegen_aggregation unimplemented for arity {}", idb_catalog.arity()),
            }
        }
    };
    TokenStream::from(expanded)
}

/* ------------------------------------------------------------------------ */
/* codegen for MIN aggregation optimization with Min semiring */
/* ------------------------------------------------------------------------ */

#[proc_macro]
pub fn codegen_min_optimize(_: TokenStream) -> TokenStream {
    let space = 1..=KV_MAX + 1; // Support up to KV_MAX + 1 arity for MIN aggregation
    let mut arms = vec![];

    for arity in space {
        let base_type = Ident::new(&format!("rel_{}", arity), Span::call_site());
        let final_rel = Ident::new(&format!("Collection{}", arity), Span::call_site());
        let key_arity = arity - 1;

        arms.push(quote! {
            #arity => Rel::#final_rel(
                input_rel.#base_type()
                    // Phase 1: Transform input tuples to (key, Min_value) pairs
                    // ========================================================
                    // Extract key columns (all but last) and value column (last)
                    // Convert value to u32 and wrap in Min semiring structure
                    .inner
                    .flat_map(move |(row, t, _)| {
                        let mut key = reading::row::Row::<#key_arity>::new();
                        for i in 0..#key_arity {
                            key.push(row.column(i));
                        }
                        let value = row.column(#key_arity) as u32;
                        std::iter::once((key, reading::Min::new(value))).into_iter().map(move |(x, d2)| (x, t.clone(), d2))
                    })
                    .as_collection()
                    
                    // Phase 2: Apply MIN semiring with thresholding
                    // ========================================================
                    // The threshold_semigroup operator uses MIN semiring semantics:
                    // - Combines multiple values for same key using min() operation
                    // - Only emits updates when minimum actually decreases
                    // - Suppresses redundant updates for non-improving values
                    .threshold_semigroup(|_k, &new_min, current_min| {
                        match current_min {
                            Some(current) if new_min < *current => Some(new_min),
                            Some(_) => None,
                            None if !new_min.is_zero() => Some(new_min),
                            None => None,
                        }
                    })
                    
                    // Phase 3: Convert back to standard tuple representation
                    // =====================================================
                    // Extract minimum value from Min semiring difference
                    // Reconstruct full tuple with key columns + minimum value
                    // Convert to Present semiring for downstream operators
                    .inner
                    .flat_map(move |(key, t, min_val)| {
                        let mut result = reading::row::Row::<#arity>::new();
                        // Add key columns
                        for i in 0..#key_arity {
                            result.push(key.column(i));
                        }
                        // Push minimized value into the last column (extracted from diff!)
                        result.push(min_val.value as i32);
                        std::iter::once((result, reading::semiring_one())).into_iter().map(move |(x2, d2)| (x2, t.clone(), d2))
                    })
                    .as_collection()
            )
        });
    }

    let expanded = quote! {
        if input_rel.is_fat() {
             Rel::CollectionFat(
                input_rel.rel_fat()
                    // Phase 1: Transform fat tuples to (key, Min_value) pairs
                    // =======================================================
                    // Extract key columns (all but last) and value column (last)
                    // Convert value to u32 and wrap in Min semiring structure
                    .inner
                    .flat_map(move |(row, t, _)| {
                        let mut key = reading::row::FatRow::new();
                        let arity = row.arity();
                            
                        // Extract all columns except the last as the group-by key
                        for i in 0..arity - 1 {
                            key.push(row.column(i));
                        }
                            
                        // Extract the last column as the value to minimize
                        let value = row.column(arity - 1) as u32;
                        std::iter::once((key, reading::Min::new(value))).into_iter().map(move |(x, d2)| (x, t.clone(), d2))
                    })
                    .as_collection()
                        
                    // Phase 2: Apply MIN semiring with intelligent thresholding
                    // ========================================================
                    // Same threshold logic as fixed-arity version:
                    // - Only emit updates when minimum actually decreases
                    // - Suppress redundant updates for non-improving values
                    .threshold_semigroup(|_k, &new_min, current_min| {
                        match current_min {
                            Some(current) if new_min < *current => Some(new_min),
                            Some(_) => None,
                            None if !new_min.is_zero() => Some(new_min),
                            None => None,
                        }
                    })
                        
                    // Phase 3: Convert back to complete FatRow representation
                    // ======================================================
                    // Reconstruct full FatRow with key columns + minimum value
                    // Convert to standard semiring for downstream operators
                    .inner
                    .flat_map(move |(key, t, min_val)| {
                        let mut result = reading::row::FatRow::new();
                            
                        // Add all key columns
                        for i in 0..key.arity() {
                            result.push(key.column(i));
                        }
                            
                        // Push minimized value as the last column
                        result.push(min_val.value as i32);
                            
                        std::iter::once((result, reading::semiring_one())).into_iter().map(move |(x2, d2)| (x2, t.clone(), d2))
                    })
                    .as_collection(),
                idb_catalog.arity() // Preserve original arity for fat relation wrapper
            )
        } else {
            match idb_catalog.arity() {
                #(#arms),*,
                _ => panic!("codegen_min_optimize unimplemented for arity {}", idb_catalog.arity()),
            }
        }
    };

    TokenStream::from(expanded)
}
