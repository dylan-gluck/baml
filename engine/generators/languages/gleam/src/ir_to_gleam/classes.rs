use internal_baml_core::ir::ClassWalker;

use crate::{
    generated_types::{ClassGleam, FieldGleam},
    package::CurrentRenderPackage,
};

pub fn ir_class_to_gleam<'a>(class: &ClassWalker, pkg: &'a CurrentRenderPackage) -> ClassGleam<'a> {
    let fields = class
        .walk_fields()
        .filter(|f| !f.item.attributes.dynamic())
        .map(|field| {
            let field_type_ir = field.r#type();
            let field_non_streaming = field_type_ir.to_non_streaming_type(pkg.ir.as_ref());
            let field_type = super::type_to_gleam(&field_non_streaming, pkg);
            FieldGleam {
                docstring: field.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
                name: field.name().to_string(),
                r#type: field_type,
                pkg,
            }
        })
        .collect();

    ClassGleam {
        name: class.name().to_string(),
        docstring: class.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
        fields,
        dynamic: class.walk_fields().any(|f| f.item.attributes.dynamic()),
        pkg,
    }
}

pub fn ir_class_to_gleam_stream<'a>(class: &ClassWalker, pkg: &'a CurrentRenderPackage) -> ClassGleam<'a> {
    let fields = class
        .walk_fields()
        .filter(|f| !f.item.attributes.dynamic())
        .map(|field| {
            let field_type_ir = field.r#type();
            let field_streaming = field_type_ir.to_streaming_type(pkg.ir.as_ref());
            let field_type = super::stream_type_to_gleam(&field_streaming, pkg);
            FieldGleam {
                docstring: field.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
                name: field.name().to_string(),
                r#type: field_type,
                pkg,
            }
        })
        .collect();

    ClassGleam {
        name: class.name().to_string(),
        docstring: class.item.attributes.get("description").and_then(|v| v.as_string_value(&Default::default()).ok()).flatten(),
        fields,
        dynamic: class.walk_fields().any(|f| f.item.attributes.dynamic()),
        pkg,
    }
}