use std::{
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use eyre::{ContextCompat, Result};

const MAX_HEIGHT: u32 = 2160;

fn main() -> Result<()> {
    let args = Args::parse();

    let files = glob::glob(args.input.join("**/*").to_str().wrap_err("Invalid path")?)?
        .filter_map(Result::ok)
        .filter(|path| path.is_file())
        .collect::<Vec<PathBuf>>();

    for file in &files {
        if !infer::get_from_path(file)?
            .wrap_err("Unable to infer file type")?
            .mime_type()
            .starts_with("image")
        {
            println!("Skipping non-image file: {}", file.display());
            continue;
        }

        let dimensions = Dimensions::try_from(file.as_path())?;
        println!("{:?}", dimensions);

        let relative_path =
            pathdiff::diff_paths(file, &args.input).wrap_err("Unable to diff paths")?;
        let output_path = args.output.join(relative_path);
        println!("Output path: {}", output_path.display());
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

#[derive(Debug)]
struct Dimensions {
    width: u32,
    height: u32,
}

impl Dimensions {
    fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    fn should_resize(&self) -> bool {
        if self.is_landscape() {
            self.height > MAX_HEIGHT
        } else {
            self.width > MAX_HEIGHT
        }
    }
}

impl TryFrom<&Path> for Dimensions {
    type Error = eyre::Report;

    fn try_from(path: &Path) -> Result<Self> {
        let output = String::from_utf8(
            Command::new("identify")
                .args([
                    "-ping",
                    "-format",
                    "'%w %h'",
                    path.to_str().wrap_err("Invalid path")?,
                ])
                .output()?
                .stdout,
        )?;

        let (width, height) = output[1..output.len() - 1]
            .split_once(' ')
            .wrap_err("Unable to split identify output")?;

        Ok(Self {
            width: str::parse(width)?,
            height: str::parse(height)?,
        })
    }
}
