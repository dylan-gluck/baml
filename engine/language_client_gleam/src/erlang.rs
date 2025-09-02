use crate::conversion::erlang_conversion::{baml_value_to_term, term_to_baml_value};
use crate::runtime::{BamlRuntimeWrapper, StreamHandle};
use anyhow::Result;
use baml_runtime::RuntimeContext;
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use rustler::{Atom, Encoder, Env, Error as NifError, NifResult, ResourceArc, Term};
use std::sync::{Arc, Mutex};

// Resource types for Rustler
struct RuntimeResource {
    runtime: Arc<BamlRuntimeWrapper>,
}

struct StreamResource {
    stream: Arc<Mutex<StreamHandle>>,
}

// Global runtime storage (for simplicity, can be improved)
static RUNTIME: OnceCell<Arc<BamlRuntimeWrapper>> = OnceCell::new();

/// Initialize the BAML runtime from a directory
#[rustler::nif]
fn init_runtime_from_directory<'a>(
    env: Env<'a>,
    baml_dir: String,
    env_vars: Vec<(String, String)>,
) -> NifResult<Term<'a>> {
    match BamlRuntimeWrapper::from_directory(&baml_dir, env_vars) {
        Ok(runtime) => {
            let runtime_arc = Arc::new(runtime);
            RUNTIME.set(runtime_arc.clone()).ok();
            
            let resource = ResourceArc::new(RuntimeResource {
                runtime: runtime_arc,
            });
            
            Ok((ok_atom(env), resource).encode(env))
        }
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Initialize the BAML runtime from a string
#[rustler::nif]
fn init_runtime_from_string<'a>(
    env: Env<'a>,
    baml_src: String,
    files: Vec<(String, String)>,
    env_vars: Vec<(String, String)>,
) -> NifResult<Term<'a>> {
    let files_map: IndexMap<String, String> = files.into_iter().collect();
    
    match BamlRuntimeWrapper::from_string(&baml_src, files_map, env_vars) {
        Ok(runtime) => {
            let runtime_arc = Arc::new(runtime);
            RUNTIME.set(runtime_arc.clone()).ok();
            
            let resource = ResourceArc::new(RuntimeResource {
                runtime: runtime_arc,
            });
            
            Ok((ok_atom(env), resource).encode(env))
        }
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Call a BAML function synchronously
#[rustler::nif]
fn call_function<'a>(
    env: Env<'a>,
    runtime_resource: ResourceArc<RuntimeResource>,
    function_name: String,
    args: Term<'a>,
    ctx: Term<'a>,
) -> NifResult<Term<'a>> {
    // Convert Erlang term to BAML value
    let baml_args = match term_to_baml_value(env, args) {
        Ok(v) => v,
        Err(e) => return Ok((error_atom(env), e.to_string()).encode(env)),
    };
    
    // Parse context if provided
    let runtime_ctx = if !ctx.is_atom() || ctx.decode::<Atom>()?.name() != "nil" {
        match parse_runtime_context(env, ctx) {
            Ok(c) => Some(c),
            Err(e) => return Ok((error_atom(env), e.to_string()).encode(env)),
        }
    } else {
        None
    };
    
    // Call the function
    match runtime_resource.runtime.call_function(&function_name, baml_args, runtime_ctx) {
        Ok(result) => {
            let term_result = baml_value_to_term(env, result);
            Ok((ok_atom(env), term_result).encode(env))
        }
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Call a BAML function with streaming support
#[rustler::nif]
fn call_function_streaming<'a>(
    env: Env<'a>,
    runtime_resource: ResourceArc<RuntimeResource>,
    function_name: String,
    args: Term<'a>,
    ctx: Term<'a>,
) -> NifResult<Term<'a>> {
    // Convert Erlang term to BAML value
    let baml_args = match term_to_baml_value(env, args) {
        Ok(v) => v,
        Err(e) => return Ok((error_atom(env), e.to_string()).encode(env)),
    };
    
    // Parse context if provided
    let runtime_ctx = if !ctx.is_atom() || ctx.decode::<Atom>()?.name() != "nil" {
        match parse_runtime_context(env, ctx) {
            Ok(c) => Some(c),
            Err(e) => return Ok((error_atom(env), e.to_string()).encode(env)),
        }
    } else {
        None
    };
    
    // Call the function with streaming
    match runtime_resource
        .runtime
        .call_function_streaming(&function_name, baml_args, runtime_ctx)
    {
        Ok(stream) => {
            let stream_resource = ResourceArc::new(StreamResource {
                stream: Arc::new(Mutex::new(stream)),
            });
            Ok((ok_atom(env), stream_resource).encode(env))
        }
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Get the next value from a stream
#[rustler::nif]
fn stream_next<'a>(env: Env<'a>, stream_resource: ResourceArc<StreamResource>) -> NifResult<Term<'a>> {
    let mut stream = stream_resource
        .stream
        .lock()
        .map_err(|_| NifError::BadArg)?;
    
    match stream.next() {
        Some(Ok(value)) => {
            let term_value = baml_value_to_term(env, value);
            Ok((ok_atom(env), term_value).encode(env))
        }
        Some(Err(e)) => Ok((error_atom(env), e.to_string()).encode(env)),
        None => Ok((Atom::from_str(env, "done").unwrap(),).encode(env)),
    }
}

/// Get type definitions from the runtime
#[rustler::nif]
fn get_types<'a>(env: Env<'a>, runtime_resource: ResourceArc<RuntimeResource>) -> NifResult<Term<'a>> {
    match runtime_resource.runtime.get_types() {
        Ok(types) => {
            // Convert JSON to string for simplicity
            let json_str = types.to_string();
            Ok((ok_atom(env), json_str).encode(env))
        }
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Get function definitions from the runtime
#[rustler::nif]
fn get_functions<'a>(
    env: Env<'a>,
    runtime_resource: ResourceArc<RuntimeResource>,
) -> NifResult<Term<'a>> {
    match runtime_resource.runtime.get_functions() {
        Ok(functions) => {
            // Convert JSON to string for simplicity
            let json_str = functions.to_string();
            Ok((ok_atom(env), json_str).encode(env))
        }
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Get version information
#[rustler::nif]
fn get_version() -> &'static str {
    crate::get_version()
}

/// Get current log level
#[rustler::nif]
fn get_log_level() -> &'static str {
    crate::get_log_level()
}

/// Set log level
#[rustler::nif]
fn set_log_level<'a>(env: Env<'a>, level: String) -> NifResult<Term<'a>> {
    match crate::set_log_level(&level) {
        Ok(()) => Ok(ok_atom(env).encode(env)),
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

/// Set JSON logging mode
#[rustler::nif]
fn set_log_json_mode<'a>(env: Env<'a>, enabled: bool) -> NifResult<Term<'a>> {
    match crate::set_log_json_mode(enabled) {
        Ok(()) => Ok(ok_atom(env).encode(env)),
        Err(e) => Ok((error_atom(env), e.to_string()).encode(env)),
    }
}

// Helper functions
fn ok_atom(env: Env) -> Atom {
    Atom::from_str(env, "ok").unwrap()
}

fn error_atom(env: Env) -> Atom {
    Atom::from_str(env, "error").unwrap()
}

fn parse_runtime_context<'a>(env: Env<'a>, term: Term<'a>) -> Result<RuntimeContext> {
    // Parse runtime context from Erlang term
    // This is a simplified version - expand as needed
    let map: rustler::types::map::MapIterator = term
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode context: {:?}", e))?;
    
    let mut ctx = RuntimeContext::new();
    
    for (key, value) in map {
        let key_str: String = key
            .decode()
            .map_err(|e| anyhow::anyhow!("Failed to decode context key: {:?}", e))?;
        
        if key_str == "client_registry" {
            // Handle client registry
            // This would need proper implementation based on BAML's requirements
        } else if key_str == "tags" {
            // Handle tags
            let tags: Vec<String> = value
                .decode()
                .map_err(|e| anyhow::anyhow!("Failed to decode tags: {:?}", e))?;
            for tag in tags {
                ctx.add_tag(&tag);
            }
        }
    }
    
    Ok(ctx)
}

// Resource implementation for Rustler
impl rustler::resource::Resource for RuntimeResource {}
impl rustler::resource::Resource for StreamResource {}

// Initialize the NIF module
rustler::init!(
    "baml_gleam_ffi",
    [
        init_runtime_from_directory,
        init_runtime_from_string,
        call_function,
        call_function_streaming,
        stream_next,
        get_types,
        get_functions,
        get_version,
        get_log_level,
        set_log_level,
        set_log_json_mode,
    ],
    load = on_load
);

fn on_load(env: Env, _info: Term) -> bool {
    rustler::resource!(RuntimeResource, env);
    rustler::resource!(StreamResource, env);
    
    // Initialize logging
    crate::init_baml_logging().ok();
    
    true
}