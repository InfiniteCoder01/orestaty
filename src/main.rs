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
    let dst = cli.output.unwrap_or(path.join("dist"));
    let mut generator = orestaty::OreStaty::new();
    generator.handlebars.set_strict_mode(true);
    generator.register_default_markdown_templates();

    if path.join("sass").exists() {
        generator.sass_options = generator.sass_options.load_path(path.join("sass"));
    }

    let command = cli.command.unwrap_or(Commands::Build);
    match command {
        Commands::Build => {
            generator.build(&path.join("src"), &dst);
            if path.join("static").exists() {
                generator
                    .unwrap_or_error(
                        orestaty::files::copy_recursively(path.join("static"), &dst),
                        "Failed to copy static files",
                    )
                    .ok();
            }
        }
    }

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
