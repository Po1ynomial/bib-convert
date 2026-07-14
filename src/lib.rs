use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::io;

use biblatex::{Bibliography, ChunksExt, Entry, ParseError};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum FieldValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    pub r#type: String,
    pub key: String,
    pub fields: BTreeMap<String, FieldValue>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProjectionMode {
    Smart,
    Raw,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Yaml,
    Toml,
    Json,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Json => "json",
        }
    }
}

#[derive(Debug)]
pub enum ConvertError {
    Io(io::Error),
    Parse(ParseError),
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Toml(toml::ser::Error),
}

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Parse(error) => write!(f, "parse error: {error}"),
            Self::Json(error) => write!(f, "JSON serialization error: {error}"),
            Self::Yaml(error) => write!(f, "YAML serialization error: {error}"),
            Self::Toml(error) => write!(f, "TOML serialization error: {error}"),
        }
    }
}

impl std::error::Error for ConvertError {}

impl From<io::Error> for ConvertError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ParseError> for ConvertError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

impl From<serde_json::Error> for ConvertError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<serde_yaml::Error> for ConvertError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Yaml(error)
    }
}

impl From<toml::ser::Error> for ConvertError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Toml(error)
    }
}

pub fn parse_bibliography(input: &str) -> Result<Bibliography, ConvertError> {
    Ok(Bibliography::parse(input)?)
}

pub fn convert_str(input: &str) -> Result<Vec<Record>, ConvertError> {
    convert_str_with_mode(input, ProjectionMode::Smart)
}

pub fn convert_str_with_mode(
    input: &str,
    mode: ProjectionMode,
) -> Result<Vec<Record>, ConvertError> {
    let bibliography = parse_bibliography(input)?;
    Ok(project_bibliography(&bibliography, mode))
}

pub fn project_bibliography(bibliography: &Bibliography, mode: ProjectionMode) -> Vec<Record> {
    bibliography
        .iter()
        .map(|entry| Record {
            r#type: entry.entry_type.to_string(),
            key: entry.key.clone(),
            fields: entry
                .fields
                .iter()
                .map(|(name, chunks)| {
                    let raw = || FieldValue::String(chunks.format_verbatim());
                    let value = match mode {
                        ProjectionMode::Raw => raw(),
                        ProjectionMode::Smart => {
                            project_known_field(entry, name).unwrap_or_else(raw)
                        }
                    };
                    (name.clone(), value)
                })
                .collect(),
        })
        .collect()
}

fn project_known_field(entry: &Entry, name: &str) -> Option<FieldValue> {
    match name {
        "author" => entry.author().ok().map(render_people),
        "editor" => entry
            .editors()
            .ok()
            .map(|groups| groups.into_iter().flat_map(|(people, _)| people).collect())
            .map(render_people),
        "translator" => entry.translator().ok().map(render_people),
        "bookauthor" => entry.book_author().ok().map(render_people),
        "afterword" => entry.afterword().ok().map(render_people),
        "annotator" => entry.annotator().ok().map(render_people),
        "commentator" => entry.commentator().ok().map(render_people),
        "foreword" => entry.foreword().ok().map(render_people),
        "holder" => entry.holder().ok().map(render_people),
        "introduction" => entry.introduction().ok().map(render_people),
        "shortauthor" => entry.short_author().ok().map(render_people),
        "shorteditor" => entry.short_editor().ok().map(render_people),
        _ => None,
    }
}

fn render_people(people: Vec<biblatex::Person>) -> FieldValue {
    FieldValue::Strings(
        people
            .into_iter()
            .map(|person| person.to_string())
            .collect(),
    )
}

pub fn bibliography_entries(bibliography: &Bibliography) -> Vec<Entry> {
    bibliography.iter().cloned().collect()
}

pub fn serialize_records(records: &[Record], format: OutputFormat) -> Result<String, ConvertError> {
    serialize_value(records, format, Some("records"))
}

pub fn serialize_bibliography_debug(
    bibliography: &Bibliography,
    format: OutputFormat,
) -> Result<String, ConvertError> {
    serialize_value(bibliography, format, None)
}

pub fn serialize_entries_debug(
    entries: &[Entry],
    format: OutputFormat,
) -> Result<String, ConvertError> {
    serialize_value(entries, format, Some("entries"))
}

fn serialize_value<T: Serialize + ?Sized>(
    value: &T,
    format: OutputFormat,
    toml_top_level_key: Option<&str>,
) -> Result<String, ConvertError> {
    match format {
        OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
        OutputFormat::Toml => {
            let toml_value = toml::Value::try_from(value)?;
            let wrapped = match (toml_value, toml_top_level_key) {
                (toml::Value::Array(array), Some(key)) => {
                    let mut table = toml::map::Map::new();
                    table.insert(key.to_string(), toml::Value::Array(array));
                    toml::Value::Table(table)
                }
                (other, _) => other,
            };
            Ok(toml::to_string_pretty(&wrapped)?)
        }
        OutputFormat::Json => Ok(serde_json::to_string_pretty(value)?),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        convert_str, convert_str_with_mode, serialize_records, FieldValue, OutputFormat,
        ProjectionMode,
    };

    #[test]
    fn output_format_extensions_match_expected_names() {
        assert_eq!(OutputFormat::Yaml.extension(), "yaml");
        assert_eq!(OutputFormat::Toml.extension(), "toml");
        assert_eq!(OutputFormat::Json.extension(), "json");
    }

    #[test]
    fn convert_str_projects_normalized_records() {
        let input = r#"
@article{edge,
  author = {Gompf, Robert E. and Stipsicz, Andr\'as I.},
  title = {State sum invariants of $3$-manifolds},
  month = aug,
  note = {A \& B},
  year = {2024},
}
"#;

        let records = match convert_str(input) {
            Ok(records) => records,
            Err(error) => panic!("convert sample bibliography: {error}"),
        };

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].r#type, "article");
        assert_eq!(records[0].key, "edge");
        assert_eq!(
            records[0].fields["author"],
            FieldValue::Strings(vec![
                "Robert E. Gompf".to_string(),
                "András I. Stipsicz".to_string(),
            ])
        );
        assert_eq!(
            records[0].fields["title"],
            FieldValue::String("State sum invariants of $3$-manifolds".to_string())
        );
        assert_eq!(
            records[0].fields["month"],
            FieldValue::String("August".to_string())
        );
        assert_eq!(
            records[0].fields["note"],
            FieldValue::String("A & B".to_string())
        );
    }

    #[test]
    fn projects_other_person_list_fields() {
        let input = r#"
@book{edge,
  editor = {Doe, Jane and Smith, John},
  translator = {Roe, Richard},
  bookauthor = {Public, Jane Q.},
  shortauthor = {Gompf, Robert E.},
  title = {Collected work},
  year = {2024},
}
"#;

        let records = match convert_str(input) {
            Ok(records) => records,
            Err(error) => panic!("convert sample bibliography with extra people fields: {error}"),
        };

        assert_eq!(
            records[0].fields["editor"],
            FieldValue::Strings(vec!["Jane Doe".to_string(), "John Smith".to_string()])
        );
        assert_eq!(
            records[0].fields["translator"],
            FieldValue::Strings(vec!["Richard Roe".to_string()])
        );
        assert_eq!(
            records[0].fields["bookauthor"],
            FieldValue::Strings(vec!["Jane Q. Public".to_string()])
        );
        assert_eq!(
            records[0].fields["shortauthor"],
            FieldValue::Strings(vec!["Robert E. Gompf".to_string()])
        );
    }

    #[test]
    fn raw_projection_keeps_author_as_string() {
        let input = "@article{edge, author = {Gompf, Robert E. and Stipsicz, Andr\\'as I.}}";

        let records = match convert_str_with_mode(input, ProjectionMode::Raw) {
            Ok(records) => records,
            Err(error) => panic!("convert sample bibliography in raw mode: {error}"),
        };

        assert_eq!(
            records[0].fields["author"],
            FieldValue::String("Gompf, Robert E. and Stipsicz, András I.".to_string())
        );
    }

    #[test]
    fn serialize_records_wraps_top_level_toml_array() {
        let input = "@book{key, title = {Title}, year = {2024}}";
        let records = match convert_str(input) {
            Ok(records) => records,
            Err(error) => panic!("convert simple bibliography: {error}"),
        };

        let toml = match serialize_records(&records, OutputFormat::Toml) {
            Ok(text) => text,
            Err(error) => panic!("serialize records to TOML: {error}"),
        };

        assert!(toml.contains("[[records]]"));
        assert!(toml.contains("key = \"key\""));
        assert!(toml.contains("type = \"book\""));
    }
}
