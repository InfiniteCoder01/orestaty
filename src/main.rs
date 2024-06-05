use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Project path, current directory by default
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Output path, ${path}/dist
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the page
    Build,
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or(std::env::current_dir().unwrap());
    let output = cli.output.unwrap_or(path.join("dist"));
    let mut generator = orestaty::OreStaty::new();

    orestaty::build(&mut generator, &path, &output);

    // * Check for errors and finish
    if generator.errors() > 0 {
        eprintln!(
            "Build completed with {} {}",
            generator.errors(),
            if generator.errors() == 1 {
                "error"
            } else {
                "errors"
            }
        );
        std::process::exit(1);
    }
}

