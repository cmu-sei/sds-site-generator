use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use url::Url;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    name: String,
    url: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    compatibility: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    repository: Option<String>,
    #[serde(default)]
    navigation: Option<Vec<Navigation>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Navigation {
    pub title: String,
    pub path: Option<String>,
    #[serde(default)]
    pub children: Vec<Navigation>,
}

impl Navigation {
    fn validate(&mut self) -> Result<()> {
        ensure!(
            !self.title.trim().is_empty(),
            "Navigation titles must not be empty."
        );
        self.title = self.title.trim().into();
        ensure!(
            self.path.is_some() || !self.children.is_empty(),
            "Navigation item {} needs a path or children.",
            self.title
        );
        if let Some(path) = &mut self.path {
            ensure!(
                !path.contains('\\') && !path.chars().any(char::is_control),
                "Navigation paths must not contain backslashes or control characters."
            );
            let local = path.starts_with('/') && !path.starts_with("//");
            let target = if local {
                Url::parse("https://navigation.invalid/")?.join(path)?
            } else {
                Url::parse(path)
                    .context("Navigation paths must be site-root paths or HTTP(S) URLs.")?
            };
            ensure!(
                matches!(target.scheme(), "http" | "https")
                    && target.host_str().is_some()
                    && target.username().is_empty()
                    && target.password().is_none()
                    && !path.starts_with("//"),
                "Navigation paths must be site-root paths or HTTP(S) URLs without credentials."
            );
        }
        for child in &mut self.children {
            child.validate()?;
        }
        Ok(())
    }
}

fn default_language() -> String {
    "en".into()
}

#[derive(Serialize)]
pub(crate) struct Config {
    pub name: String,
    pub url: Url,
    pub language: String,
    pub description: String,
    pub compatibility: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Url>,
    pub navigation: Option<Vec<Navigation>>,
}

impl Config {
    pub fn new(name: &str, url: &str) -> Result<Self> {
        Self::validate(FileConfig {
            name: name.into(),
            url: url.into(),
            language: default_language(),
            description: String::new(),
            compatibility: false,
            repository: None,
            navigation: None,
        })
    }

    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("site.toml");
        let metadata = path
            .symlink_metadata()
            .context("Missing site.toml. Start with `site-generator new my-site`.")?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "site.toml must not be a symbolic link."
        );
        let text = fs::read_to_string(&path)?;
        let file = toml::from_str(&text)
            .with_context(|| format!("{}: invalid configuration", path.display()))?;
        Self::validate(file).with_context(|| format!("{}: invalid configuration", path.display()))
    }

    fn validate(file: FileConfig) -> Result<Self> {
        ensure!(!file.name.trim().is_empty(), "name must not be empty.");
        let mut url = Url::parse(&file.url)
            .context("url must be an absolute publishing URL, e.g. https://example.org/docs/")?;
        ensure!(
            matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
            "url must start with https:// or http://."
        );
        ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "url must not contain credentials, a query, or a fragment."
        );
        ensure!(
            url.path().split('/').filter(|s| !s.is_empty()).all(|s| s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')),
            "Publishing URL subdirectories may contain only letters, numbers, hyphens, and underscores."
        );
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        ensure!(
            !file.language.is_empty()
                && file
                    .language
                    .split('-')
                    .all(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric())),
            "language must be a language tag such as en or en-US."
        );
        let repository = file
            .repository
            .map(|value| -> Result<Url> {
                let repository =
                    Url::parse(&value).context("repository must be an absolute HTTP(S) URL.")?;
                ensure!(
                    matches!(repository.scheme(), "http" | "https")
                        && repository.host_str().is_some()
                        && repository.username().is_empty()
                        && repository.password().is_none(),
                    "repository must be an absolute HTTP(S) URL without credentials."
                );
                Ok(repository)
            })
            .transpose()?;
        let mut navigation = file.navigation;
        if let Some(items) = &mut navigation {
            for item in items {
                item.validate()?;
            }
        }
        Ok(Self {
            name: file.name.trim().into(),
            url,
            language: file.language,
            description: file.description,
            compatibility: file.compatibility,
            repository,
            navigation,
        })
    }

    pub fn href(&self, route: &str) -> String {
        format!("{}{}", self.url.path(), route.trim_start_matches('/'))
    }

    pub fn canonical(&self, route: &str) -> String {
        format!("{}{}", self.url.as_str(), route.trim_start_matches('/'))
    }
}
