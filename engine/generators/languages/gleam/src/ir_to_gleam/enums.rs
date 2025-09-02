use internal_baml_core::ir::Enum;
use crate::package::CurrentRenderPackage;

pub struct GleamEnum {
    pub name: String,
    pub values: Vec<GleamEnumValue>,
}

pub struct GleamEnumValue {
    pub name: String,
    pub alias: Option<String>,
}

pub fn ir_enum_to_gleam(enum_def: &Enum, _pkg: &CurrentRenderPackage) -> GleamEnum {
    let values = enum_def
        .elem
        .values
        .iter()
        .map(|(value, _docstring)| GleamEnumValue {
            name: value.elem.0.clone(),
            alias: None, // Enum values in BAML don't have aliases directly
        })
        .collect();

    GleamEnum {
        name: enum_def.elem.name.clone(),
        values,
    }
}