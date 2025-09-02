use std::fmt;

use baml_types::{BamlMediaType, TypeValue};

use crate::package::CurrentRenderPackage;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeGleam {
    // Primitive types
    String(Option<String>),
    Int(Option<i64>),
    Float,
    Bool(Option<bool>),
    Nil,
    
    // Media types
    Media(MediaTypeGleam),
    
    // Custom types
    Class { 
        module: String, 
        name: String,
        dynamic: bool,
    },
    Enum { 
        module: String, 
        name: String,
        dynamic: bool,
    },
    
    // Container types
    List(Box<TypeGleam>),
    Option(Box<TypeGleam>),
    Result { 
        ok: Box<TypeGleam>, 
        error: Box<TypeGleam> 
    },
    Dict(Box<TypeGleam>, Box<TypeGleam>),
    
    // Type aliases
    TypeAlias {
        module: String,
        name: String,
    },
    
    // For union types
    Union {
        module: String,
        name: String,
    },
    
    // For any type we can't represent
    Dynamic {
        reason: String,
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
        matches!(self, TypeGleam::Option(_))
    }
    
    pub fn make_optional(self) -> TypeGleam {
        if self.is_optional() {
            self
        } else {
            TypeGleam::Option(Box::new(self))
        }
    }
    
    pub fn serialize_type(&self, pkg: &CurrentRenderPackage) -> String {
        match self {
            TypeGleam::String(_) => "String".to_string(),
            TypeGleam::Int(_) => "Int".to_string(),
            TypeGleam::Float => "Float".to_string(),
            TypeGleam::Bool(_) => "Bool".to_string(),
            TypeGleam::Nil => "Nil".to_string(),
            
            TypeGleam::Media(media) => media.serialize_type(pkg),
            
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
            
            TypeGleam::List(inner) => {
                format!("List({})", inner.serialize_type(pkg))
            }
            
            TypeGleam::Option(inner) => {
                format!("Option({})", inner.serialize_type(pkg))
            }
            
            TypeGleam::Result { ok, error } => {
                format!("Result({}, {})", ok.serialize_type(pkg), error.serialize_type(pkg))
            }
            
            TypeGleam::Dict(key, value) => {
                format!("Dict({}, {})", key.serialize_type(pkg), value.serialize_type(pkg))
            }
            
            TypeGleam::TypeAlias { module, name } => {
                if module == &pkg.current_module {
                    name.clone()
                } else {
                    format!("{}.{}", module, name)
                }
            }
            
            TypeGleam::Union { module, name } => {
                if module == &pkg.current_module {
                    name.clone()
                } else {
                    format!("{}.{}", module, name)
                }
            }
            
            TypeGleam::Dynamic { .. } => "Dynamic".to_string(),
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
        match value {
            TypeValue::String => TypeGleam::String(None),
            TypeValue::Int => TypeGleam::Int(None),
            TypeValue::Float => TypeGleam::Float,
            TypeValue::Bool => TypeGleam::Bool(None),
            TypeValue::Null => TypeGleam::Nil,
            TypeValue::Media(media_type) => TypeGleam::Media(media_type.into()),
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
            TypeGleam::String(_) => "String",
            TypeGleam::Int(_) => "Int",
            TypeGleam::Float => "Float",
            TypeGleam::Bool(_) => "Bool",
            TypeGleam::Nil => "Nil",
            TypeGleam::Media(media) => match media {
                MediaTypeGleam::Image => "Image",
                MediaTypeGleam::Audio => "Audio",
                MediaTypeGleam::Pdf => "Pdf",
                MediaTypeGleam::Video => "Video",
            },
            TypeGleam::Class { name, .. } => name,
            TypeGleam::Enum { name, .. } => name,
            TypeGleam::List(inner) => return write!(f, "List({})", inner),
            TypeGleam::Option(inner) => return write!(f, "Option({})", inner),
            TypeGleam::Result { ok, error } => return write!(f, "Result({}, {})", ok, error),
            TypeGleam::Dict(key, value) => return write!(f, "Dict({}, {})", key, value),
            TypeGleam::TypeAlias { name, .. } => name,
            TypeGleam::Union { name, .. } => name,
            TypeGleam::Dynamic { .. } => "Dynamic",
        };
        write!(f, "{}", display_str)
    }
}