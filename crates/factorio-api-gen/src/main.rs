#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::exit)]

use std::env;
use std::path::PathBuf;
use std::process;

use factorio_api_gen::{
    generate_from_bundled_api, generate_from_json, write_generated_api, write_macro_event_lookup,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("factorio-api-gen: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(output_dir) = args.next() else {
        eprintln!(
            "usage: factorio-api-gen <output-dir> [--macro-map <macro-output-dir>] [--json <path>]"
        );
        process::exit(2);
    };

    let output_dir = PathBuf::from(output_dir);
    let mut macro_map_dir = None;
    let mut json_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--macro-map" => {
                macro_map_dir = Some(PathBuf::from(
                    args.next().ok_or("--macro-map requires a path argument")?,
                ));
            }
            "--json" => {
                json_path = Some(PathBuf::from(
                    args.next().ok_or("--json requires a path argument")?,
                ));
            }
            other => return Err(format!("unknown argument `{other}`").into()),
        }
    }

    let generated = if let Some(path) = json_path {
        let json = std::fs::read_to_string(path)?;
        generate_from_json(&json)?
    } else {
        generate_from_bundled_api()?
    };

    write_generated_api(&output_dir, &generated)?;

    if let Some(macro_dir) = macro_map_dir {
        write_macro_event_lookup(&macro_dir, &generated)?;
    }

    println!(
        "generated Factorio runtime API bindings for v{} (format v{})",
        generated.application_version, generated.api_version
    );

    Ok(())
}
