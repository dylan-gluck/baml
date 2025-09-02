use crate::package::CurrentRenderPackage;
use anyhow::Result;

pub fn render_client(_pkg: &CurrentRenderPackage) -> Result<String> {
    Ok(String::from("//// Generated BAML client\n"))
}

pub fn render_runtime(_pkg: &CurrentRenderPackage) -> Result<String> {
    Ok(String::from("//// Generated BAML runtime\n"))
}

pub fn render_ffi(_pkg: &CurrentRenderPackage) -> Result<String> {
    Ok(String::from("//// Generated BAML FFI bindings\n"))
}