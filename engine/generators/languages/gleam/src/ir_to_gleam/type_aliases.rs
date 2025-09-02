use internal_baml_core::ir::TypeAliasWalker;

use crate::{
    generated_types::TypeAliasGleam,
    package::CurrentRenderPackage,
};

pub fn ir_type_alias_to_gleam<'a>(
    alias: &TypeAliasWalker,
    pkg: &'a CurrentRenderPackage,
    invalid_cycle: Option<&String>,
) -> TypeAliasGleam<'a> {
    let gleam_type = if let Some(_) = invalid_cycle {
        // If there's an invalid cycle, use Dynamic type
        crate::r#type::TypeGleam::Dynamic {
            reason: format!("Recursive type alias {} has an invalid cycle", alias.name()),
            meta: crate::r#type::TypeMetaGleam::default(),
        }
    } else {
        let target_non_streaming = alias.value().to_non_streaming_type(pkg.ir.as_ref());
        super::type_to_gleam(&target_non_streaming, pkg)
    };
    
    TypeAliasGleam {
        name: alias.name().to_string(),
        type_: gleam_type,
        docstring: alias.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
        pkg,
    }
}