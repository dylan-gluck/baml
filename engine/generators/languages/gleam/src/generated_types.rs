use crate::{
    package::CurrentRenderPackage,
    r#type::{SerializeType, TypeGleam},
};

mod filters {
    // This filter converts snake_case to PascalCase for Gleam types
    pub fn type_name(s: &str, _: &dyn askama::Values) -> askama::Result<String> {
        let parts: Vec<&str> = s.split('_').collect();
        let pascal_case = parts
            .iter()
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();
        Ok(pascal_case)
    }

    // This filter converts to snake_case for Gleam field names
    pub fn field_name(s: &str, _: &dyn askama::Values) -> askama::Result<String> {
        // Convert camelCase or PascalCase to snake_case
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() {
                if !result.is_empty() {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

pub mod class {
    use super::*;
    use askama::Template;

    #[derive(askama::Template)]
    #[template(path = "class.gleam.j2", escape = "none", ext = "txt")]
    pub struct ClassGleam<'a> {
        pub name: String,
        pub docstring: Option<String>,
        pub fields: Vec<FieldGleam<'a>>,
        pub dynamic: bool,
        pub pkg: &'a CurrentRenderPackage,
    }

    /// A field in a .
    ///
    /// ```askama
    /// {% match docstring -%}
    /// {%- when Some with (doc) -%}
    /// /// {{ doc }}
    /// {%- when None -%}
    /// {%- endmatch %}
    /// {{ name|field_name }}: {{ type.serialize_type(pkg) }}
    /// ```
    #[derive(askama::Template, Clone)]
    #[template(in_doc = true, escape = "none", ext = "txt")]
    pub struct FieldGleam<'a> {
        pub docstring: Option<String>,
        pub name: String,
        pub r#type: TypeGleam,
        pub pkg: &'a CurrentRenderPackage,
    }

    impl std::fmt::Debug for FieldGleam<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "FieldGleam {{docstring: {:?}, name: {}, type: {:?}, pkg: <<CurrentRenderPackage>> }}",
                self.docstring, self.name, self.r#type
            )
        }
    }
}

pub mod enums {
    use super::*;
    use askama::Template;

    #[derive(askama::Template)]
    #[template(path = "enums.gleam.j2", escape = "none")]
    pub struct EnumGleam<'a> {
        pub name: String,
        pub docstring: Option<String>,
        pub values: Vec<(String, Option<String>)>,
        pub dynamic: bool,
        pub pkg: &'a CurrentRenderPackage,
    }
}

pub mod union {
    use super::*;
    use askama::Template;

    #[derive(askama::Template)]
    #[template(path = "unions.gleam.j2", escape = "none")]
    pub struct UnionGleam<'a> {
        pub name: String,
        pub cffi_name: String,
        pub docstring: Option<String>,
        pub variants: Vec<VariantGleam>,
        pub pkg: &'a CurrentRenderPackage,
    }

    #[derive(Clone)]
    pub struct VariantGleam {
        pub name: String,
        pub cffi_name: String,
        pub literal_repr: Option<String>,
        pub type_: TypeGleam,
    }
}

pub mod type_aliases {
    use super::*;
    use askama::Template;

    /// A type alias in Gleam.
    ///
    /// ```askama
    /// {% match docstring -%}
    /// {%- when Some with (doc) -%}
    /// /// {{ doc }}
    /// {%- when None -%}
    /// {%- endmatch %}
    /// pub type {{ name }} = {{ type_.serialize_type(pkg) }}
    /// ```
    #[derive(askama::Template)]
    #[template(in_doc = true, escape = "none", ext = "txt")]
    pub struct TypeAliasGleam<'a> {
        pub name: String,
        pub type_: TypeGleam,
        pub docstring: Option<String>,
        pub pkg: &'a CurrentRenderPackage,
    }
}

/// Render Gleam type utilities
///
/// ```askama
/// import gleam/dynamic.{type Dynamic}
/// import gleam/option.{type Option}
/// import gleam/result.{type Result}
/// import baml_gleam
///
/// pub type Checked(a) = Result(a, List(String))
///
/// pub type Image = baml_gleam.Image
/// pub type Audio = baml_gleam.Audio
/// pub type Video = baml_gleam.Video
/// pub type Pdf = baml_gleam.Pdf
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, escape = "none", ext = "txt")]
struct GleamTypesUtils {}

pub(crate) fn render_gleam_types_utils(
    _pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    use askama::Template;
    GleamTypesUtils {}.render()
}

/// Render classes for Gleam
///
/// ```askama
/// import gleam/dynamic.{type Dynamic}
/// import gleam/json.{type Json}
/// import gleam/option.{type Option}
/// import gleam/result.{type Result}
/// import baml_gleam
///
/// {% for class in classes %}
/// {{ class.render()? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, escape = "none", ext = "txt")]
pub struct ClassesTemplate<'a> {
    classes: &'a [class::ClassGleam<'a>],
    pkg: &'a CurrentRenderPackage,
}

pub fn render_classes(
    classes: &[class::ClassGleam],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    use askama::Template;
    ClassesTemplate { classes, pkg }.render()
}

/// Render enums for Gleam
///
/// ```askama
/// import gleam/dynamic.{type Dynamic}
/// import gleam/json.{type Json}
/// import gleam/option.{type Option}
/// import gleam/result.{type Result}
///
/// {% for enum_ in enums %}
/// {{ enum_.render()? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, escape = "none", ext = "txt")]
pub struct EnumsTemplate<'a> {
    enums: &'a [enums::EnumGleam<'a>],
    pkg: &'a CurrentRenderPackage,
}

pub fn render_enums(
    enums: &[enums::EnumGleam],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    use askama::Template;
    EnumsTemplate { enums, pkg }.render()
}

/// Render unions for Gleam
///
/// ```askama
/// import gleam/dynamic.{type Dynamic}
/// import gleam/json.{type Json}
/// import gleam/option.{type Option}
/// import gleam/result.{type Result}
///
/// {% for union_type in unions %}
/// {{ union_type.render()? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, escape = "none", ext = "txt")]
pub struct UnionsTemplate<'a> {
    unions: &'a [union::UnionGleam<'a>],
    pkg: &'a CurrentRenderPackage,
}

pub fn render_unions(
    unions: &[union::UnionGleam],
    pkg: &CurrentRenderPackage,
) -> Result<String, askama::Error> {
    use askama::Template;
    UnionsTemplate { unions, pkg }.render()
}

/// Render type aliases for Gleam
///
/// ```askama
/// {% for alias in aliases %}
/// {{ alias.render()? }}
/// {% endfor %}
/// ```
#[derive(askama::Template)]
#[template(in_doc = true, escape = "none", ext = "txt")]
pub struct TypeAliasesTemplate<'a> {
    aliases: &'a [type_aliases::TypeAliasGleam<'a>],
}

pub fn render_type_aliases(
    aliases: &[type_aliases::TypeAliasGleam],
) -> Result<String, askama::Error> {
    use askama::Template;
    TypeAliasesTemplate { aliases }.render()
}

// Re-export the types at the module level for easier access
pub use class::{ClassGleam, FieldGleam};
pub use enums::EnumGleam;
pub use type_aliases::TypeAliasGleam;
pub use union::{UnionGleam, VariantGleam};
