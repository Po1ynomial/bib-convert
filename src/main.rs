use std::fs;
use std::path::{Path, PathBuf};

use bib_convert::{
    bibliography_entries, parse_bibliography, project_bibliography, serialize_bibliography_debug,
    serialize_entries_debug, serialize_records, ConvertError, OutputFormat,
};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "bib-convert")]
#[command(about = "Convert .bib files into structured YAML, TOML, or JSON")]
struct Cli {
    input: PathBuf,

    #[arg(short = 'f', long = "format", value_enum, default_value_t = CliFormat::Yaml)]
    format: CliFormat,

    #[arg(short = 'o')]
    output: Option<PathBuf>,

    #[arg(long = "debug-biblatex")]
    debug_biblatex: Option<PathBuf>,

    #[arg(long = "debug-entries")]
    debug_entries: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum CliFormat {
    Yaml,
    Toml,
    Json,
}

impl From<CliFormat> for OutputFormat {
    fn from(value: CliFormat) -> Self {
        match value {
            CliFormat::Yaml => Self::Yaml,
            CliFormat::Toml => Self::Toml,
            CliFormat::Json => Self::Json,
        }
    }
}

fn main() -> Result<(), ConvertError> {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.input)?;
    let format: OutputFormat = cli.format.into();

    let bibliography = parse_bibliography(&input)?;
    let records = project_bibliography(&bibliography);
    let output = serialize_records(&records, format)?;

    let output_path = match cli.output {
        Some(path) => path,
        None => default_output_path(&cli.input, format)?,
    };
    write_text(&output_path, &output)?;

    if let Some(path) = cli.debug_biblatex {
        let debug = serialize_bibliography_debug(&bibliography, OutputFormat::Yaml)?;
        write_text(&path, &debug)?;
    }

    if let Some(path) = cli.debug_entries {
        let entries = bibliography_entries(&bibliography);
        let debug = serialize_entries_debug(&entries, OutputFormat::Yaml)?;
        write_text(&path, &debug)?;
    }

    Ok(())
}

fn default_output_path(input: &Path, format: OutputFormat) -> Result<PathBuf, ConvertError> {
    let absolute_input = fs::canonicalize(input)?;
    Ok(absolute_input.with_extension(format.extension()))
}

fn write_text(path: &Path, content: &str) -> Result<(), ConvertError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}
