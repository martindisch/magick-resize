use std::path::PathBuf;

use clap::Parser;
use eyre::{ContextCompat, Result};

fn main() -> Result<()> {
    let args = Args::parse();

    let files = glob::glob(args.input.join("**/*").to_str().wrap_err("Invalid path")?)?
        .filter_map(Result::ok)
        .filter(|path| path.is_file())
        .collect::<Vec<PathBuf>>();

    for file in &files {
        let res = infer::get_from_path(file)?.wrap_err("Unable to infer file type")?;
        if !res.mime_type().starts_with("image") {
            println!("Skipping non-image file: {}", file.display());
        }
    }

    Ok(())
}

/// A dead simple tool using ImageMagick to resize images.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input directory.
    input: PathBuf,

    /// Path to the output directory.
    output: PathBuf,
}
