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
    let mut non_image_files = Vec::new();

    for file in &files {
        if !infer::get_from_path(file)?
            .wrap_err("Unable to infer file type")?
            .mime_type()
            .starts_with("image")
        {
            non_image_files.push(file);
            continue;
        }

        let relative_path =
            pathdiff::diff_paths(file, &args.input).wrap_err("Unable to diff paths")?;
        let output_path = args.output.join(relative_path);
        resize_or_copy_image(file, &output_path)?;
    }

    if !non_image_files.is_empty() {
        println!("The following files are not images and were not processed:");
        for file in non_image_files {
            println!("{}", file.display());
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

fn resize_or_copy_image(input: &Path, output: &Path) -> Result<()> {
    std::fs::create_dir_all(output.parent().wrap_err("Invalid output path")?)?;

    let dimensions = Dimensions::try_from(input)?;

    if dimensions.should_resize() {
        let resize_arg = if dimensions.is_landscape() {
            format!("x{MAX_HEIGHT}")
        } else {
            format!("{MAX_HEIGHT}")
        };

        Command::new("convert")
            .args([
                input.to_str().wrap_err("Invalid input path")?,
                "-resize",
                &resize_arg,
                "-quality",
                "90",
                output.to_str().wrap_err("Invalid output path")?,
            ])
            .output()?;
    } else {
        std::fs::copy(input, output)?;
    }

    Ok(())
}
