mod converter;
mod ldtk_schema;
mod output_schema;
mod validator;
mod writer;

use std::path::Path;
use std::process;

use anyhow::{Context, Result};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse CLI args: --input <path> --output <path>
    let input = find_arg(&args, "--input")
        .context("Missing required argument: --input <path.ldtk>")?;
    let output = find_arg(&args, "--output")
        .context("Missing required argument: --output <path.json>")?;

    println!("ldtk_compiler v{}", env!("CARGO_PKG_VERSION"));
    println!("  Input:  {input}");
    println!("  Output: {output}");

    // Step 1: Read and parse
    let contents = std::fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {input}"))?;
    let root: ldtk_schema::LdtkRoot = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse LDtk JSON: {input}"))?;

    println!("  Parsed {} level(s)", root.levels.len());

    // Step 2: Validate
    if let Err(errors) = validator::validate(&root) {
        eprintln!("Validation failed with {} error(s):", errors.len());
        for err in &errors {
            eprintln!("  - {err}");
        }
        process::exit(1);
    }
    println!("  Validation passed");

    // Step 3: Convert
    let converted = converter::convert(&root);
    println!("  Converted {} level(s)", converted.len());

    // Step 4: Build output
    let output_root = output_schema::OutputRoot::from_converted(converted);

    // Step 5: Write atomically
    let output_path = Path::new(&output);
    writer::write_atomic(&output_root, output_path)
        .with_context(|| format!("Failed to write output: {output}"))?;

    println!("  Written to {output}");
    println!("Done.");

    Ok(())
}

fn find_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
