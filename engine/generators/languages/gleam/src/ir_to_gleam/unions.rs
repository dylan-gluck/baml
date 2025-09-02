use baml_types::ir_type::TypeNonStreaming;

use crate::{
    generated_types::{UnionGleam, VariantGleam},
    package::CurrentRenderPackage,
};

// Union types are handled differently in BAML - they come from TypeNonStreaming::Union
pub fn ir_union_to_gleam<'a>(
    union_type: &TypeNonStreaming,
    pkg: &'a CurrentRenderPackage,
) -> Option<UnionGleam<'a>> {
    match union_type {
        TypeNonStreaming::Union(variants, _) => {
            let variants_vec = variants.iter_include_null();
            let name = format!("Union{}", variants_vec.len());
            let cffi_name = format!("Union{}", name);

            let union_variants = variants_vec
                .into_iter()
                .enumerate()
                .map(|(idx, variant)| {
                    let variant_name = format!("Variant{}", idx);
                    let type_ = super::type_to_gleam(variant, pkg);
                    let cffi_variant_name = format!("{}_{}", cffi_name, variant_name);

                    VariantGleam {
                        name: variant_name,
                        cffi_name: cffi_variant_name,
                        literal_repr: None,
                        type_,
                    }
                })
                .collect();

            Some(UnionGleam {
                name,
                cffi_name,
                docstring: None,
                variants: union_variants,
                pkg,
            })
        }
        _ => None,
    }
}