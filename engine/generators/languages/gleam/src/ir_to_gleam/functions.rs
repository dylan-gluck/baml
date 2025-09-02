use internal_baml_core::ir::FunctionWalker;

use crate::{
    functions::FunctionGleam,
    package::CurrentRenderPackage,
};

pub fn ir_function_to_gleam(function: &FunctionWalker, pkg: &CurrentRenderPackage) -> FunctionGleam {
    let args = function
        .inputs()
        .iter()
        .map(|(name, r#type)| {
            let non_streaming = r#type.to_non_streaming_type(pkg.ir.as_ref());
            let gleam_type = super::type_to_gleam(&non_streaming, pkg);
            (name.clone(), gleam_type)
        })
        .collect();

    let output_non_streaming = function.output().to_non_streaming_type(pkg.ir.as_ref());
    let output_streaming = function.output().to_streaming_type(pkg.ir.as_ref());
    
    let return_type = super::type_to_gleam(&output_non_streaming, pkg);
    let stream_return_type = super::stream_type_to_gleam(&output_streaming, pkg);

    FunctionGleam {
        documentation: None,
        name: function.name().to_string(),
        args,
        return_type,
        stream_return_type,
    }
}