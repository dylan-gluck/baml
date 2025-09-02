use baml_types::{
    ir_type::{TypeNonStreaming, TypeStreaming},
    LiteralValue, TypeValue,
};

use crate::{
    package::CurrentRenderPackage,
    r#type::{TypeGleam, TypeMetaGleam, TypeWrapper},
};

pub mod classes;
pub mod enums;
pub mod functions;
pub mod type_aliases;
pub mod unions;

pub fn type_to_gleam(field: &TypeNonStreaming, pkg: &CurrentRenderPackage) -> TypeGleam {
    let meta = TypeMetaGleam {
        type_wrapper: TypeWrapper::None,
        wrap_stream_state: false,
    };
    
    type_to_gleam_inner(field, pkg, meta)
}

pub fn stream_type_to_gleam(field: &TypeStreaming, pkg: &CurrentRenderPackage) -> TypeGleam {
    let meta = TypeMetaGleam {
        type_wrapper: TypeWrapper::None,
        wrap_stream_state: true,
    };
    
    // Convert streaming to non-streaming for processing
    let non_streaming = field.to_ir_type().to_non_streaming_type(pkg.ir.as_ref());
    type_to_gleam_inner(&non_streaming, pkg, meta)
}

fn type_to_gleam_inner(field: &TypeNonStreaming, _pkg: &CurrentRenderPackage, meta: TypeMetaGleam) -> TypeGleam {
    use TypeNonStreaming as T;
    
    match field {
        T::Primitive(TypeValue::String, _) => TypeGleam::String(None, meta),
        T::Primitive(TypeValue::Int, _) => TypeGleam::Int(None, meta),
        T::Primitive(TypeValue::Float, _) => TypeGleam::Float(meta),
        T::Primitive(TypeValue::Bool, _) => TypeGleam::Bool(None, meta),
        T::Primitive(TypeValue::Null, _) => TypeGleam::Nil(meta),
        T::Primitive(TypeValue::Media(media), _) => TypeGleam::Media(media.into(), meta),
        
        T::Enum { name, dynamic, .. } => TypeGleam::Enum {
            module: "baml_types".to_string(),
            name: name.clone(),
            dynamic: *dynamic,
            meta,
        },
        
        T::Literal(LiteralValue::String(val), _) => TypeGleam::String(Some(val.clone()), meta),
        T::Literal(LiteralValue::Int(val), _) => TypeGleam::Int(Some(*val), meta),
        T::Literal(LiteralValue::Bool(val), _) => TypeGleam::Bool(Some(*val), meta),
        
        T::Class { name, dynamic, .. } => TypeGleam::Class {
            module: "baml_types".to_string(),
            name: name.clone(),
            dynamic: *dynamic,
            meta,
        },
        
        T::List(inner, _) => {
            let inner_meta = TypeMetaGleam {
                type_wrapper: TypeWrapper::None,
                wrap_stream_state: false,
            };
            TypeGleam::List(
                Box::new(type_to_gleam_inner(inner, _pkg, inner_meta)),
                meta
            )
        }
        
        T::Map(key, value, _) => {
            let key_meta = TypeMetaGleam {
                type_wrapper: TypeWrapper::None,
                wrap_stream_state: false,
            };
            let value_meta = TypeMetaGleam {
                type_wrapper: TypeWrapper::None,
                wrap_stream_state: false,
            };
            TypeGleam::Dict(
                Box::new(type_to_gleam_inner(key, _pkg, key_meta)),
                Box::new(type_to_gleam_inner(value, _pkg, value_meta)),
                meta
            )
        }
        
        T::Union(variants, _) => {
            // Generate a union type name based on the variants
            let union_name = format!("Union{}", variants.len());
            TypeGleam::Union {
                module: "baml_types".to_string(),
                name: union_name,
                meta,
            }
        }
        
        T::Tuple(types, _) => {
            // Gleam doesn't have built-in tuples for arbitrary sizes
            TypeGleam::Dynamic {
                reason: "Tuple types are not yet supported in Gleam".to_string(),
                meta,
            }
        }
        
        
        T::RecursiveTypeAlias { name, .. } => {
            TypeGleam::TypeAlias {
                module: "baml_types".to_string(),
                name: name.clone(),
                meta,
            }
        }
    }
}