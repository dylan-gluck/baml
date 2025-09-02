use dir_writer::IntermediateRepr;
use std::sync::Arc;

pub struct CurrentRenderPackage {
    pub name: String,
    pub ir: Arc<IntermediateRepr>,
    pub current_module: String,
}

impl CurrentRenderPackage {
    pub fn new(name: &str, ir: Arc<IntermediateRepr>) -> Self {
        Self {
            name: name.to_string(),
            ir,
            current_module: String::from("baml_client"),
        }
    }

    pub fn set(&mut self, module: &str) {
        self.current_module = module.to_string();
    }
}