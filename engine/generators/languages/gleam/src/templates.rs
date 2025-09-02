use anyhow::Result;
use askama::Template;
use dir_writer::GeneratorArgs;
use internal_baml_core::ir::repr::IntermediateRepr;

mod filters {
    pub fn escape_string(s: &str, _: &dyn askama::Values) -> askama::Result<String> {
        // Escape string for Gleam string literals
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        Ok(escaped)
    }
}

#[derive(Template)]
#[template(path = "runtime.gleam.j2")]
pub struct GleamRuntime<'a> {
    ir: &'a IntermediateRepr,
    args: &'a GeneratorArgs,
    inlined_files: Vec<(String, String)>,
}

impl<'a> GleamRuntime<'a> {
    pub fn render(&self) -> Result<String> {
        Ok(self.render_template()?)
    }

    fn render_template(&self) -> Result<String, askama::Error> {
        <Self as Template>::render(self)
    }
}

impl<'a> TryFrom<(&'a IntermediateRepr, &'a GeneratorArgs)> for GleamRuntime<'a> {
    type Error = anyhow::Error;

    fn try_from((ir, args): (&'a IntermediateRepr, &'a GeneratorArgs)) -> Result<Self> {
        // TODO: Collect actual BAML files if needed
        let inlined_files = Vec::new();
        Ok(Self { ir, args, inlined_files })
    }
}

#[derive(Template, Default)]
#[template(path = "ffi.gleam.j2")]
pub struct GleamFFI;

impl GleamFFI {
    pub fn render(&self) -> Result<String> {
        Ok(self.render_template()?)
    }

    fn render_template(&self) -> Result<String, askama::Error> {
        <Self as Template>::render(self)
    }
}
