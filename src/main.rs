use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use topoxide::{
    RenderOptions, TopoxideInput, read_input, render_document_svg, render_svg, validate_graph,
};

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
        #[arg(long)]
        profile: Option<String>,
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
    ExportJson {
        #[arg(long)]
        input: PathBuf,
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
            profile,
            width,
            height,
        } => {
            let input = read_input(&input)?;
            let svg = match input {
                TopoxideInput::Graph(spec) => render_svg(&spec, RenderOptions { width, height })?,
                TopoxideInput::Document(document) => render_document_svg(
                    &document,
                    profile.as_deref(),
                    RenderOptions { width, height },
                )?,
            };
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, svg)?;
        }
        Command::Validate { input } => {
            match read_input(&input)? {
                TopoxideInput::Graph(spec) => validate_graph(&spec)?,
                TopoxideInput::Document(document) => validate_graph(&document.graph)?,
            }
            println!("OK");
        }
        Command::Schema { output } => {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, schema_document())?;
        }
        Command::ExportJson { input, output } => {
            let input = read_input(&input)?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = match input {
                TopoxideInput::Graph(spec) => serde_json::to_string_pretty(&spec)?,
                TopoxideInput::Document(document) => serde_json::to_string_pretty(&document)?,
            };
            fs::write(output, json)?;
        }
    }
    Ok(())
}

fn schema_document() -> &'static str {
    r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Topoxide GraphSpec",
  "type": "object",
  "required": ["title", "groups", "nodes"],
  "additionalProperties": false,
  "properties": {
    "title": {"type": "string"},
    "subtitle": {"type": "string"},
    "groups": {
      "type": "array",
      "items": {"$ref": "#/$defs/group"}
    },
    "nodes": {
      "type": "array",
      "items": {"$ref": "#/$defs/node"}
    },
    "edges": {
      "type": "array",
      "items": {"$ref": "#/$defs/edge"}
    },
    "theme": {"$ref": "#/$defs/theme"}
  },
  "$defs": {
    "group": {
      "type": "object",
      "required": ["id", "label"],
      "additionalProperties": false,
      "properties": {
        "id": {"type": "string", "minLength": 1},
        "label": {"type": "string"},
        "order": {"type": "integer", "minimum": 0},
        "description": {"type": "string"}
      }
    },
    "node": {
      "type": "object",
      "required": ["id", "label", "kind", "group"],
      "additionalProperties": false,
      "properties": {
        "id": {"type": "string", "minLength": 1},
        "label": {"type": "string"},
        "kind": {"type": "string"},
        "group": {"type": "string"},
        "detail": {"type": "string"},
        "metrics": {
          "type": "array",
          "items": {"$ref": "#/$defs/nodeMetric"}
        },
        "order": {"type": "integer", "minimum": 0},
        "weight": {"type": "number"}
      }
    },
    "nodeMetric": {
      "type": "object",
      "required": ["label", "value"],
      "additionalProperties": false,
      "properties": {
        "label": {"type": "string"},
        "value": {"type": "string"}
      }
    },
    "edge": {
      "type": "object",
      "required": ["source", "target"],
      "additionalProperties": false,
      "properties": {
        "source": {"type": "string", "minLength": 1},
        "target": {"type": "string", "minLength": 1},
        "kind": {"type": "string"},
        "label": {"type": "string"}
      }
    },
    "theme": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "background": {"type": "string"},
        "ink": {"type": "string"},
        "muted": {"type": "string"},
        "grid": {"type": "string"},
        "accent": {"type": "string"},
        "font-family": {"type": "string"}
      }
    }
  }
}
"##
}
