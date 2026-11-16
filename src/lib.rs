mod config;
mod content;
mod markup;
mod preview;
mod render;
mod search;

use anyhow::{Context, Result, bail, ensure};
use config::Config;
use content::Site;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub use content::Diagnostic;
pub use preview::preview;

pub struct CheckReport {
    pub pages: usize,
    pub warnings: Vec<Diagnostic>,
}

pub struct BuildReport {
    pub pages: usize,
    pub output: PathBuf,
    pub placeholder_url: bool,
    pub warnings: Vec<Diagnostic>,
}

pub fn create(directory: &Path, name: &str, url: &str) -> Result<()> {
    ensure!(
        !directory.exists(),
        "{} already exists; choose a NEW directory.",
        directory.display()
    );
    let config = Config::new(name, url)?;
    let parent = directory
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let temporary = tempfile::tempdir_in(parent)?;
    let staging = temporary.path().join("site");
    fs::create_dir(&staging)?;
    fs::create_dir(staging.join("content"))?;
    fs::create_dir(staging.join("content/guides"))?;
    fs::create_dir(staging.join("public"))?;
    fs::write(staging.join("site.toml"), toml::to_string_pretty(&config)?)?;
    fs::write(
        staging.join("content/index.md"),
        include_str!("../starter/index.md"),
    )?;
    fs::write(
        staging.join("content/guides/index.md"),
        include_str!("../starter/guides.md"),
    )?;
    fs::write(
        staging.join("content/guides/publishing.md"),
        include_str!("../starter/publishing.md"),
    )?;
    fs::write(staging.join(".gitignore"), "/dist/\n/.tmp*/\n")?;
    fs::write(
        staging.join("README.md"),
        include_str!("../starter/README.md"),
    )?;
    check(&staging)?;
    fs::rename(staging, directory)?;
    Ok(())
}

pub fn check(directory: &Path) -> Result<CheckReport> {
    let site = Site::load(directory)?;
    Ok(CheckReport {
        pages: site.pages.len(),
        warnings: site.diagnostics,
    })
}

pub fn build(directory: &Path) -> Result<BuildReport> {
    let root = directory
        .canonicalize()
        .with_context(|| format!("Open {}", directory.display()))?;
    let site = Site::load(&root)?;
    let output = root.join("dist");
    if output.symlink_metadata().is_ok() {
        ensure!(
            !output.symlink_metadata()?.file_type().is_symlink()
                && output.is_dir()
                && output.join(".site-generator-output").is_file(),
            "Refusing to replace {}: it is not an output directory created by this generator.",
            output.display()
        );
    }
    let temporary = tempfile::tempdir_in(&root)?;
    let staging = temporary.path().join("output");
    fs::create_dir(&staging)?;
    render::write(&site, &staging)?;
    search::write(&site, &staging)?;
    for (relative, asset) in &site.assets {
        let destination = staging.join(relative);
        ensure!(
            !destination.exists(),
            "Public asset {} conflicts with generated output.",
            relative.display()
        );
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(bytes) = &asset.bytes {
            fs::write(destination, bytes)?;
        } else {
            fs::copy(&asset.source, destination)?;
        }
    }
    fs::write(
        staging.join("_site/diagnostics.json"),
        serde_json::to_vec_pretty(&site.diagnostics)?,
    )?;
    fs::write(
        staging.join(".site-generator-output"),
        "site-generator v1\n",
    )?;
    for path in content::files(&staging)? {
        if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("html" | "css" | "js" | "json" | "xml" | "svg" | "txt")
        ) {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&fs::read(&path)?)?;
            let compressed = encoder.finish()?;
            let destination = path.with_file_name(format!(
                "{}.gz",
                path.file_name()
                    .context("Output file has no name.")?
                    .to_string_lossy()
            ));
            ensure!(
                !destination.exists(),
                "Precompressed output {} conflicts with an authored public file.",
                destination.display()
            );
            fs::write(destination, compressed)?;
        }
    }
    let previous = temporary.path().join("previous");
    if output.exists() {
        fs::rename(&output, &previous).context("Move previous build aside")?;
    }
    if let Err(error) = fs::rename(&staging, &output) {
        if previous.exists()
            && let Err(restore_error) = fs::rename(&previous, &output)
        {
            let recovery = temporary.keep().join("previous");
            bail!(
                "Publishing failed: {error}. Restoring the previous build also failed: {restore_error}. The previous build is preserved at {}.",
                recovery.display()
            );
        }
        bail!("Could not publish the build: {error}");
    }
    Ok(BuildReport {
        pages: site.pages.len(),
        output,
        placeholder_url: site.config.url.host_str() == Some("example.com"),
        warnings: site.diagnostics,
    })
}
