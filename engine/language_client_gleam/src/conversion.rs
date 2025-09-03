use anyhow::{anyhow, Result};
use baml_types::BamlValue;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

/// Convert a JSON value to a BAML value
pub fn json_to_baml_value(json: JsonValue) -> Result<BamlValue> {
    match json {
        JsonValue::Null => Ok(BamlValue::Null),
        JsonValue::Bool(b) => Ok(BamlValue::Bool(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(BamlValue::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(BamlValue::Float(f))
            } else {
                Err(anyhow!("Invalid number: {}", n))
            }
        }
        JsonValue::String(s) => Ok(BamlValue::String(s)),
        JsonValue::Array(arr) => {
            let values = arr
                .into_iter()
                .map(json_to_baml_value)
                .collect::<Result<Vec<_>>>()?;
            Ok(BamlValue::List(values))
        }
        JsonValue::Object(obj) => {
            let map = obj
                .into_iter()
                .map(|(k, v)| json_to_baml_value(v).map(|v| (k, v)))
                .collect::<Result<IndexMap<_, _>>>()?;
            Ok(BamlValue::Map(map))
        }
    }
}

/// Convert a BAML value to a JSON value
pub fn baml_value_to_json(value: BamlValue) -> JsonValue {
    match value {
        BamlValue::Null => JsonValue::Null,
        BamlValue::Bool(b) => JsonValue::Bool(b),
        BamlValue::Int(i) => JsonValue::Number(i.into()),
        BamlValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(f) {
                JsonValue::Number(n)
            } else {
                JsonValue::Null
            }
        }
        BamlValue::String(s) => JsonValue::String(s),
        BamlValue::List(list) => {
            JsonValue::Array(list.into_iter().map(baml_value_to_json).collect())
        }
        BamlValue::Map(map) => {
            let obj = map
                .into_iter()
                .map(|(k, v)| (k, baml_value_to_json(v)))
                .collect();
            JsonValue::Object(obj)
        }
        BamlValue::Enum(name, value) => {
            serde_json::json!({
                "__enum__": name,
                "value": value
            })
        }
        BamlValue::Class(name, fields) => {
            let mut obj = serde_json::Map::new();
            obj.insert("__class__".to_string(), JsonValue::String(name));
            for (k, v) in fields {
                obj.insert(k, baml_value_to_json(v));
            }
            JsonValue::Object(obj)
        }
        BamlValue::Media(media) => {
            use baml_types::{BamlMediaContent, BamlMediaType};
            
            let media_type_str = match media.media_type {
                BamlMediaType::Image => "image",
                BamlMediaType::Audio => "audio",
                BamlMediaType::Pdf => "pdf",
                BamlMediaType::Video => "video",
            };
            
            let content_data = match &media.content {
                BamlMediaContent::Url(url) => serde_json::json!({
                    "url": url.url,
                }),
                BamlMediaContent::Base64(base64) => serde_json::json!({
                    "base64": base64.base64,
                }),
                BamlMediaContent::File(file) => {
                    // Use relpath as the file reference since path() returns Result
                    serde_json::json!({
                        "file": file.relpath.display().to_string(),
                    })
                },
            };
            
            let mut result = serde_json::json!({
                "__media__": media_type_str,
            });
            
            if let Some(mime) = &media.mime_type {
                result["mime_type"] = serde_json::json!(mime);
            }
            
            // Merge content data
            if let serde_json::Value::Object(content_map) = content_data {
                if let serde_json::Value::Object(result_map) = &mut result {
                    for (k, v) in content_map {
                        result_map.insert(k, v);
                    }
                }
            }
            
            result
        }
    }
}

// NOTE: BamlValueWithProvenance is not part of the public API
// This function is commented out until the type is available
// /// Convert a BAML value with provenance to JSON
// pub fn baml_value_with_provenance_to_json(value: BamlValueWithProvenance) -> JsonValue {
//     serde_json::json!({
//         "value": baml_value_to_json(value.value),
//         "provenance": value.provenance,
//     })
// }

/// Parse JSON arguments into a BAML value map
pub fn parse_json_args(args: JsonValue) -> Result<BamlValue> {
    match args {
        JsonValue::Object(obj) => {
            let map = obj
                .into_iter()
                .map(|(k, v)| json_to_baml_value(v).map(|v| (k, v)))
                .collect::<Result<IndexMap<_, _>>>()?;
            Ok(BamlValue::Map(map))
        }
        _ => Err(anyhow!("Arguments must be a JSON object")),
    }
}

#[cfg(feature = "erlang")]
pub mod erlang_conversion {
    use super::*;
    use rustler::{Atom, Encoder, Env, Term};

    /// Convert an Erlang term to a BAML value
    pub fn term_to_baml_value<'a>(env: Env<'a>, term: Term<'a>) -> Result<BamlValue> {
        if term.is_atom() {
            // Convert atom to string for comparison
            let atom_str = term.atom_to_string().map_err(|e| anyhow!("Failed to convert atom to string: {:?}", e))?;
            
            match atom_str.as_str() {
                "nil" | "undefined" => Ok(BamlValue::Null),
                "true" => Ok(BamlValue::Bool(true)),
                "false" => Ok(BamlValue::Bool(false)),
                _ => Ok(BamlValue::String(atom_str))
            }
        } else if term.is_number() {
            if let Ok(i) = term.decode::<i64>() {
                Ok(BamlValue::Int(i))
            } else if let Ok(f) = term.decode::<f64>() {
                Ok(BamlValue::Float(f))
            } else {
                Err(anyhow!("Failed to decode number"))
            }
        } else if term.is_binary() {
            let s: String = term.decode().map_err(|e| anyhow!("Failed to decode string: {:?}", e))?;
            Ok(BamlValue::String(s))
        } else if term.is_list() {
            let list: Vec<Term> = term.decode().map_err(|e| anyhow!("Failed to decode list: {:?}", e))?;
            let values = list
                .into_iter()
                .map(|t| term_to_baml_value(env, t))
                .collect::<Result<Vec<_>>>()?;
            Ok(BamlValue::List(values))
        } else if term.is_map() {
            let map: rustler::types::map::MapIterator = term.decode()
                .map_err(|e| anyhow!("Failed to decode map: {:?}", e))?;
            
            let mut baml_map = IndexMap::new();
            for (key, value) in map {
                let key_str = if key.is_atom() {
                    key.atom_to_string().map_err(|e| anyhow!("Failed to decode map key: {:?}", e))?
                } else if key.is_binary() {
                    key.decode().map_err(|e| anyhow!("Failed to decode map key: {:?}", e))?
                } else {
                    return Err(anyhow!("Map keys must be atoms or strings"));
                };
                
                let value = term_to_baml_value(env, value)?;
                baml_map.insert(key_str, value);
            }
            Ok(BamlValue::Map(baml_map))
        } else if term.is_tuple() {
            // Handle special tuple formats for enums and classes
            let tuple: Vec<Term> = term.decode().map_err(|e| anyhow!("Failed to decode tuple: {:?}", e))?;
            
            if tuple.len() == 2 {
                if tuple[0].is_atom() {
                    let tag_str = tuple[0].atom_to_string().map_err(|e| anyhow!("Failed to convert tag atom: {:?}", e))?;
                    if tag_str == "baml_enum" {
                        let (name, value): (String, String) = tuple[1].decode()
                            .map_err(|e| anyhow!("Failed to decode enum: {:?}", e))?;
                        return Ok(BamlValue::Enum(name, value));
                    } else if tag_str == "baml_class" {
                        let (name, fields) = decode_class_fields(env, tuple[1])?;
                        return Ok(BamlValue::Class(name, fields));
                    }
                }
            }
            
            // Otherwise, treat as a list
            let values = tuple
                .into_iter()
                .map(|t| term_to_baml_value(env, t))
                .collect::<Result<Vec<_>>>()?;
            Ok(BamlValue::List(values))
        } else {
            Err(anyhow!("Unsupported Erlang term type"))
        }
    }

    /// Convert a BAML value to an Erlang term
    pub fn baml_value_to_term<'a>(env: Env<'a>, value: BamlValue) -> Term<'a> {
        match value {
            BamlValue::Null => Atom::from_str(env, "nil").unwrap().encode(env),
            BamlValue::Bool(b) => b.encode(env),
            BamlValue::Int(i) => i.encode(env),
            BamlValue::Float(f) => f.encode(env),
            BamlValue::String(s) => s.encode(env),
            BamlValue::List(list) => {
                let terms: Vec<Term> = list
                    .into_iter()
                    .map(|v| baml_value_to_term(env, v))
                    .collect();
                terms.encode(env)
            }
            BamlValue::Map(map) => {
                let mut erlang_map = rustler::types::map::map_new(env);
                for (k, v) in map {
                    let key = Atom::from_str(env, &k)
                        .map(|a| a.encode(env))
                        .unwrap_or_else(|_| k.encode(env));
                    let value = baml_value_to_term(env, v);
                    erlang_map = erlang_map.map_put(key, value).unwrap();
                }
                erlang_map.encode(env)
            }
            BamlValue::Enum(name, value) => {
                let tag = Atom::from_str(env, "baml_enum").unwrap();
                (tag, (name, value)).encode(env)
            }
            BamlValue::Class(name, fields) => {
                let tag = Atom::from_str(env, "baml_class").unwrap();
                let mut field_map = rustler::types::map::map_new(env);
                for (k, v) in fields {
                    let key = Atom::from_str(env, &k)
                        .map(|a| a.encode(env))
                        .unwrap_or_else(|_| k.encode(env));
                    let value = baml_value_to_term(env, v);
                    field_map = field_map.map_put(key, value).unwrap();
                }
                (tag, (name, field_map)).encode(env)
            }
            BamlValue::Media(media) => {
                use baml_types::{BamlMediaContent, BamlMediaType};
                
                let tag = Atom::from_str(env, "baml_media").unwrap();
                
                let media_type_atom = match media.media_type {
                    BamlMediaType::Image => Atom::from_str(env, "image").unwrap(),
                    BamlMediaType::Audio => Atom::from_str(env, "audio").unwrap(),
                    BamlMediaType::Pdf => Atom::from_str(env, "pdf").unwrap(),
                    BamlMediaType::Video => Atom::from_str(env, "video").unwrap(),
                };
                
                let content_data = match &media.content {
                    BamlMediaContent::Url(url) => {
                        let url_tag = Atom::from_str(env, "url").unwrap();
                        (url_tag, url.url.clone()).encode(env)
                    }
                    BamlMediaContent::Base64(base64) => {
                        let base64_tag = Atom::from_str(env, "base64").unwrap();
                        (base64_tag, base64.base64.clone()).encode(env)
                    }
                    BamlMediaContent::File(file) => {
                        let file_tag = Atom::from_str(env, "file").unwrap();
                        (file_tag, file.relpath.display().to_string()).encode(env)
                    }
                };
                
                (tag, (media_type_atom, media.mime_type.clone(), content_data)).encode(env)
            }
        }
    }

    fn decode_class_fields<'a>(
        env: Env<'a>,
        term: Term<'a>,
    ) -> Result<(String, IndexMap<String, BamlValue>)> {
        let (name, fields_term): (String, Term) = term.decode()
            .map_err(|e| anyhow!("Failed to decode class: {:?}", e))?;
        
        let fields_map: rustler::types::map::MapIterator = fields_term.decode()
            .map_err(|e| anyhow!("Failed to decode class fields: {:?}", e))?;
        
        let mut fields = IndexMap::new();
        for (key, value) in fields_map {
            let key_str = if key.is_atom() {
                key.atom_to_string().map_err(|e| anyhow!("Failed to decode field key: {:?}", e))?
            } else if key.is_binary() {
                key.decode().map_err(|e| anyhow!("Failed to decode field key: {:?}", e))?
            } else {
                return Err(anyhow!("Field keys must be atoms or strings"));
            };
            
            let value = term_to_baml_value(env, value)?;
            fields.insert(key_str, value);
        }
        
        Ok((name, fields))
    }
}