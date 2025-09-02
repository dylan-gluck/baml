use internal_baml_core::ir::TypeAlias;
use crate::package::CurrentRenderPackage;

pub struct GleamTypeAlias {
    pub name: String,
    pub r#type: crate::r#type::TypeGleam,
}

pub fn ir_type_alias_to_gleam(alias: &TypeAlias, pkg: &CurrentRenderPackage) -> GleamTypeAlias {
    let gleam_type = crate::ir_to_gleam::type_to_gleam(
        &alias.elem.r#type.elem.to_non_streaming_type(pkg.ir.as_ref()),
        pkg.ir.as_ref()
    );
    
    GleamTypeAlias {
        name: alias.elem.name.clone(),
        r#type: gleam_type,
    }
}