use internal_baml_core::ir::{Class, Field};
use crate::package::CurrentRenderPackage;

pub struct GleamClass {
    pub name: String,
    pub fields: Vec<GleamField>,
}

pub struct GleamField {
    pub name: String,
    pub r#type: crate::r#type::TypeGleam,
    pub optional: bool,
}

pub fn ir_class_to_gleam(class: &Class, pkg: &CurrentRenderPackage) -> GleamClass {
    let fields = class
        .elem
        .static_fields
        .iter()
        .map(|field| ir_field_to_gleam(field, pkg))
        .collect();

    GleamClass {
        name: class.elem.name.clone(),
        fields,
    }
}

fn ir_field_to_gleam(field: &Field, pkg: &CurrentRenderPackage) -> GleamField {
    let field_type = crate::ir_to_gleam::type_to_gleam(
        &field.elem.r#type.elem.to_non_streaming_type(pkg.ir.as_ref()),
        pkg.ir.as_ref()
    );
    
    let is_optional = field_type.is_optional();
    
    GleamField {
        name: field.elem.name.clone(),
        r#type: field_type,
        optional: is_optional,
    }
}