use proc_macro::TokenStream;
use quote::quote;
use itertools::iproduct;
use syn::Ident;
use proc_macro2::Span;

const KV_MAX: usize = 2;
const ROW_MAX: usize = 2;
const PROD_MAX: usize = 2;

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
        match (iv, target) {
            #(#arms),*,
            _ => panic!("codegen_row_row unimplemented for {}, {}", iv, target),
        }
    };

    TokenStream::from(expanded)
}


/* row → kv */
#[proc_macro]
pub fn codegen_row_kv(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=ROW_MAX, 1..=KV_MAX, 1..=KV_MAX)
        .filter(|&(iv, ok, ov)| iv >= ok + ov);
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
        match (iv, ok, ov) {
            #(#arms),*,
            _ => panic!("codegen_row_kv unimplemented for {}, {}, {}", iv, ok, ov),
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
        match (ik0, iv0, iv1, target) {
            #(#arms),*,
            _ => panic!("codegen_jn unimplemented for {}, {}, {}, {}", ik0, iv0, iv1, target),
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
        match (iv0, iv1, target) {
            #(#arms),*,
            _ => panic!("codegen_cartesian unimplemented for {}, {}, {}", iv0, iv1, target),
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
        match (ik0, iv0, target) {
            #(#arms),*,
            _ => panic!("cpdegen_kv_k_jn unimplemented for {}, {}, {}", ik0, iv0, target),
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
        match (ik0, target) {
            #(#arms),*,
            _ => panic!("codegen_k_k_jn unimplemented for {}, {}", ik0, target),
        }
    };

    TokenStream::from(expanded)
}




/* ------------------------------------------------------------------------ */
/* codegen for aj flatten */
/* ------------------------------------------------------------------------ */

#[proc_macro]
pub fn codegen_kv_flatten(_: TokenStream) -> TokenStream {
    let space = iproduct!(1..=KV_MAX, 1..=KV_MAX, 1..=KV_MAX).filter(|&(ik, iv, target)| ik + iv >= target);

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
        match (ik0, iv0, target) {
            #(#arms),*,
            _ => panic!("codegen_kv_flatten unimplemented for {}, {}, {}", ik0, iv0, target),
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
        match (ik0, target) {
            #(#arms),*,
            _ => panic!("codegen_k_flatten unimplemented for {}, {}", ik0, target),
        }
    };

    TokenStream::from(expanded)
}