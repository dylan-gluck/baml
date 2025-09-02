use std::fmt;

use baml_types::{BamlMediaType, TypeValue};

use crate::package::CurrentRenderPackage;

/// Type wrapper for Gleam types (similar to Go's TypeWrapper)
#[derive(Clone, Debug, PartialEq)]
pub enum TypeWrapper {
    None,
    Checked(Box<TypeWrapper>),     // For validation constraints
    Optional(Box<TypeWrapper>),    // For nullable types
}

/// Type metadata for Gleam types (similar to Go's TypeMetaGo)
#[derive(Clone, Debug, PartialEq)]
pub struct TypeMetaGleam {
    pub type_wrapper: TypeWrapper,
    pub wrap_stream_state: bool,   // For streaming support
}

impl Default for TypeMetaGleam {
    fn default() -> Self {
        TypeMetaGleam {
            type_wrapper: TypeWrapper::None,
            wrap_stream_state: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeGleam {
    // Primitive types
    String(Option<String>, TypeMetaGleam),
    Int(Option<i64>, TypeMetaGleam),
    Float(TypeMetaGleam),
    Bool(Option<bool>, TypeMetaGleam),
    Nil(TypeMetaGleam),
    
    // Media types
    Media(MediaTypeGleam, TypeMetaGleam),
    
    // Custom types
    Class { 
        module: String, 
        name: String,
        dynamic: bool,
        meta: TypeMetaGleam,
    },
    Enum { 
        module: String, 
        name: String,
        dynamic: bool,
        meta: TypeMetaGleam,
    },
    
    // Container types
    List(Box<TypeGleam>, TypeMetaGleam),
    Dict(Box<TypeGleam>, Box<TypeGleam>, TypeMetaGleam),
    
    // Type aliases
    TypeAlias {
        module: String,
        name: String,
        meta: TypeMetaGleam,
    },
    
    // For union types
    Union {
        module: String,
        name: String,
        meta: TypeMetaGleam,
    },
    
    // For any type we can't represent
    Dynamic {
        reason: String,
        meta: TypeMetaGleam,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaTypeGleam {
    Image,
    Audio,
    Pdf,
    Video,
}

impl TypeGleam {
    pub fn is_optional(&self) -> bool {
        match self {
            TypeGleam::String(_, meta) |
            TypeGleam::Int(_, meta) |
            TypeGleam::Float(meta) |
            TypeGleam::Bool(_, meta) |
            TypeGleam::Nil(meta) |
            TypeGleam::Media(_, meta) |
            TypeGleam::List(_, meta) |
            TypeGleam::Dict(_, _, meta) |
            TypeGleam::Dynamic { meta, .. } => {
                matches!(meta.type_wrapper, TypeWrapper::Optional(_))
            }
            TypeGleam::Class { meta, .. } |
            TypeGleam::Enum { meta, .. } |
            TypeGleam::TypeAlias { meta, .. } |
            TypeGleam::Union { meta, .. } => {
                matches!(meta.type_wrapper, TypeWrapper::Optional(_))
            }
        }
    }
    
    pub fn make_optional(mut self) -> TypeGleam {
        if self.is_optional() {
            return self;
        }
        
        let meta = self.meta_mut();
        meta.type_wrapper = TypeWrapper::Optional(Box::new(meta.type_wrapper.clone()));
        self
    }
    
    pub fn meta(&self) -> &TypeMetaGleam {
        match self {
            TypeGleam::String(_, meta) |
            TypeGleam::Int(_, meta) |
            TypeGleam::Float(meta) |
            TypeGleam::Bool(_, meta) |
            TypeGleam::Nil(meta) |
            TypeGleam::Media(_, meta) |
            TypeGleam::List(_, meta) |
            TypeGleam::Dict(_, _, meta) |
            TypeGleam::Dynamic { meta, .. } => meta,
            TypeGleam::Class { meta, .. } |
            TypeGleam::Enum { meta, .. } |
            TypeGleam::TypeAlias { meta, .. } |
            TypeGleam::Union { meta, .. } => meta,
        }
    }
    
    fn meta_mut(&mut self) -> &mut TypeMetaGleam {
        match self {
            TypeGleam::String(_, meta) |
            TypeGleam::Int(_, meta) |
            TypeGleam::Float(meta) |
            TypeGleam::Bool(_, meta) |
            TypeGleam::Nil(meta) |
            TypeGleam::Media(_, meta) |
            TypeGleam::List(_, meta) |
            TypeGleam::Dict(_, _, meta) |
            TypeGleam::Dynamic { meta, .. } => meta,
            TypeGleam::Class { meta, .. } |
            TypeGleam::Enum { meta, .. } |
            TypeGleam::TypeAlias { meta, .. } |
            TypeGleam::Union { meta, .. } => meta,
        }
    }
    
    pub fn decoder_name(&self) -> String {
        // Return the appropriate decoder name for json decoding
        match self {
            TypeGleam::String(_, _) => "dynamic.string".to_string(),
            TypeGleam::Int(_, _) => "dynamic.int".to_string(),
            TypeGleam::Float(_) => "dynamic.float".to_string(),
            TypeGleam::Bool(_, _) => "dynamic.bool".to_string(),
            TypeGleam::Nil(_) => "fn(_) { Ok(Nil) }".to_string(),
            TypeGleam::List(inner, _) => {
                format!("dynamic.list({})", inner.decoder_name())
            }
            TypeGleam::Dict(_, value, _) => {
                format!("dynamic.dict(dynamic.string, {})", value.decoder_name())
            }
            _ => "dynamic.dynamic".to_string(),
        }
    }
}

/// Trait for serializing types
pub trait SerializeType {
    fn serialize_type(&self, pkg: &CurrentRenderPackage) -> String;
}

impl SerializeType for TypeGleam {
    fn serialize_type(&self, pkg: &CurrentRenderPackage) -> String {
        let base_type = match self {
            TypeGleam::String(_, _) => "String".to_string(),
            TypeGleam::Int(_, _) => "Int".to_string(),
            TypeGleam::Float(_) => "Float".to_string(),
            TypeGleam::Bool(_, _) => "Bool".to_string(),
            TypeGleam::Nil(_) => "Nil".to_string(),
            
            TypeGleam::Media(media, _) => media.serialize_type(pkg),
            
            TypeGleam::Class { module, name, .. } => {
                if module == &pkg.current_module {
                    name.clone()
                } else {
                    format!("{}.{}", module, name)
                }
            }
            
            TypeGleam::Enum { module, name, .. } => {
                if module == &pkg.current_module {
                    name.clone()
                } else {
                    format!("{}.{}", module, name)
                }
            }
            
            TypeGleam::List(inner, _) => {
                format!("List({})", inner.serialize_type(pkg))
            }
            
            TypeGleam::Dict(key, value, _) => {
                format!("Dict({}, {})", key.serialize_type(pkg), value.serialize_type(pkg))
            }
            
            TypeGleam::TypeAlias { module, name, .. } => {
                if module == &pkg.current_module {
                    name.clone()
                } else {
                    format!("{}.{}", module, name)
                }
            }
            
            TypeGleam::Union { module, name, .. } => {
                if module == &pkg.current_module {
                    name.clone()
                } else {
                    format!("{}.{}", module, name)
                }
            }
            
            TypeGleam::Dynamic { .. } => "Dynamic".to_string(),
        };
        
        // Apply type wrappers
        let meta = self.meta();
        apply_type_wrapper(base_type, &meta.type_wrapper, meta.wrap_stream_state)
    }
}

fn apply_type_wrapper(base_type: String, wrapper: &TypeWrapper, wrap_stream_state: bool) -> String {
    match wrapper {
        TypeWrapper::None => {
            if wrap_stream_state {
                format!("StreamState({})", base_type)
            } else {
                base_type
            }
        }
        TypeWrapper::Optional(inner) => {
            let inner_type = apply_type_wrapper(base_type, inner, wrap_stream_state);
            format!("Option({})", inner_type)
        }
        TypeWrapper::Checked(inner) => {
            let inner_type = apply_type_wrapper(base_type, inner, wrap_stream_state);
            format!("Checked({})", inner_type)
        }
    }
}

impl MediaTypeGleam {
    pub fn serialize_type(&self, _pkg: &CurrentRenderPackage) -> String {
        match self {
            MediaTypeGleam::Image => "baml_types.Image".to_string(),
            MediaTypeGleam::Audio => "baml_types.Audio".to_string(),
            MediaTypeGleam::Pdf => "baml_types.Pdf".to_string(),
            MediaTypeGleam::Video => "baml_types.Video".to_string(),
        }
    }
}

impl From<&TypeValue> for TypeGleam {
    fn from(value: &TypeValue) -> Self {
        let meta = TypeMetaGleam::default();
        match value {
            TypeValue::String => TypeGleam::String(None, meta),
            TypeValue::Int => TypeGleam::Int(None, meta),
            TypeValue::Float => TypeGleam::Float(meta),
            TypeValue::Bool => TypeGleam::Bool(None, meta),
            TypeValue::Null => TypeGleam::Nil(meta),
            TypeValue::Media(media_type) => TypeGleam::Media(media_type.into(), meta),
        }
    }
}

impl From<&BamlMediaType> for MediaTypeGleam {
    fn from(media_type: &BamlMediaType) -> Self {
        match media_type {
            BamlMediaType::Image => MediaTypeGleam::Image,
            BamlMediaType::Audio => MediaTypeGleam::Audio,
            BamlMediaType::Pdf => MediaTypeGleam::Pdf,
            BamlMediaType::Video => MediaTypeGleam::Video,
        }
    }
}

impl fmt::Display for TypeGleam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For display purposes, we use a simplified format without package context
        let display_str = match self {
            TypeGleam::String(_, _) => "String",
            TypeGleam::Int(_, _) => "Int",
            TypeGleam::Float(_) => "Float",
            TypeGleam::Bool(_, _) => "Bool",
            TypeGleam::Nil(_) => "Nil",
            TypeGleam::Media(media, _) => match media {
                MediaTypeGleam::Image => "Image",
                MediaTypeGleam::Audio => "Audio",
                MediaTypeGleam::Pdf => "Pdf",
                MediaTypeGleam::Video => "Video",
            },
            TypeGleam::Class { name, .. } => name,
            TypeGleam::Enum { name, .. } => name,
            TypeGleam::List(inner, _) => return write!(f, "List({})", inner),
            TypeGleam::Dict(key, value, _) => return write!(f, "Dict({}, {})", key, value),
            TypeGleam::TypeAlias { name, .. } => name,
            TypeGleam::Union { name, .. } => name,
            TypeGleam::Dynamic { .. } => "Dynamic",
        };
        write!(f, "{}", display_str)
    }
}