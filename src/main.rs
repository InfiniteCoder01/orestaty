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
    command: Commands,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Subcommand)]
enum Commands {
    /// Init a new website in the current directory (override using -p/--path)
    Init,
    /// Build the website
    #[default]
    Build,
}

fn parse_config(path: &std::path::Path) -> Result<orestaty::Config, ()> {
    if path.exists() {
        let config = std::fs::read_to_string(path).map_err(|err| {
            eprintln!("Failed to read config file: {}", err);
        })?;
        toml::from_str(&config).map_err(|err| {
            eprintln!("Failed to parse config file: {}", err);
        })
    } else {
        Err(())
    }
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or(std::env::current_dir().unwrap());
    let dst = cli.output.unwrap_or(path.join("dist"));

    if cli.command == Commands::Init {
        if let Err(err) = std::fs::create_dir_all(path.join("src")) {
            eprintln!("Failed to create src directory: {}", err);
        }
        if let Err(err) = std::fs::write(
            path.join("src").join("index.md"),
            include_str!("templates/index.md"),
        ) {
            eprintln!("Failed to create index.md: {}", err);
        }
        return;
    }

    let config = parse_config(&path.join("config.toml")).unwrap_or_default();
    let mut generator = orestaty::OreStaty::new(config);

    generator.handlebars.set_strict_mode(true);
    generator.register_default_markdown_templates();

    let plugin_path = path.join("plugins");
    if plugin_path.exists() {
        generator.sass_options = generator.sass_options.load_path(&plugin_path);
        generator.load_plugins(&plugin_path, "").ok();
    }

    match cli.command {
        Commands::Init => unreachable!(),
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
