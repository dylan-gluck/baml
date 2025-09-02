use baml_types::{
    baml_value::TypeLookups,
    ir_type::{TypeNonStreaming, TypeStreaming},
};

use crate::r#type::TypeGleam;

pub mod classes;
pub mod enums;
pub mod functions;
pub mod type_aliases;

pub fn type_to_gleam(field: &TypeNonStreaming, lookup: &impl TypeLookups) -> TypeGleam {
    use TypeNonStreaming as T;
    
    match field {
        T::Primitive(type_value, _) => type_value.into(),
        
        T::Enum { name, dynamic, .. } => TypeGleam::Enum {
            module: "baml_types".to_string(),
            name: name.clone(),
            dynamic: *dynamic,
        },
        
        T::Literal(literal_value, _) => match literal_value {
            baml_types::LiteralValue::String(val) => TypeGleam::String(Some(val.clone())),
            baml_types::LiteralValue::Int(val) => TypeGleam::Int(Some(*val)),
            baml_types::LiteralValue::Bool(val) => TypeGleam::Bool(Some(*val)),
        },
        
        T::Class { name, dynamic, .. } => TypeGleam::Class {
            module: "baml_types".to_string(),
            name: name.clone(),
            dynamic: *dynamic,
        },
        
        T::List(inner, _) => {
            TypeGleam::List(Box::new(type_to_gleam(inner, lookup)))
        }
        
        T::Map(key, value, _) => {
            TypeGleam::Dict(
                Box::new(type_to_gleam(key, lookup)),
                Box::new(type_to_gleam(value, lookup))
            )
        }
        
        T::RecursiveTypeAlias { name, .. } => {
            if lookup.expand_recursive_type(name).is_err() {
                TypeGleam::Dynamic {
                    reason: format!("Recursive type alias {} is not yet supported", name),
                }
            } else {
                TypeGleam::TypeAlias {
                    module: "baml_types".to_string(),
                    name: name.clone(),
                }
            }
        }
        
        T::Tuple(..) => TypeGleam::Dynamic {
            reason: "Tuples are not directly supported in Gleam, using Dynamic".to_string(),
        },
        
        T::Arrow(..) => TypeGleam::Dynamic {
            reason: "Arrow types are not supported in Gleam".to_string(),
        },
        
        T::Union(union_type, _) => match union_type.view() {
            baml_types::ir_type::UnionTypeViewGeneric::Null => TypeGleam::Nil,
            
            baml_types::ir_type::UnionTypeViewGeneric::Optional(inner) => {
                TypeGleam::Option(Box::new(type_to_gleam(inner, lookup)))
            }
            
            baml_types::ir_type::UnionTypeViewGeneric::OneOf(types) => {
                // For now, we'll generate a union type name
                let options: Vec<_> = types.into_iter()
                    .map(|t| type_to_gleam(t, lookup))
                    .collect();
                let num_options = options.len();
                TypeGleam::Union {
                    module: "baml_types".to_string(),
                    name: format!("Union{}", num_options),
                }
            }
            
            baml_types::ir_type::UnionTypeViewGeneric::OneOfOptional(types) => {
                // Optional union type
                let options: Vec<_> = types.into_iter()
                    .map(|t| type_to_gleam(t, lookup))
                    .collect();
                let num_options = options.len();
                TypeGleam::Option(Box::new(TypeGleam::Union {
                    module: "baml_types".to_string(),
                    name: format!("Union{}", num_options),
                }))
            }
        },
        
        T::Top(_) => panic!(
            "TypeGeneric::Top should have been resolved by the compiler before code generation. \
             This indicates a bug in the type resolution phase."
        ),
    }
}

pub fn stream_type_to_gleam(field: &TypeStreaming, _lookup: &impl TypeLookups) -> TypeGleam {
    use TypeStreaming as T;
    
    // For streaming types, we'll wrap them in a StreamState type
    // This is simplified for now - proper streaming support will come later
    match field {
        T::Primitive(type_value, _) => type_value.into(),
        
        T::Enum { name, dynamic, .. } => TypeGleam::Enum {
            module: "baml_stream_types".to_string(),
            name: name.clone(),
            dynamic: *dynamic,
        },
        
        T::Class { name, dynamic, .. } => TypeGleam::Class {
            module: "baml_stream_types".to_string(),
            name: name.clone(),
            dynamic: *dynamic,
        },
        
        _ => {
            // For now, convert to non-streaming type
            // This will need to be properly implemented for streaming support
            TypeGleam::Dynamic {
                reason: "Streaming types not fully implemented yet".to_string(),
            }
        }
    }
}