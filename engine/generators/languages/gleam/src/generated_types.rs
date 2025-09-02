use crate::package::CurrentRenderPackage;
use anyhow::Result;

pub fn render_types(_pkg: &CurrentRenderPackage) -> Result<String> {
    Ok(String::from("//// Generated BAML types\n"))
}