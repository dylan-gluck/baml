use crate::conversion::{baml_value_to_json, json_to_baml_value};
use crate::runtime::BamlRuntimeWrapper;
use baml_runtime::RuntimeContext;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

/// WASM wrapper for the BAML runtime
#[wasm_bindgen]
pub struct WasmBamlRuntime {
    runtime: Arc<BamlRuntimeWrapper>,
}

#[wasm_bindgen]
impl WasmBamlRuntime {
    /// Create a new runtime from a directory
    #[wasm_bindgen(constructor)]
    pub fn new(baml_dir: &str, env_vars: JsValue) -> Result<WasmBamlRuntime, JsValue> {
        // Initialize logging
        console_error_panic_hook::set_once();
        
        let env_vars = parse_env_vars(env_vars)?;
        
        match BamlRuntimeWrapper::from_directory(baml_dir, env_vars) {
            Ok(runtime) => Ok(WasmBamlRuntime {
                runtime: Arc::new(runtime),
            }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Create a new runtime from source code
    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(
        baml_src: &str,
        files: JsValue,
        env_vars: JsValue,
    ) -> Result<WasmBamlRuntime, JsValue> {
        // Initialize logging
        console_error_panic_hook::set_once();
        
        let files = parse_files(files)?;
        let env_vars = parse_env_vars(env_vars)?;
        
        match BamlRuntimeWrapper::from_string(baml_src, files, env_vars) {
            Ok(runtime) => Ok(WasmBamlRuntime {
                runtime: Arc::new(runtime),
            }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Call a BAML function synchronously
    #[wasm_bindgen(js_name = callFunction)]
    pub fn call_function(
        &self,
        function_name: &str,
        args: JsValue,
        ctx: JsValue,
    ) -> Result<JsValue, JsValue> {
        // Parse arguments
        let json_args: JsonValue = serde_wasm_bindgen::from_value(args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let baml_args = json_to_baml_value(json_args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Parse context if provided
        let runtime_ctx = if !ctx.is_null() && !ctx.is_undefined() {
            Some(parse_runtime_context(ctx)?)
        } else {
            None
        };
        
        // Call the function
        match self.runtime.call_function(function_name, baml_args, runtime_ctx) {
            Ok(result) => {
                let json_result = baml_value_to_json(result);
                serde_wasm_bindgen::to_value(&json_result)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Call a BAML function asynchronously
    #[wasm_bindgen(js_name = callFunctionAsync)]
    pub fn call_function_async(
        &self,
        function_name: String,
        args: JsValue,
        ctx: JsValue,
    ) -> js_sys::Promise {
        let runtime = self.runtime.clone();
        
        future_to_promise(async move {
            // Parse arguments
            let json_args: JsonValue = serde_wasm_bindgen::from_value(args)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
            let baml_args = json_to_baml_value(json_args)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
            // Parse context if provided
            let runtime_ctx = if !ctx.is_null() && !ctx.is_undefined() {
                Some(parse_runtime_context(ctx)?)
            } else {
                None
            };
            
            // Call the function
            match runtime.call_function(&function_name, baml_args, runtime_ctx) {
                Ok(result) => {
                    let json_result = baml_value_to_json(result);
                    serde_wasm_bindgen::to_value(&json_result)
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                }
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        })
    }

    /// Call a BAML function with streaming support
    #[wasm_bindgen(js_name = callFunctionStreaming)]
    pub fn call_function_streaming(
        &self,
        function_name: &str,
        args: JsValue,
        ctx: JsValue,
    ) -> Result<WasmStreamHandle, JsValue> {
        // Parse arguments
        let json_args: JsonValue = serde_wasm_bindgen::from_value(args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let baml_args = json_to_baml_value(json_args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Parse context if provided
        let runtime_ctx = if !ctx.is_null() && !ctx.is_undefined() {
            Some(parse_runtime_context(ctx)?)
        } else {
            None
        };
        
        // Call the function with streaming
        match self
            .runtime
            .call_function_streaming(function_name, baml_args, runtime_ctx)
        {
            Ok(stream) => Ok(WasmStreamHandle { stream }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Get type definitions
    #[wasm_bindgen(js_name = getTypes)]
    pub fn get_types(&self) -> Result<JsValue, JsValue> {
        match self.runtime.get_types() {
            Ok(types) => serde_wasm_bindgen::to_value(&types)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Get function definitions
    #[wasm_bindgen(js_name = getFunctions)]
    pub fn get_functions(&self) -> Result<JsValue, JsValue> {
        match self.runtime.get_functions() {
            Ok(functions) => serde_wasm_bindgen::to_value(&functions)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}

/// WASM wrapper for stream handle
#[wasm_bindgen]
pub struct WasmStreamHandle {
    stream: crate::runtime::StreamHandle,
}

#[wasm_bindgen]
impl WasmStreamHandle {
    /// Get the next value from the stream
    #[wasm_bindgen(js_name = next)]
    pub fn next(&mut self) -> Result<JsValue, JsValue> {
        match self.stream.next() {
            Some(Ok(value)) => {
                let json_value = baml_value_to_json(value);
                let js_value = serde_wasm_bindgen::to_value(&json_value)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                Ok(js_value)
            }
            Some(Err(e)) => Err(JsValue::from_str(&e.to_string())),
            None => Ok(JsValue::NULL),
        }
    }

    /// Get the next value asynchronously
    #[wasm_bindgen(js_name = nextAsync)]
    pub fn next_async(&mut self) -> js_sys::Promise {
        let result = self.next();
        
        future_to_promise(async move {
            result
        })
    }
}

/// Get version information
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    crate::get_version().to_string()
}

/// Get current log level
#[wasm_bindgen(js_name = getLogLevel)]
pub fn get_log_level() -> String {
    crate::get_log_level().to_string()
}

/// Set log level
#[wasm_bindgen(js_name = setLogLevel)]
pub fn set_log_level(level: &str) -> Result<(), JsValue> {
    crate::set_log_level(level).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Enable or disable JSON logging
#[wasm_bindgen(js_name = setLogJsonMode)]
pub fn set_log_json_mode(enabled: bool) -> Result<(), JsValue> {
    crate::set_log_json_mode(enabled).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Set maximum chunk length for log messages
#[wasm_bindgen(js_name = setLogMaxChunkLength)]
pub fn set_log_max_chunk_length(length: usize) -> Result<(), JsValue> {
    crate::set_log_max_chunk_length(length).map_err(|e| JsValue::from_str(&e.to_string()))
}

// Helper functions

fn parse_env_vars(js_value: JsValue) -> Result<Vec<(String, String)>, JsValue> {
    if js_value.is_null() || js_value.is_undefined() {
        return Ok(Vec::new());
    }
    
    let obj = js_sys::Object::from(js_value);
    let entries = js_sys::Object::entries(&obj);
    let mut env_vars = Vec::new();
    
    for i in 0..entries.length() {
        let entry = entries.get(i);
        let arr = js_sys::Array::from(&entry);
        
        let key = arr.get(0).as_string()
            .ok_or_else(|| JsValue::from_str("Invalid env var key"))?;
        let value = arr.get(1).as_string()
            .ok_or_else(|| JsValue::from_str("Invalid env var value"))?;
        
        env_vars.push((key, value));
    }
    
    Ok(env_vars)
}

fn parse_files(js_value: JsValue) -> Result<IndexMap<String, String>, JsValue> {
    if js_value.is_null() || js_value.is_undefined() {
        return Ok(IndexMap::new());
    }
    
    let obj = js_sys::Object::from(js_value);
    let entries = js_sys::Object::entries(&obj);
    let mut files = IndexMap::new();
    
    for i in 0..entries.length() {
        let entry = entries.get(i);
        let arr = js_sys::Array::from(&entry);
        
        let key = arr.get(0).as_string()
            .ok_or_else(|| JsValue::from_str("Invalid file path"))?;
        let value = arr.get(1).as_string()
            .ok_or_else(|| JsValue::from_str("Invalid file content"))?;
        
        files.insert(key, value);
    }
    
    Ok(files)
}

fn parse_runtime_context(js_value: JsValue) -> Result<RuntimeContext, JsValue> {
    // Parse runtime context from JavaScript object
    // This is a simplified version - expand as needed
    let mut ctx = RuntimeContext::new();
    
    if let Ok(obj) = js_sys::Object::try_from(&js_value) {
        // Handle tags
        if let Ok(tags) = js_sys::Reflect::get(&obj, &JsValue::from_str("tags")) {
            if let Some(arr) = tags.dyn_ref::<js_sys::Array>() {
                for i in 0..arr.length() {
                    if let Some(tag) = arr.get(i).as_string() {
                        ctx.add_tag(&tag);
                    }
                }
            }
        }
        
        // Handle other context properties as needed
        // client_registry, etc.
    }
    
    Ok(ctx)
}

// Required for panic handling in WASM
#[cfg(feature = "javascript")]
pub use console_error_panic_hook;