use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bib_convert::{convert_str, serialize_records, FieldValue, OutputFormat};

const SAMPLE_BIB: &str = r#"
@article{edge1,
  author = {Gompf, Robert E. and Stipsicz, Andr\'as I.},
  title = {State sum invariants of $3$-manifolds and quantum $6j$-symbols},
  month = aug,
  note = {A \& B},
  year = {2024},
}

@book{edge2,
  author = {Cencelj, M. and Repov\v{s}, D. and Skopenkov, A.},
  editor = {Doe, Jane and Smith, John},
  translator = {Roe, Richard},
  bookauthor = {Public, Jane Q.},
  title = {Codimension two PL embeddings of spheres with
           nonstandard regular neighborhoods},
  year = {2006},
}
"#;

#[test]
fn converts_characteristic_entries() {
    let records = match convert_str(SAMPLE_BIB) {
        Ok(records) => records,
        Err(error) => panic!("convert sample bib: {error}"),
    };

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].r#type, "article");
    assert_eq!(records[0].key, "edge1");
    assert_eq!(
        records[0].fields["author"],
        FieldValue::Strings(vec![
            "Robert E. Gompf".to_string(),
            "András I. Stipsicz".to_string(),
        ])
    );
    assert_eq!(
        records[0].fields["date"],
        FieldValue::String("2024-08".to_string())
    );
    assert!(!records[0].fields.contains_key("month"));
    assert!(!records[0].fields.contains_key("year"));
    assert_eq!(
        records[0].fields["note"],
        FieldValue::String("A & B".to_string())
    );
    assert_eq!(
        records[0].fields["title"],
        FieldValue::String(
            "State sum invariants of $3$-manifolds and quantum $6j$-symbols".to_string()
        )
    );
    assert_eq!(
        records[1].fields["author"],
        FieldValue::Strings(vec![
            "M. Cencelj".to_string(),
            "D. Repovš".to_string(),
            "A. Skopenkov".to_string(),
        ])
    );
    assert_eq!(
        records[1].fields["editor"],
        FieldValue::Strings(vec!["Jane Doe".to_string(), "John Smith".to_string()])
    );
    assert_eq!(
        records[1].fields["translator"],
        FieldValue::Strings(vec!["Richard Roe".to_string()])
    );
    assert_eq!(
        records[1].fields["bookauthor"],
        FieldValue::Strings(vec!["Jane Q. Public".to_string()])
    );
    assert_eq!(
        records[1].fields["date"],
        FieldValue::String("2006".to_string())
    );
    assert_eq!(
        records[1].fields["title"],
        FieldValue::String(
            "Codimension two PL embeddings of spheres with nonstandard regular neighborhoods"
                .to_string()
        )
    );
}

#[test]
fn serializes_supported_formats() {
    let records = match convert_str(SAMPLE_BIB) {
        Ok(records) => records,
        Err(error) => panic!("convert sample bib: {error}"),
    };

    let yaml = match serialize_records(&records, OutputFormat::Yaml) {
        Ok(text) => text,
        Err(error) => panic!("serialize yaml: {error}"),
    };
    assert!(yaml.contains("- type: article"));
    assert!(yaml.contains("date: 2024-08"));
    assert!(!yaml.contains("month: August"));
    assert!(yaml.contains("author:"));
    assert!(yaml.contains("- Robert E. Gompf"));
    assert!(yaml.contains("translator:"));
    assert!(yaml.contains("- Richard Roe"));

    let toml = match serialize_records(&records, OutputFormat::Toml) {
        Ok(text) => text,
        Err(error) => panic!("serialize toml: {error}"),
    };
    assert!(toml.contains("[[records]]"));
    assert!(toml.contains("type = \"article\""));
    assert!(toml.contains("author = [\n"));
    assert!(toml.contains("translator = [\"Richard Roe\"]"));

    let json = match serialize_records(&records, OutputFormat::Json) {
        Ok(text) => text,
        Err(error) => panic!("serialize json: {error}"),
    };
    assert!(json.starts_with("[\n  {"));
    assert!(json.contains("\"key\": \"edge1\""));
    assert!(json.contains("\"author\": ["));
    assert!(json.contains("\"translator\": ["));
}

#[test]
fn cli_writes_default_output_and_debug_files() {
    let temp_dir = unique_temp_dir();
    if let Err(error) = fs::create_dir_all(&temp_dir) {
        panic!("create temp dir: {error}");
    }

    let input_path = temp_dir.join("sample.bib");
    if let Err(error) = fs::write(&input_path, SAMPLE_BIB) {
        panic!("write sample bib: {error}");
    }

    let debug_biblatex = temp_dir.join("debug").join("biblatex.yaml");
    let debug_entries = temp_dir.join("debug").join("entries.yaml");

    let status = match Command::new(env!("CARGO_BIN_EXE_bib-convert"))
        .arg(&input_path)
        .args(["-f", "json"])
        .arg("--debug-biblatex")
        .arg(&debug_biblatex)
        .arg("--debug-entries")
        .arg(&debug_entries)
        .status()
    {
        Ok(status) => status,
        Err(error) => panic!("run cli: {error}"),
    };
    assert!(status.success());

    let output_path = default_output_path(&input_path, "json");
    let output = match fs::read_to_string(&output_path) {
        Ok(output) => output,
        Err(error) => panic!("read output file: {error}"),
    };
    assert!(output.contains("\"type\": \"article\""));
    assert!(output.contains("\"date\": \"2024-08\""));
    assert!(!output.contains("\"month\": \"August\""));
    assert!(output.contains("\"author\": ["));
    assert!(output.contains("\"Robert E. Gompf\""));

    let biblatex_debug = match fs::read_to_string(&debug_biblatex) {
        Ok(output) => output,
        Err(error) => panic!("read biblatex debug file: {error}"),
    };
    assert!(biblatex_debug.contains("entries:"));

    let entries_debug = match fs::read_to_string(&debug_entries) {
        Ok(output) => output,
        Err(error) => panic!("read entries debug file: {error}"),
    };
    assert!(entries_debug.contains("- key: edge1"));

    if let Err(error) = fs::remove_dir_all(&temp_dir) {
        panic!("remove temp dir: {error}");
    }
}

#[test]
fn cli_raw_fields_keeps_author_as_string() {
    let temp_dir = unique_temp_dir();
    if let Err(error) = fs::create_dir_all(&temp_dir) {
        panic!("create temp dir: {error}");
    }

    let input_path = temp_dir.join("sample.bib");
    if let Err(error) = fs::write(&input_path, SAMPLE_BIB) {
        panic!("write sample bib: {error}");
    }

    let output_path = temp_dir.join("raw.json");
    let status = match Command::new(env!("CARGO_BIN_EXE_bib-convert"))
        .arg(&input_path)
        .args(["-f", "json"])
        .arg("--raw-fields")
        .arg("-o")
        .arg(&output_path)
        .status()
    {
        Ok(status) => status,
        Err(error) => panic!("run cli in raw mode: {error}"),
    };
    assert!(status.success());

    let output = match fs::read_to_string(&output_path) {
        Ok(output) => output,
        Err(error) => panic!("read raw output file: {error}"),
    };
    assert!(output.contains("\"author\": \"Gompf, Robert E. and Stipsicz, András I.\""));
    assert!(output.contains("\"month\": \"August\""));
    assert!(output.contains("\"year\": \"2024\""));
    assert!(!output.contains("\"date\":"));

    if let Err(error) = fs::remove_dir_all(&temp_dir) {
        panic!("remove temp dir: {error}");
    }
}

fn unique_temp_dir() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(error) => panic!("compute unix timestamp: {error}"),
    };
    std::env::temp_dir().join(format!("bib-convert-test-{}-{nanos}", std::process::id()))
}

fn default_output_path(input: &Path, extension: &str) -> PathBuf {
    let absolute_input = match fs::canonicalize(input) {
        Ok(path) => path,
        Err(error) => panic!("canonicalize input: {error}"),
    };
    absolute_input.with_extension(extension)
}
