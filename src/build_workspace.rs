use crate::Diagnostic;
use anyhow::{Context, Result, bail, ensure};
use std::fs::{self, File, Metadata, OpenOptions, TryLockError};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

const PREFIX: &str = ".site-generator-build-";
const MARKER: &str = ".site-generator-workspace";
const OWNERSHIP: &str = "site-generator workspace v1\n";
const OUTPUT_MARKER: &str = ".site-generator-output";
const OUTPUT_OWNERSHIP: &str = "site-generator v1\n";

pub(crate) struct BuildWorkspace {
    root: PathBuf,
    directory: PathBuf,
    staging: PathBuf,
    warnings: Vec<Diagnostic>,
    deferred_cleanup: Vec<PathBuf>,
    preserve_recovery: bool,
    _lock: File,
}

impl BuildWorkspace {
    pub(crate) fn new(root: &Path) -> Result<Self> {
        let lock = acquire_lock(root)?;
        let (warnings, deferred_cleanup) = clean_abandoned(root)?;
        let temporary = tempfile::Builder::new()
            .prefix(PREFIX)
            .tempdir_in(root)
            .with_context(|| format!("Create build workspace in {}", root.display()))?;
        let directory = temporary.keep();
        let workspace = Self {
            root: root.to_owned(),
            staging: directory.join("output"),
            directory,
            warnings,
            deferred_cleanup,
            preserve_recovery: false,
            _lock: lock,
        };
        let preparation = (|| {
            fs::write(workspace.directory.join(MARKER), OWNERSHIP)
                .context("Mark generator-owned build workspace")?;
            fs::create_dir(&workspace.staging).context("Create staged output")?;
            validate_destination(&workspace.root.join("dist"))
        })();
        match preparation {
            Ok(()) => Ok(workspace),
            Err(error) => workspace.finish(Err(error)),
        }
    }

    pub(crate) fn staging(&self) -> &Path {
        &self.staging
    }

    pub(crate) fn warnings(&self) -> &[Diagnostic] {
        &self.warnings
    }

    pub(crate) fn publish(&mut self) -> Result<()> {
        self.publish_using(|from, to| fs::rename(from, to))
    }

    fn publish_using(
        &mut self,
        mut rename: impl FnMut(&Path, &Path) -> std::io::Result<()>,
    ) -> Result<()> {
        let output = self.root.join("dist");
        let previous = self.directory.join("previous");
        validate_destination(&output)?;
        fs::write(self.staging.join(OUTPUT_MARKER), OUTPUT_OWNERSHIP)
            .context("Mark completed generated output")?;
        if metadata(&output)?.is_some() {
            rename(&output, &previous).context("Move previous build aside")?;
            self.preserve_recovery = true;
        }
        if let Err(error) = rename(&self.staging, &output) {
            if self.preserve_recovery {
                rename(&previous, &output).with_context(|| {
                    format!("Publishing failed: {error}. Restoring the previous build also failed")
                })?;
                self.preserve_recovery = false;
            }
            bail!("Could not publish the build: {error}");
        }
        ensure!(
            recognizable_output(&output)?,
            "Published output at {} is incomplete.",
            output.display()
        );
        self.preserve_recovery = false;
        Ok(())
    }

    pub(crate) fn finish<T>(self, result: Result<T>) -> Result<T> {
        let result = result.and_then(|value| {
            for directory in &self.deferred_cleanup {
                remove_workspace(directory).context(
                    "The build was published, but recovery cleanup failed. A later build will retry.",
                )?;
            }
            Ok(value)
        });
        let result = result.map_err(|error| {
            let mut messages: Vec<_> = self
                .warnings
                .iter()
                .map(|warning| warning.message.clone())
                .collect();
            messages.extend(self.deferred_cleanup.iter().map(|directory| {
                format!(
                    "Recovery backup location: {}",
                    directory.join("previous").display()
                )
            }));
            if messages.is_empty() {
                error
            } else {
                error.context(messages.join("\n"))
            }
        });
        if self.preserve_recovery {
            return result.with_context(|| {
                format!(
                    "The previous build is preserved at {}.",
                    self.directory.join("previous").display()
                )
            });
        }
        match (result, remove_workspace(&self.directory)) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error.context(
                "The build was published, but temporary cleanup failed. A later build will retry.",
            )),
            (Err(error), Err(cleanup)) => {
                Err(error.context(format!("Temporary cleanup also failed: {cleanup:#}")))
            }
        }
    }
}

fn acquire_lock(root: &Path) -> Result<File> {
    let path = root.join(".site-generator.lock");
    if let Some(metadata) = metadata(&path)? {
        ensure!(
            metadata.file_type().is_file(),
            "Build lock {} must be a regular file, not a directory or symbolic link.",
            path.display()
        );
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("Open build lock {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            eprintln!(
                "Waiting for another build of {} to finish...",
                root.display()
            );
            file.lock()
                .with_context(|| format!("Wait for build lock {}", path.display()))?;
        }
        Err(TryLockError::Error(error)) => {
            return Err(error).with_context(|| format!("Acquire build lock {}", path.display()));
        }
    }
    Ok(file)
}

fn clean_abandoned(root: &Path) -> Result<(Vec<Diagnostic>, Vec<PathBuf>)> {
    let published = recognizable_output(&root.join("dist"))?;
    let mut warnings = Vec::new();
    let mut deferred_cleanup = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("Inspect {}", root.display()))? {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().starts_with(PREFIX) || !entry.file_type()?.is_dir()
        {
            continue;
        }
        let directory = entry.path();
        let marker = directory.join(MARKER);
        if !regular_file(&marker)? || fs::read(&marker)? != OWNERSHIP.as_bytes() {
            continue;
        }
        let previous = directory.join("previous");
        if metadata(&previous)?.is_some() {
            if published && recognizable_output(&previous)? {
                // Existing output may have lost pages; only a fresh publication permits deletion.
                deferred_cleanup.push(directory);
                continue;
            }
            warnings.push(Diagnostic {
                file: previous.display().to_string(),
                line: 0,
                message: format!(
                    "Previous-build files are preserved at {} because published output is missing/incomplete or the backup is unrecognized. Inspect them before removing them.",
                    previous.display()
                ),
            });
            continue;
        }
        remove_workspace(&directory)?;
    }
    Ok((warnings, deferred_cleanup))
}

fn validate_destination(output: &Path) -> Result<()> {
    if metadata(output)?.is_some() {
        ensure!(
            owned_output(output)?,
            "Refusing to replace {}: it is not an output directory created by this generator.",
            output.display()
        );
    }
    Ok(())
}

fn owned_output(path: &Path) -> Result<bool> {
    Ok(directory(path)?
        && regular_file(&path.join(OUTPUT_MARKER))?
        && fs::read(path.join(OUTPUT_MARKER))? == OUTPUT_OWNERSHIP.as_bytes())
}

fn recognizable_output(path: &Path) -> Result<bool> {
    if !owned_output(path)? {
        return Ok(false);
    }
    for relative in ["search", "_site", "_site/search"] {
        if !directory(&path.join(relative))? {
            return Ok(false);
        }
    }
    for relative in [
        "404.html",
        "search/index.html",
        "_site/diagnostics.json",
        "_site/search/manifest.json",
    ] {
        if !regular_file(&path.join(relative))? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn remove_workspace(path: &Path) -> Result<()> {
    let cleanup = match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let restoration: Result<()> = (|| {
                let marker = path.join(MARKER);
                if directory(path)? && metadata(&marker)?.is_none() {
                    // Keep proof of ownership after partially completed cleanup.
                    OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(marker)?
                        .write_all(OWNERSHIP.as_bytes())?;
                }
                Ok(())
            })();
            match restoration {
                Ok(()) => Err(error.into()),
                Err(restoration) => Err(anyhow::Error::from(error).context(format!(
                    "Restoring the workspace ownership marker also failed: {restoration:#}"
                ))),
            }
        }
    };
    cleanup.with_context(|| format!("Clean generator workspace {}", path.display()))
}

fn metadata(path: &Path) -> Result<Option<Metadata>> {
    match path.symlink_metadata() {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Inspect {}", path.display())),
    }
}

fn regular_file(path: &Path) -> Result<bool> {
    Ok(metadata(path)?.is_some_and(|metadata| metadata.file_type().is_file()))
}

fn directory(path: &Path) -> Result<bool> {
    Ok(metadata(path)?.is_some_and(|metadata| metadata.file_type().is_dir()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc;
    use std::time::Duration;

    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("site");
        crate::create(&root, "Recovery test", "https://example.org/").unwrap();
        crate::build(&root).unwrap();
        (temporary, root)
    }

    #[test]
    fn publication_failures_restore_output_or_preserve_a_recoverable_backup() {
        for (failing_rename, fail_restoration) in [(1, false), (2, false), (2, true)] {
            let (_temporary, root) = fixture();
            let before = fs::read(root.join("dist/index.html")).unwrap();
            let abandoned = root.join(format!("{PREFIX}older-backup"));
            fs::create_dir(&abandoned).unwrap();
            fs::write(abandoned.join(MARKER), OWNERSHIP).unwrap();
            fs::rename(root.join("dist"), abandoned.join("previous")).unwrap();
            crate::build(&root).unwrap();
            let mut workspace = BuildWorkspace::new(&root).unwrap();
            let directory = workspace.directory.clone();
            let mut renames = 0;
            let result = workspace.publish_using(|from, to| {
                renames += 1;
                if renames == failing_rename || (renames == 3 && fail_restoration) {
                    return Err(std::io::Error::other("injected rename failure"));
                }
                fs::rename(from, to)
            });
            let error = format!("{:#}", workspace.finish(result).unwrap_err());
            assert!(error.contains("injected rename failure"), "{error}");
            assert_eq!(
                fs::read(abandoned.join("previous/index.html")).unwrap(),
                before
            );
            if fail_restoration {
                let previous = directory.join("previous");
                assert!(error.contains(&previous.display().to_string()), "{error}");
                assert_eq!(fs::read(previous.join("index.html")).unwrap(), before);
                assert!(!root.join("dist").exists());
                crate::build(&root).unwrap();
                assert_eq!(fs::read(root.join("dist/index.html")).unwrap(), before);
                assert!(previous.exists());
                crate::build(&root).unwrap();
            } else {
                assert_eq!(fs::read(root.join("dist/index.html")).unwrap(), before);
            }
            assert!(!directory.exists());
        }
    }

    #[test]
    fn workspace_process() {
        let Some(root) = std::env::var_os("SDS_TEST_INTERRUPTED_WORKSPACE") else {
            return;
        };
        let root = PathBuf::from(root);
        let workspace = BuildWorkspace::new(&root).unwrap();
        fs::rename(root.join("dist"), workspace.directory.join("previous")).unwrap();
        println!("WORKSPACE_READY");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        drop(workspace);
    }

    struct Process(Child);

    impl Drop for Process {
        fn drop(&mut self) {
            if self.0.try_wait().unwrap().is_none() {
                self.0.kill().unwrap();
                self.0.wait().unwrap();
            }
        }
    }

    #[test]
    fn killed_build_releases_its_lock_and_preserves_interrupted_publication() {
        let (_temporary, root) = fixture();
        let before = fs::read(root.join("dist/index.html")).unwrap();
        let mut process = Process(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "build_workspace::tests::workspace_process",
                    "--nocapture",
                ])
                .env("SDS_TEST_INTERRUPTED_WORKSPACE", &root)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        let stdout = process.0.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if line.unwrap().contains("WORKSPACE_READY") {
                    sender.send(()).unwrap();
                    break;
                }
            }
        });
        receiver.recv_timeout(Duration::from_secs(30)).unwrap();
        reader.join().unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join(".site-generator.lock"))
            .unwrap();
        assert!(matches!(lock.try_lock(), Err(TryLockError::WouldBlock)));
        process.0.kill().unwrap();
        process.0.wait().unwrap();
        lock.try_lock()
            .expect("a killed build must release its lock");
        drop(lock);

        assert!(!root.join("dist").exists());
        crate::build(&root).unwrap();
        assert_eq!(fs::read(root.join("dist/index.html")).unwrap(), before);
        let workspaces: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(PREFIX)
            })
            .collect();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(
            fs::read(workspaces[0].join("previous/index.html")).unwrap(),
            before
        );
        crate::build(&root).unwrap();
        assert!(!workspaces[0].exists());
    }
}
