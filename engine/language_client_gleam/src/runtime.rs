use anyhow::{Context, Result};
use baml_runtime::{BamlRuntime as CoreRuntime, RuntimeContext};
use baml_types::BamlValue;
use internal_baml_core::feature_flags::FeatureFlags;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;

/// Wrapper around the BAML runtime for FFI usage
pub struct BamlRuntimeWrapper {
    inner: Arc<CoreRuntime>,
    tokio_runtime: Arc<TokioRuntime>,
}

impl BamlRuntimeWrapper {
    /// Create a new runtime wrapper from BAML directory
    pub fn from_directory(baml_dir: &str, env_vars: Vec<(String, String)>) -> Result<Self> {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("Failed to create Tokio runtime")?;

        let inner = CoreRuntime::from_directory(
            std::path::Path::new(baml_dir), 
            env_vars.into_iter().collect(),
            FeatureFlags::new()
        )?;

        Ok(Self {
            inner: Arc::new(inner),
            tokio_runtime: Arc::new(tokio_runtime),
        })
    }

    // NOTE: from_string is not yet implemented
    // The from_str_with_files method is not available in the public API
    // This functionality will need to be added when the API is available
    /*
    /// Create a new runtime wrapper from a string
    pub fn from_string(
        baml_src: &str,
        files: IndexMap<String, String>,
        env_vars: Vec<(String, String)>,
    ) -> Result<Self> {
        // TODO: Implement when API is available
        unimplemented!("from_string is not yet implemented")
    }
    */

    /// Call a BAML function synchronously
    pub fn call_function(
        &self,
        function_name: &str,
        args: BamlValue,
        ctx: Option<RuntimeContext>,
    ) -> Result<BamlValue> {
        // TODO: Implement actual function calling when API is available
        // The runtime interface API needs to be properly exposed
        let _ = (function_name, args, ctx);
        unimplemented!("call_function is not yet implemented")
    }

    /// Call a BAML function with streaming support
    pub fn call_function_streaming(
        &self,
        function_name: &str,
        args: BamlValue,
        ctx: Option<RuntimeContext>,
    ) -> Result<StreamHandle> {
        // TODO: Implement streaming when API is available
        let _ = (function_name, args, ctx);
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Ok(StreamHandle { receiver: rx })
    }

    /// Get type definitions from the runtime
    pub fn get_types(&self) -> Result<JsonValue> {
        // TODO: Implement when API is available
        Ok(serde_json::json!({
            "types": []
        }))
    }

    /// Get function definitions from the runtime
    pub fn get_functions(&self) -> Result<JsonValue> {
        // TODO: Implement when API is available  
        Ok(serde_json::json!({
            "functions": []
        }))
    }
}

/// Handle for streaming responses
pub struct StreamHandle {
    receiver: tokio::sync::mpsc::UnboundedReceiver<Result<BamlValue>>,
}

impl StreamHandle {
    /// Get the next value from the stream
    pub fn next(&mut self) -> Option<Result<BamlValue>> {
        self.receiver.blocking_recv()
    }
}