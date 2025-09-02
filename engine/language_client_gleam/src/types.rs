use baml_runtime::{BamlSpan as CoreBamlSpan, RuntimeContext};
use baml_types::{BamlMedia, BamlValue, TypeBuilder as CoreTypeBuilder};
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

    pub fn add_enum(&mut self, name: String, values: Vec<String>) -> &mut Self {
        self.inner.add_enum(name, values);
        self
    }

    pub fn add_class(&mut self, name: String, fields: HashMap<String, String>) -> &mut Self {
        self.inner.add_class(name, fields);
        self
    }

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
        BamlMedia::Image(baml_types::media::Image {
            mime_type: img.mime_type,
            data: img.data,
        })
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
        BamlMedia::Audio(baml_types::media::Audio {
            mime_type: audio.mime_type,
            data: audio.data,
        })
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
        BamlMedia::Pdf(baml_types::media::Pdf {
            mime_type: pdf.mime_type,
            data: pdf.data,
        })
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
        BamlMedia::Video(baml_types::media::Video {
            mime_type: video.mime_type,
            data: video.data,
        })
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

impl From<CoreBamlSpan> for BamlSpan {
    fn from(span: CoreBamlSpan) -> Self {
        BamlSpan {
            span_id: span.span_id,
            parent_id: span.parent_id,
            name: span.name,
            start_time: span.start_time,
            end_time: span.end_time,
            attributes: span.attributes,
        }
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
        let mut ctx = RuntimeContext::new();
        for tag in self.tags {
            ctx.add_tag(&tag);
        }
        // Note: ClientRegistry integration would need to be properly implemented
        // based on BAML's actual requirements
        ctx
    }
}