use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Project path, current directory by default
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Output path, defaults to <path>/dist
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Subcommand)]
enum Commands {
    /// Init a new website in the current directory (override using -p/--path)
    Init,
    /// Build the website
    #[default]
    Build,
    /// Generate .css file from a colorscheme
    ThemeToCSS {
        /// Theme name or path to .tmTheme
        theme: String,
        /// Output path, defaults to <theme-name>.css
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
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
    if let Commands::ThemeToCSS { theme, output } = cli.command {
        if let Some(syntax_highlighting) =
            orestaty::plugins::syntax_highlighting::SyntaxHighlighting::new(&theme, "".as_ref())
        {
            if let Some(css) = syntax_highlighting.export_theme() {
                let path = output.unwrap_or_else(|| {
                    if theme.ends_with(".light") || theme.ends_with(".dark") {
                        std::path::PathBuf::from(format!("{theme}.css"))
                    } else {
                        std::path::Path::new(&theme).with_extension("css")
                    }
                });
                if let Err(err) = std::fs::write(path, css) {
                    eprintln!("Failed to write theme to a file: {}", err);
                }
            }
        }
        return;
    }

    let config = parse_config(&path.join("config.toml")).unwrap_or_default();
    let mut generator = orestaty::OreStaty::new(config, &path);

    generator.handlebars.set_strict_mode(true);
    generator.register_default_markdown_template();
    generator.register_builtin_plugins();

    let plugin_path = path.join("plugins");
    if plugin_path.exists() {
        generator.sass_options = generator.sass_options.load_path(&plugin_path);
        generator.load_plugins(&plugin_path, "").ok();
    }

    match cli.command {
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
        _ => unreachable!(),
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
