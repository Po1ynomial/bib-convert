use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::io;

use biblatex::{Bibliography, ChunksExt, Entry, ParseError};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    pub r#type: String,
    pub key: String,
    pub fields: BTreeMap<String, String>,
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
    let bibliography = parse_bibliography(input)?;
    Ok(project_bibliography(&bibliography))
}

pub fn project_bibliography(bibliography: &Bibliography) -> Vec<Record> {
    bibliography
        .iter()
        .map(|entry| Record {
            r#type: entry.entry_type.to_string(),
            key: entry.key.clone(),
            fields: entry
                .fields
                .iter()
                .map(|(name, chunks)| (name.clone(), chunks.format_verbatim()))
                .collect(),
        })
        .collect()
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
    use super::{convert_str, serialize_records, OutputFormat};

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
            "Gompf, Robert E. and Stipsicz, András I."
        );
        assert_eq!(
            records[0].fields["title"],
            "State sum invariants of $3$-manifolds"
        );
        assert_eq!(records[0].fields["month"], "August");
        assert_eq!(records[0].fields["note"], "A & B");
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
