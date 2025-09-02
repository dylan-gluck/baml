use anyhow::{Context, Result};
use baml_runtime::{BamlRuntime as CoreRuntime, RuntimeContext, RuntimeInterface};
use baml_types::{BamlValue, GeneratorOutputType};
use indexmap::IndexMap;
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

        let inner = tokio_runtime.block_on(async {
            CoreRuntime::from_directory(baml_dir, env_vars, GeneratorOutputType::Gleam).await
        })?;

        Ok(Self {
            inner: Arc::new(inner),
            tokio_runtime: Arc::new(tokio_runtime),
        })
    }

    /// Create a new runtime wrapper from a string
    pub fn from_string(
        baml_src: &str,
        files: IndexMap<String, String>,
        env_vars: Vec<(String, String)>,
    ) -> Result<Self> {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("Failed to create Tokio runtime")?;

        let inner = tokio_runtime.block_on(async {
            CoreRuntime::from_str_with_files(
                baml_src,
                files,
                env_vars,
                GeneratorOutputType::Gleam,
            )
            .await
        })?;

        Ok(Self {
            inner: Arc::new(inner),
            tokio_runtime: Arc::new(tokio_runtime),
        })
    }

    /// Call a BAML function synchronously
    pub fn call_function(
        &self,
        function_name: &str,
        args: BamlValue,
        ctx: Option<RuntimeContext>,
    ) -> Result<BamlValue> {
        let runtime_interface = self.inner.create_runtime_interface()?;
        
        self.tokio_runtime.block_on(async {
            runtime_interface
                .call_function(function_name, &args, &ctx.unwrap_or_default())
                .await
        })
    }

    /// Call a BAML function with streaming support
    pub fn call_function_streaming(
        &self,
        function_name: &str,
        args: BamlValue,
        ctx: Option<RuntimeContext>,
    ) -> Result<StreamHandle> {
        let runtime_interface = self.inner.create_runtime_interface()?;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        let function_name = function_name.to_string();
        let ctx = ctx.unwrap_or_default();
        
        self.tokio_runtime.spawn(async move {
            match runtime_interface
                .call_function_streaming(&function_name, &args, &ctx)
                .await
            {
                Ok(mut stream) => {
                    use futures::StreamExt;
                    while let Some(item) = stream.next().await {
                        if tx.send(item).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        });

        Ok(StreamHandle { receiver: rx })
    }

    /// Get type definitions from the runtime
    pub fn get_types(&self) -> Result<JsonValue> {
        let runtime_interface = self.inner.create_runtime_interface()?;
        
        // Get type information from the runtime
        let types = self.tokio_runtime.block_on(async {
            runtime_interface.get_type_schemas()
        })?;

        Ok(serde_json::to_value(types)?)
    }

    /// Get function definitions from the runtime
    pub fn get_functions(&self) -> Result<JsonValue> {
        let runtime_interface = self.inner.create_runtime_interface()?;
        
        // Get function information from the runtime
        let functions = self.tokio_runtime.block_on(async {
            runtime_interface.get_function_schemas()
        })?;

        Ok(serde_json::to_value(functions)?)
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