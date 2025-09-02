use internal_baml_core::ir::EnumWalker;

use crate::{
    generated_types::EnumGleam,
    package::CurrentRenderPackage,
};

pub fn ir_enum_to_gleam<'a>(enum_def: &EnumWalker, pkg: &'a CurrentRenderPackage) -> EnumGleam<'a> {
    let values = enum_def
        .walk_values()
        .map(|value| {
            (
                value.name().to_string(),
                value.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
            )
        })
        .collect();

    EnumGleam {
        name: enum_def.name().to_string(),
        docstring: enum_def.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
        values,
        dynamic: false, // Gleam enums are not dynamic
        pkg,
    }
}