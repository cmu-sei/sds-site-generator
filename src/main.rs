use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Publish a documentation website from Markdown. No web tooling needed."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a ready-to-publish website in a NEW directory.
    New {
        directory: PathBuf,
        #[arg(long, default_value = "My documentation")]
        name: String,
        /// Your publishing URL, including any subdirectory.
        #[arg(long, default_value = "https://example.com/")]
        url: String,
    },
    /// Validate and generate the website into its dist directory.
    Build {
        #[arg(default_value = ".")]
        directory: PathBuf,
    },
    /// Check content, links, images and configuration without changing dist.
    Check {
        #[arg(default_value = ".")]
        directory: PathBuf,
        /// Fail on compatibility warnings as well as fatal errors.
        #[arg(long)]
        strict: bool,
    },
    /// Build, serve locally, and rebuild when source files change.
    Preview {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long, default_value_t = 4173)]
        port: u16,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::New {
            directory,
            name,
            url,
        } => {
            site_generator::create(&directory, &name, &url)?;
            println!(
                "Created {}.\nNext: sds-site-generator preview \"{}\"\nEdit content/index.md to get started.",
                directory.display(),
                directory.display()
            );
        }
        Command::Build { directory } => {
            let report = site_generator::build(&directory)?;
            print_warnings(&report.warnings);
            println!(
                "Built {} pages in {}",
                report.pages,
                report.output.display()
            );
            if report.placeholder_url {
                eprintln!(
                    "Before publishing: replace https://example.com/ in site.toml with your real URL."
                );
            }
        }
        Command::Check { directory, strict } => {
            let report = site_generator::check(&directory)
                .with_context(|| format!("Checking {}", directory.display()))?;
            print_warnings(&report.warnings);
            ensure!(
                !strict || report.warnings.is_empty(),
                "Strict validation failed: {} compatibility warnings. Fix the reported source issues.",
                report.warnings.len()
            );
            println!(
                "Checked {} pages with {} warnings.",
                report.pages,
                report.warnings.len()
            );
        }

        Command::Preview { directory, port } => site_generator::preview(&directory, port)?,
    }
    Ok(())
}

fn print_warnings(warnings: &[site_generator::Diagnostic]) {
    for warning in warnings.iter().take(20) {
        eprintln!(
            "Warning: {}:{}: {}",
            warning.file, warning.line, warning.message
        );
    }
    if warnings.len() > 20 {
        eprintln!(
            "{} additional warnings. Build to write the complete report to dist/_site/diagnostics.json.",
            warnings.len() - 20
        );
    }
}
