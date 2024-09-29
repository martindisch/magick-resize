use std::path::PathBuf;

use clap::Parser;

fn main() {
    let args = Args::parse();

    println!("{args:?}");
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
