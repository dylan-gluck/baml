use std::fmt;

/// Error type for BAML Gleam FFI
#[derive(Debug)]
pub enum BamlError {
    /// Runtime initialization error
    InitError(String),
    /// Function call error
    CallError(String),
    /// Type conversion error
    ConversionError(String),
    /// Streaming error
    StreamError(String),
    /// Generic error
    Other(String),
}

impl fmt::Display for BamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BamlError::InitError(msg) => write!(f, "Runtime initialization error: {}", msg),
            BamlError::CallError(msg) => write!(f, "Function call error: {}", msg),
            BamlError::ConversionError(msg) => write!(f, "Type conversion error: {}", msg),
            BamlError::StreamError(msg) => write!(f, "Streaming error: {}", msg),
            BamlError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for BamlError {}

impl From<anyhow::Error> for BamlError {
    fn from(err: anyhow::Error) -> Self {
        BamlError::Other(err.to_string())
    }
}

impl From<serde_json::Error> for BamlError {
    fn from(err: serde_json::Error) -> Self {
        BamlError::ConversionError(err.to_string())
    }
}

impl From<baml_runtime::errors::ExposedError> for BamlError {
    fn from(err: baml_runtime::errors::ExposedError) -> Self {
        BamlError::CallError(err.to_string())
    }
}

#[cfg(feature = "erlang")]
impl From<BamlError> for rustler::Error {
    fn from(err: BamlError) -> Self {
        rustler::Error::Term(Box::new(err.to_string()))
    }
}

#[cfg(feature = "javascript")]
impl From<BamlError> for wasm_bindgen::JsValue {
    fn from(err: BamlError) -> Self {
        wasm_bindgen::JsValue::from_str(&err.to_string())
    }
}

/// Result type for BAML operations
pub type BamlResult<T> = Result<T, BamlError>;