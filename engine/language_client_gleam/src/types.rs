use baml_runtime::RuntimeContext;
use baml_runtime::tracing::TracingCall;
use baml_runtime::type_builder::TypeBuilder as CoreTypeBuilder;
use baml_types::{BamlMedia, BamlValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result from a BAML function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    /// The returned value
    pub value: BamlValue,
    /// Metadata about the function call
    pub metadata: FunctionMetadata,
}

/// Metadata about a function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Function name
    pub function_name: String,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Tracing span ID if available
    pub span_id: Option<String>,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Streaming result from a BAML function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResult {
    /// Current partial value
    pub partial: Option<BamlValue>,
    /// Whether the stream is complete
    pub is_complete: bool,
    /// Any errors encountered
    pub error: Option<String>,
}

/// Type builder for dynamic type creation
pub struct TypeBuilder {
    inner: CoreTypeBuilder,
}

impl TypeBuilder {
    pub fn new() -> Self {
        Self {
            inner: CoreTypeBuilder::new(),
        }
    }

    // TODO: Implement when TypeBuilder API is available
    // pub fn add_enum(&mut self, name: String, values: Vec<String>) -> &mut Self {
    //     self.inner.add_enum(name, values);
    //     self
    // }

    // pub fn add_class(&mut self, name: String, fields: HashMap<String, String>) -> &mut Self {
    //     self.inner.add_class(name, fields);
    //     self
    // }

    pub fn build(self) -> CoreTypeBuilder {
        self.inner
    }
}

/// Client registry for managing LLM clients
#[derive(Debug, Clone)]
pub struct ClientRegistry {
    clients: HashMap<String, ClientConfig>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, name: String, config: ClientConfig) {
        self.clients.insert(name, config);
    }

    pub fn get_client(&self, name: &str) -> Option<&ClientConfig> {
        self.clients.get(name)
    }

    pub fn list_clients(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }
}

/// Configuration for an LLM client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub options: HashMap<String, serde_json::Value>,
}

/// Image data for BAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamlImage {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl From<BamlImage> for BamlMedia {
    fn from(img: BamlImage) -> Self {
        use baml_types::BamlMediaType;
        // Convert raw data to base64
        let base64_data = base64::encode(&img.data);
        BamlMedia::base64(
            BamlMediaType::Image,
            base64_data,
            Some(img.mime_type),
        )
    }
}

/// Audio data for BAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamlAudio {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl From<BamlAudio> for BamlMedia {
    fn from(audio: BamlAudio) -> Self {
        use baml_types::BamlMediaType;
        // Convert raw data to base64
        let base64_data = base64::encode(&audio.data);
        BamlMedia::base64(
            BamlMediaType::Audio,
            base64_data,
            Some(audio.mime_type),
        )
    }
}

/// PDF data for BAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamlPdf {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl From<BamlPdf> for BamlMedia {
    fn from(pdf: BamlPdf) -> Self {
        use baml_types::BamlMediaType;
        // Convert raw data to base64
        let base64_data = base64::encode(&pdf.data);
        BamlMedia::base64(
            BamlMediaType::Pdf,
            base64_data,
            Some(pdf.mime_type),
        )
    }
}

/// Video data for BAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamlVideo {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl From<BamlVideo> for BamlMedia {
    fn from(video: BamlVideo) -> Self {
        use baml_types::BamlMediaType;
        // Convert raw data to base64
        let base64_data = base64::encode(&video.data);
        BamlMedia::base64(
            BamlMediaType::Video,
            base64_data,
            Some(video.mime_type),
        )
    }
}

/// Tracing span for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamlSpan {
    pub span_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub attributes: HashMap<String, serde_json::Value>,
}

// Note: BamlSpan should wrap TracingCall from baml_runtime
// This is a simplified representation for FFI purposes
impl BamlSpan {
    pub fn from_tracing_call(_call: Option<TracingCall>) -> Option<Self> {
        // TODO: Implement proper conversion from TracingCall
        // For now, return None as TracingCall details are internal
        None
    }
}

/// Runtime context builder
pub struct RuntimeContextBuilder {
    tags: Vec<String>,
    client_registry: Option<ClientRegistry>,
}

impl RuntimeContextBuilder {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            client_registry: None,
        }
    }

    pub fn add_tag(&mut self, tag: String) -> &mut Self {
        self.tags.push(tag);
        self
    }

    pub fn with_client_registry(&mut self, registry: ClientRegistry) -> &mut Self {
        self.client_registry = Some(registry);
        self
    }

    pub fn build(self) -> RuntimeContext {
        use std::sync::Arc;
        use std::collections::HashMap;
        use indexmap::IndexMap;
        
        // Convert tags Vec<String> to HashMap<String, BamlValue>
        let mut tags_map = HashMap::new();
        for tag in self.tags {
            tags_map.insert(tag.clone(), BamlValue::String(tag));
        }
        
        RuntimeContext::new(
            Arc::new(None), // baml_src
            HashMap::new(), // env
            tags_map, // tags
            None, // client_overrides
            IndexMap::new(), // class_override
            IndexMap::new(), // enum_overrides
            IndexMap::new(), // type_alias_overrides
            Vec::new(), // recursive_class_overrides
            Vec::new(), // recursive_type_alias_overrides
            Vec::new(), // call_id_stack
        )
    }
}