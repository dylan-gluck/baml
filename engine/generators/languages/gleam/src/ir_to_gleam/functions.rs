use internal_baml_core::ir::FunctionNode;
use crate::package::CurrentRenderPackage;

pub struct GleamFunction {
    pub name: String,
    pub params: Vec<GleamParam>,
    pub return_type: crate::r#type::TypeGleam,
}

pub struct GleamParam {
    pub name: String,
    pub r#type: crate::r#type::TypeGleam,
}

pub fn ir_function_to_gleam(function: &FunctionNode, pkg: &CurrentRenderPackage) -> GleamFunction {
    let params = function
        .elem
        .inputs()
        .iter()
        .map(|(name, field_type)| {
            let param_type = crate::ir_to_gleam::type_to_gleam(
                &field_type.to_non_streaming_type(pkg.ir.as_ref()),
                pkg.ir.as_ref()
            );
            GleamParam {
                name: name.clone(),
                r#type: param_type,
            }
        })
        .collect();

    let return_type = crate::ir_to_gleam::type_to_gleam(
        &function.elem.output().to_non_streaming_type(pkg.ir.as_ref()),
        pkg.ir.as_ref()
    );

    GleamFunction {
        name: function.elem.name().to_string(),
        params,
        return_type,
    }
}