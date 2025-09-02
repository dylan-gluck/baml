use askama::Template;

use crate::{
    package::CurrentRenderPackage,
    r#type::{SerializeType, TypeGleam},
};

pub struct FunctionGleam {
    pub(crate) documentation: Option<String>,
    pub(crate) name: String,
    pub(crate) args: Vec<(String, TypeGleam)>,
    pub(crate) return_type: TypeGleam,
    pub(crate) stream_return_type: TypeGleam,
}

fn render_function(
    function: &FunctionGleam,
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    let template = FunctionTemplate {
        r#fn: function,
        pkg,
    };

    template.render()
}

fn render_function_stream(
    function: &FunctionGleam,
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    let stream_template = FunctionStreamTemplate {
        r#fn: function,
        pkg,
    };

    stream_template.render()
}

fn render_function_parse(
    function: &FunctionGleam,
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    let parse_template = FunctionParseTemplate {
        r#fn: function,
        pkg,
    };
    parse_template.render()
}

fn render_function_parse_stream(
    function: &FunctionGleam,
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    let parse_stream_template = FunctionParseStreamTemplate {
        r#fn: function,
        pkg,
    };
    parse_stream_template.render()
}

/// We use doc comments to render the functions.
///
/// ```askama
/// import baml_gleam
/// import baml_types
/// import gleam/option.{type Option}
/// import gleam/result.{type Result}
/// 
/// {% for function in functions %}
/// {{ crate::functions::render_function(function, pkg)? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionsTemplate<'a> {
    functions: &'a [FunctionGleam],
    pkg: &'a CurrentRenderPackage,
}

pub fn render_functions(
    functions: &[FunctionGleam],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    FunctionsTemplate {
        functions,
        pkg,
    }
    .render()
}

/// A map of type names to their Gleam types.
///
/// ```askama
/// import gleam/dict.{type Dict}
/// import gleam/dynamic.{type Dynamic}
/// import baml_gleam
/// 
/// pub fn type_map() -> Dict(String, fn() -> Dynamic) {
///   dict.from_list([
/// {% for (name, type_name) in types %}
///     #("{{ name }}", fn() { dynamic.from({{ type_name }}) }),
/// {% endfor %}
///   ])
/// }
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct TypeMapTemplate<'a> {
    types: &'a [(String, String)],
}

pub fn render_type_map(types: &[(String, String)]) -> Result<String, askama::Error> {
    TypeMapTemplate { types }.render()
}

/// Functions stream template.
///
/// ```askama
/// import baml_gleam
/// import baml_types
/// import gleam/iterator.{type Iterator}
/// import gleam/option.{type Option}
/// import gleam/result.{type Result}
/// 
/// {% for function in functions %}
/// {{ crate::functions::render_function_stream(function, pkg)? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionsStreamTemplate<'a> {
    functions: &'a [FunctionGleam],
    pkg: &'a CurrentRenderPackage,
}

pub fn render_functions_stream(
    functions: &[FunctionGleam],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    FunctionsStreamTemplate { functions, pkg }.render()
}

/// Functions parse template.
///
/// ```askama
/// import baml_gleam
/// import baml_types
/// import gleam/json
/// import gleam/result.{type Result}
/// 
/// {% for function in functions %}
/// {{ crate::functions::render_function_parse(function, pkg)? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionsParseTemplate<'a> {
    functions: &'a [FunctionGleam],
    pkg: &'a CurrentRenderPackage,
}

pub fn render_functions_parse(
    functions: &[FunctionGleam],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    FunctionsParseTemplate { functions, pkg }.render()
}

/// Individual function template.
///
/// ```askama
/// {% if let Some(doc) = fn.documentation %}
/// /// {{ doc }}
/// {% endif %}
/// pub fn {{ fn.name }}(
/// {% for (arg_name, arg_type) in fn.args %}
///   {{ arg_name }}: {{ arg_type.serialize_type(pkg) }},
/// {% endfor %}
/// ) -> Result({{ fn.return_type.serialize_type(pkg) }}, String) {
///   baml_gleam.call_function(
///     "{{ fn.name }}",
///     [
/// {% for (arg_name, _) in fn.args %}
///       #("{{ arg_name }}", dynamic.from({{ arg_name }})),
/// {% endfor %}
///     ],
///   )
/// }
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionTemplate<'a> {
    r#fn: &'a FunctionGleam,
    pkg: &'a CurrentRenderPackage,
}

/// Individual function stream template.
///
/// ```askama
/// {% if let Some(doc) = fn.documentation %}
/// /// {{ doc }} (streaming version)
/// {% endif %}
/// pub fn {{ fn.name }}_stream(
/// {% for (arg_name, arg_type) in fn.args %}
///   {{ arg_name }}: {{ arg_type.serialize_type(pkg) }},
/// {% endfor %}
/// ) -> Iterator(Result({{ fn.stream_return_type.serialize_type(pkg) }}, String)) {
///   baml_gleam.stream_function(
///     "{{ fn.name }}",
///     [
/// {% for (arg_name, _) in fn.args %}
///       #("{{ arg_name }}", dynamic.from({{ arg_name }})),
/// {% endfor %}
///     ],
///   )
/// }
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionStreamTemplate<'a> {
    r#fn: &'a FunctionGleam,
    pkg: &'a CurrentRenderPackage,
}

/// Individual function parse template.
///
/// ```askama
/// pub fn parse_{{ fn.name }}(json_str: String) -> Result({{ fn.return_type.serialize_type(pkg) }}, String) {
///   json.decode(json_str, {{ fn.return_type.decoder_name() }})
///   |> result.map_error(fn(e) { string.inspect(e) })
/// }
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionParseTemplate<'a> {
    r#fn: &'a FunctionGleam,
    pkg: &'a CurrentRenderPackage,
}

/// Individual function parse stream template.
///
/// ```askama
/// pub fn parse_{{ fn.name }}_stream(json_str: String) -> Result({{ fn.stream_return_type.serialize_type(pkg) }}, String) {
///   json.decode(json_str, {{ fn.stream_return_type.decoder_name() }})
///   |> result.map_error(fn(e) { string.inspect(e) })
/// }
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, ext = "txt", escape = "none")]
struct FunctionParseStreamTemplate<'a> {
    r#fn: &'a FunctionGleam,
    pkg: &'a CurrentRenderPackage,
}