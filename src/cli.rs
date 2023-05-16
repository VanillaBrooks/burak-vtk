use clap::Parser;
use std::path::PathBuf;

/// Burak's csv to VTK file conversion
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// path to .csv file to convert
    #[arg(short, long)]
    pub(crate) csv_path: PathBuf,

    /// output file .vtr extension
    #[arg(short, long)]
    pub(crate) output: PathBuf,
}
