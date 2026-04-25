use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use topoxide::{GraphSpec, RenderOptions, render_svg, validate_graph};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Render {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1800.0)]
        width: f32,
        #[arg(long, default_value_t = 1200.0)]
        height: f32,
    },
    Validate {
        #[arg(long)]
        input: PathBuf,
    },
    Schema {
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Render {
            input,
            output,
            width,
            height,
        } => {
            let spec = read_spec(&input)?;
            let svg = render_svg(&spec, RenderOptions { width, height })?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, svg)?;
        }
        Command::Validate { input } => {
            let spec = read_spec(&input)?;
            validate_graph(&spec)?;
            println!("OK");
        }
        Command::Schema { output } => {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, schema_document())?;
        }
    }
    Ok(())
}

fn read_spec(path: &PathBuf) -> Result<GraphSpec, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    match ext {
        "json" => Ok(serde_json::from_str(&text)?),
        "toml" => Ok(toml::from_str(&text)?),
        other => Err(format!("unsupported input extension: {other}").into()),
    }
}

fn schema_document() -> &'static str {
    r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Topoxide GraphSpec",
  "type": "object",
  "required": ["title", "groups", "nodes"],
  "properties": {
    "title": {"type": "string"},
    "subtitle": {"type": "string"},
    "groups": {"type": "array"},
    "nodes": {"type": "array"},
    "edges": {"type": "array"},
    "theme": {"type": "object"}
  }
}
"#
}
