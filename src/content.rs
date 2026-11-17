use crate::{config::Config, markup};
use anyhow::{Context, Result, ensure};
use kuchiki::NodeRef;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub message: String,
}

pub(crate) struct Site {
    pub config: Config,
    pub pages: Vec<Page>,
    pub assets: BTreeMap<PathBuf, Asset>,
    pub diagnostics: Vec<Diagnostic>,
    routes: BTreeMap<String, usize>,
    sources: BTreeMap<PathBuf, usize>,
}

pub(crate) struct Asset {
    pub source: PathBuf,
    pub bytes: Option<Vec<u8>>,
    pub dimensions: Option<(usize, usize)>,
}

pub(crate) struct Page {
    pub source: PathBuf,
    pub route: String,
    pub title: String,
    pub navigation_title: String,
    pub in_navigation: bool,
    pub description: String,
    pub order: i32,
    pub headings: Vec<Heading>,
    pub sections: Vec<Section>,
    pub html: String,
    document: NodeRef,
    anchors: BTreeSet<String>,
    original: String,
}

pub(crate) struct Heading {
    pub level: usize,
    pub id: String,
    pub text: String,
}

pub(crate) struct Section {
    pub heading: String,
    pub anchor: String,
    pub text: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Metadata {
    title: Option<String>,
    description: Option<String>,
    order: i32,
    draft: bool,
    navigation: Option<Navigation>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Navigation {
    Visible(bool),
    Options { title: Option<String> },
}

pub(crate) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn tokens(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn slug(value: &str) -> String {
    tokens(value).join("-")
}

fn without_order(value: &str) -> &str {
    value
        .split_once('.')
        .filter(|(prefix, _)| !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()))
        .map_or(value, |(_, rest)| rest)
}

fn route(source: &Path, compatibility: bool) -> Result<String> {
    let mut segments = Vec::new();
    let count = source.components().count();
    for (index, part) in source.iter().enumerate() {
        let text = part
            .to_str()
            .context("Content filenames must be valid UTF-8.")?;
        let stem = if index == count - 1 {
            text.strip_suffix(".md")
                .context("Content files must end in .md.")?
        } else {
            text
        };
        let clean = without_order(stem);
        if index == count - 1 && clean == "index" {
            continue;
        }
        let segment = if compatibility {
            clean
                .split(|character: char| !character.is_alphanumeric() && character != '_')
                .filter(|part| !part.is_empty())
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
                .join("-")
        } else {
            slug(clean)
        };
        ensure!(
            !segment.is_empty(),
            "{}: filename needs a letter or number.",
            source.display()
        );
        segments.push(segment);
    }
    Ok(if segments.is_empty() {
        "/".into()
    } else {
        format!("/{}/", segments.join("/"))
    })
}

pub(crate) fn files(root: &Path) -> Result<Vec<PathBuf>> {
    fn walk(root: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
        ensure!(
            !root.symlink_metadata()?.file_type().is_symlink(),
            "{}: symbolic links are not allowed.",
            root.display()
        );
        for entry in fs::read_dir(root).with_context(|| format!("Read {}", root.display()))? {
            let entry = entry?;
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "{}: symbolic links are not allowed.",
                entry.path().display()
            );
            if kind.is_dir() {
                walk(&entry.path(), result)?;
            } else {
                ensure!(
                    kind.is_file(),
                    "{} is not a regular file.",
                    entry.path().display()
                );
                result.push(entry.path());
            }
        }
        Ok(())
    }
    let mut result = Vec::new();
    walk(root, &mut result)?;
    result.sort();
    Ok(result)
}

fn issue(
    config: &Config,
    diagnostics: &mut Vec<Diagnostic>,
    file: &str,
    line: usize,
    message: impl Into<String>,
) -> Result<()> {
    let message = message.into();
    ensure!(config.compatibility, "{file}:{line}: {message}");
    diagnostics.push(Diagnostic {
        file: file.into(),
        line,
        message,
    });
    Ok(())
}

fn reserved_id(id: &str) -> bool {
    matches!(
        id,
        "site-main"
            | "site-sidebar"
            | "site-search"
            | "site-query"
            | "site-results"
            | "site-status"
            | "site-search-dialog"
            | "site-search-title"
            | "site-dialog-search"
            | "site-dialog-query"
            | "site-dialog-status"
            | "site-dialog-results"
    )
}

fn rename_heading(node: &NodeRef, level: usize) -> Result<NodeRef> {
    let document = markup::parse(&format!("<h{level}></h{level}>"));
    let replacement = document
        .select_first(&format!("h{level}"))
        .map_err(|()| anyhow::anyhow!("Could not normalize heading."))?;
    if let Some(element) = node.as_element() {
        replacement.attributes.borrow_mut().map = element.attributes.borrow().map.clone();
    }
    for child in node.children().collect::<Vec<_>>() {
        replacement.as_node().append(child);
    }
    node.insert_before(replacement.as_node().clone());
    node.detach();
    Ok(replacement.as_node().clone())
}

impl Page {
    fn parse(
        source: PathBuf,
        input: &str,
        config: &Config,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Result<Option<Self>> {
        let original = input.trim_start_matches('\u{feff}').replace("\r\n", "\n");
        let file = format!("content/{}", source.display());
        let first = original.lines().next().unwrap_or("");
        let (metadata, markdown, offset) = if matches!(first, "---" | "+++") {
            ensure!(original.len() > first.len(), "Unclosed metadata block.");
            let mut end = None;
            let mut position = first.len() + 1;
            for line in original[first.len() + 1..].split_inclusive('\n') {
                let marker = line.trim_end_matches('\n');
                if marker == first || (first == "---" && marker == "...") {
                    end = Some((position, position + line.len()));
                    break;
                }
                position += line.len();
            }
            let (end, start) = end.context(
                "Unclosed metadata block; repeat its opening delimiter on its own line.",
            )?;
            let text = &original[first.len() + 1..end];
            ensure!(text.len() <= 65_536, "Metadata must not exceed 64 KiB.");
            let metadata: Metadata = if first == "---" {
                serde_yaml_ng::from_str(text).context("Invalid YAML metadata")?
            } else {
                let metadata: Metadata = toml::from_str(text).context("Invalid TOML metadata")?;
                ensure!(
                    metadata.extra.is_empty(),
                    "Invalid TOML metadata: unknown fields {}.",
                    metadata
                        .extra
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                metadata
            };
            (
                metadata,
                &original[start..],
                original[..start].lines().count(),
            )
        } else {
            (Metadata::default(), original.as_str(), 0)
        };
        if metadata.draft {
            return Ok(None);
        }
        let (document, changes) = markup::markdown(markdown, config.compatibility)?;
        for change in changes {
            issue(config, diagnostics, &file, offset + 1, change)?;
        }
        let existing_h1 = document
            .select_first("h1")
            .ok()
            .map(|node| markup::text(node.as_node()));
        let fallback = without_order(
            source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Page"),
        )
        .replace(['-', '_'], " ");
        let title = metadata
            .title
            .unwrap_or_else(|| existing_h1.clone().unwrap_or(fallback));
        ensure!(
            !title.trim().is_empty(),
            "{file}: page title must not be empty."
        );
        let first_heading = document
            .select("h1,h2,h3,h4,h5,h6")
            .map_err(|()| anyhow::anyhow!("Invalid heading selector."))?
            .next()
            .map(|node| node.name.local.to_string());
        if first_heading.as_deref() != Some("h1") {
            if existing_h1.is_some() {
                issue(
                    config,
                    diagnostics,
                    &file,
                    offset + 1,
                    "The main heading must precede other headings; generated a leading H1 and demoted later H1s.",
                )?;
            }
            let generated = markup::parse(&format!("<h1>{}</h1>", escape(&title)));
            let heading = generated
                .select_first("h1")
                .map_err(|()| anyhow::anyhow!("Could not generate page heading."))?;
            document
                .select_first("body")
                .map_err(|()| anyhow::anyhow!("Missing page body."))?
                .as_node()
                .prepend(heading.as_node().clone());
        }
        let mut anchors = BTreeSet::new();
        for node in document
            .select("[id],a[name]")
            .map_err(|()| anyhow::anyhow!("Invalid anchor selector."))?
        {
            if node.name.local.starts_with('h')
                && matches!(
                    node.name.local.as_ref(),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                )
            {
                continue;
            }
            let id = {
                let attrs = node.attributes.borrow();
                attrs
                    .get("id")
                    .or_else(|| attrs.get("name"))
                    .map(str::to_string)
            };
            if let Some(id) = id {
                if reserved_id(&id) || !anchors.insert(id.clone()) {
                    issue(
                        config,
                        diagnostics,
                        &file,
                        offset + 1,
                        format!(
                            "Duplicate or reserved HTML identifier {id}; removed the duplicate identifier."
                        ),
                    )?;
                    node.attributes.borrow_mut().remove("id");
                    node.attributes.borrow_mut().remove("name");
                } else {
                    node.attributes.borrow_mut().insert("id", id);
                }
            }
        }
        let heading_nodes = document
            .select("h1,h2,h3,h4,h5,h6")
            .map_err(|()| anyhow::anyhow!("Invalid heading selector."))?
            .map(|node| node.as_node().clone())
            .collect::<Vec<_>>();
        let mut headings = Vec::new();
        let mut previous = 1;
        let mut seen_h1 = false;
        for mut node in heading_nodes {
            let element = node.as_element().context("Heading is not an element.")?;
            let original_level = element.name.local.as_ref()[1..].parse::<usize>()?;
            let mut level = original_level;
            let text = markup::text(&node);
            let line = original
                .find(&text)
                .map(|offset| original[..offset].lines().count().max(1))
                .unwrap_or(offset + 1);
            if text.is_empty() {
                issue(
                    config,
                    diagnostics,
                    &file,
                    line,
                    "Removed an empty heading.",
                )?;
                node.detach();
                continue;
            }
            if level == 1 && seen_h1 {
                issue(
                    config,
                    diagnostics,
                    &file,
                    line,
                    "Use one main heading; demoted an additional H1 to H2.",
                )?;
                level = 2;
            }
            if level > previous + 1 {
                issue(
                    config,
                    diagnostics,
                    &file,
                    line,
                    format!(
                        "Heading skips a level; changed H{level} to H{}.",
                        previous + 1
                    ),
                )?;
                level = previous + 1;
            }
            seen_h1 |= level == 1;
            previous = level;
            if level != original_level {
                node = rename_heading(&node, level)?;
            }
            let element = node.as_element().context("Heading is not an element.")?;
            let mut id = element
                .attributes
                .borrow()
                .get("id")
                .map(str::to_string)
                .unwrap_or_else(|| slug(&text));
            if id.is_empty() {
                id = "section".into();
            }
            if reserved_id(&id) {
                issue(
                    config,
                    diagnostics,
                    &file,
                    line,
                    "This heading uses a reserved identifier; prefixed it with content-.",
                )?;
                id = format!("content-{id}");
            }
            let base = id.clone();
            let mut suffix = 2;
            while !anchors.insert(id.clone()) {
                id = format!("{base}-{suffix}");
                suffix += 1;
            }
            let mut attributes = element.attributes.borrow_mut();
            attributes.insert("id", id.clone());
            headings.push(Heading { level, id, text });
        }
        let description = metadata.description.unwrap_or_else(|| {
            document
                .select("p")
                .ok()
                .and_then(|mut nodes| {
                    nodes.find_map(|node| {
                        let text = markup::text(node.as_node());
                        (!text.is_empty()).then_some(text)
                    })
                })
                .unwrap_or_default()
                .chars()
                .take(160)
                .collect()
        });
        let description = if description.trim().is_empty() {
            issue(
                config,
                diagnostics,
                &file,
                offset + 1,
                "Add an introductory paragraph or metadata description for search engines; generated a title-based description.",
            )?;
            format!("Documentation: {title}")
        } else {
            description
        };
        let navigation_title = match &metadata.navigation {
            Some(Navigation::Options { title: Some(title) }) => title.clone(),
            _ => title.clone(),
        };
        let in_navigation = !matches!(metadata.navigation, Some(Navigation::Visible(false)));
        let mut sections = vec![Section {
            heading: String::new(),
            anchor: String::new(),
            text: String::new(),
        }];
        for node in document.descendants() {
            if let Some(element) = node.as_element()
                && matches!(
                    element.name.local.as_ref(),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                )
            {
                let id = element
                    .attributes
                    .borrow()
                    .get("id")
                    .unwrap_or("")
                    .to_string();
                sections.push(Section {
                    heading: markup::text(&node),
                    anchor: id,
                    text: String::new(),
                });
            }
            if let Some(text) = node.as_text() {
                let excluded = node.ancestors().any(|ancestor| {
                    ancestor.as_element().is_some_and(|element| {
                        matches!(
                            element.name.local.as_ref(),
                            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "button"
                        ) || element.attributes.borrow().contains("data-code-status")
                    })
                });
                if !excluded && let Some(section) = sections.last_mut() {
                    section.text.push_str(&text.borrow());
                    section.text.push(' ');
                }
            }
        }
        Ok(Some(Self {
            route: route(&source, config.compatibility)?,
            source,
            title,
            navigation_title,
            in_navigation,
            description,
            order: metadata.order,
            headings,
            sections,
            html: String::new(),
            document,
            anchors,
            original,
        }))
    }

    fn line(&self, value: &str) -> usize {
        self.original
            .find(value)
            .map(|offset| {
                self.original[..offset]
                    .bytes()
                    .filter(|&b| b == b'\n')
                    .count()
                    + 1
            })
            .unwrap_or(1)
    }
}

impl Site {
    pub fn load(root: &Path) -> Result<Self> {
        let config = Config::load(root)?;
        let mut diagnostics = Vec::new();
        let mut pages = Vec::new();
        let mut seen = BTreeSet::new();
        for path in files(&root.join("content")).context("Read content directory")? {
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                issue(
                    &config,
                    &mut diagnostics,
                    &path.display().to_string(),
                    1,
                    "Content accepts only .md files; put other files in public/. File was not published.",
                )?;
                continue;
            }
            let source = path.strip_prefix(root.join("content"))?.to_path_buf();
            let text =
                fs::read_to_string(&path).with_context(|| format!("Read {}", path.display()))?;
            if let Some(page) = Page::parse(source, &text, &config, &mut diagnostics)
                .with_context(|| path.display().to_string())?
            {
                ensure!(
                    !matches!(page.route.as_str(), "/search/" | "/404/")
                        && !page.route.starts_with("/_site/"),
                    "{}: this URL is reserved by the generator.",
                    path.display()
                );
                ensure!(
                    seen.insert(page.route.clone()),
                    "{}: duplicate URL {}. Rename this page or its directory.",
                    path.display(),
                    page.route
                );
                pages.push(page);
            }
        }
        ensure!(
            seen.contains("/"),
            "Add content/index.md as your website's home page (it must not be a draft)."
        );
        pages.sort_by(|a, b| a.order.cmp(&b.order).then(a.source.cmp(&b.source)));
        let mut assets = BTreeMap::new();
        if root.join("public").exists() {
            for path in files(&root.join("public"))? {
                let relative = path.strip_prefix(root.join("public"))?.to_path_buf();
                let file = format!("public/{}", relative.display());
                ensure!(
                    !relative.starts_with("_site") && relative != Path::new("robots.txt"),
                    "{file}: reserved output filename."
                );
                let extension = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if !matches!(
                    extension.as_str(),
                    "png"
                        | "jpg"
                        | "jpeg"
                        | "gif"
                        | "webp"
                        | "avif"
                        | "ico"
                        | "pdf"
                        | "txt"
                        | "csv"
                        | "zip"
                        | "woff2"
                        | "svg"
                        | "html"
                        | "c"
                        | "cpp"
                        | "h"
                        | "java"
                        | "xml"
                        | "url"
                        | "jar"
                        | "tar"
                        | "gz"
                        | "tif"
                        | "tiff"
                ) {
                    issue(
                        &config,
                        &mut diagnostics,
                        &file,
                        1,
                        "Unsupported public attachment type; file was not published.",
                    )?;
                    continue;
                }
                ensure!(
                    relative
                        .to_str()
                        .is_some_and(|s| s.split('/').all(|part| !part.starts_with('.')
                            && !part
                                .chars()
                                .any(|c| c.is_control() || matches!(c, '\\' | ':' | '?' | '#')))),
                    "{file}: unsafe asset filename."
                );
                let mut asset = Asset {
                    source: path.clone(),
                    bytes: None,
                    dimensions: None,
                };
                if extension == "svg" {
                    let svg = markup::svg(&fs::read(&path)?).with_context(|| file.clone())?;
                    for change in svg.changes {
                        issue(&config, &mut diagnostics, &file, 1, change)?;
                    }
                    asset.bytes = Some(svg.bytes);
                    asset.dimensions = svg.dimensions;
                } else if extension == "html" {
                    let (clean, changes) = markup::sanitize(&fs::read_to_string(&path)?);
                    for change in changes {
                        issue(&config, &mut diagnostics, &file, 1, change)?;
                    }
                    asset.bytes = Some(format!("<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Documentation attachment</title></head><body>{clean}</body></html>", escape(&config.language)).into_bytes());
                } else if matches!(extension.as_str(), "tif" | "tiff") {
                    let image = image::open(&path)
                        .with_context(|| format!("{file}: invalid TIFF attachment"))?;
                    let mut data = std::io::Cursor::new(Vec::new());
                    image.write_to(&mut data, image::ImageFormat::Png)?;
                    let alias = relative.with_file_name(format!(
                        "{}.png",
                        relative
                            .file_name()
                            .context("Missing asset filename.")?
                            .to_string_lossy()
                    ));
                    ensure!(
                        !root.join("public").join(&alias).exists(),
                        "{file}: converted TIFF filename conflicts with an existing file."
                    );
                    assets.insert(
                        alias,
                        Asset {
                            source: path.clone(),
                            bytes: Some(data.into_inner()),
                            dimensions: Some((image.width() as usize, image.height() as usize)),
                        },
                    );
                }
                assets.insert(relative, asset);
            }
        }
        let routes = pages
            .iter()
            .enumerate()
            .map(|(index, page)| (page.route.clone(), index))
            .collect();
        let sources = pages
            .iter()
            .enumerate()
            .map(|(index, page)| (page.source.clone(), index))
            .collect();
        let mut site = Self {
            config,
            pages,
            assets,
            diagnostics: Vec::new(),
            routes,
            sources,
        };
        let mut navigation = site.config.navigation.take();
        if let Some(items) = &mut navigation {
            site.resolve_navigation(items)?;
        }
        site.config.navigation = navigation;
        for page in &site.pages {
            site.rewrite(page, &mut diagnostics)?;
        }
        for page in &mut site.pages {
            page.html = markup::body_html(&page.document)?;
        }
        diagnostics.sort();
        diagnostics.dedup();
        site.diagnostics = diagnostics;
        Ok(site)
    }

    fn resolve(
        &self,
        page: &Page,
        destination: &str,
        image: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Result<String> {
        let file = format!("content/{}", page.source.display());
        let line = page.line(destination);
        if destination.starts_with("//") {
            issue(
                &self.config,
                diagnostics,
                &file,
                line,
                "Use explicit https:// URLs; normalized a protocol-relative link to HTTPS.",
            )?;
            return self.resolve(page, &format!("https:{destination}"), image, diagnostics);
        }
        ensure!(
            !destination.contains('\\'),
            "{file}:{line}: Use URLs, not backslashes."
        );
        if let Ok(absolute) = Url::parse(destination) {
            ensure!(
                matches!(
                    absolute.scheme(),
                    "http" | "https" | "mailto" | "tel" | "ftp"
                ),
                "{file}:{line}: URL scheme {} is not allowed.",
                absolute.scheme()
            );
            ensure!(
                absolute.username().is_empty() && absolute.password().is_none(),
                "{file}:{line}: Links must not contain credentials."
            );
            if absolute.origin() != self.config.url.origin()
                || !absolute.path().starts_with(self.config.url.path())
            {
                if image {
                    issue(
                        &self.config,
                        diagnostics,
                        &file,
                        line,
                        format!(
                            "Image {destination} is remote; use local files in public/ to validate dimensions and availability."
                        ),
                    )?;
                }
                return Ok(absolute.to_string());
            }
            let mut relative = format!("/{}", &absolute.path()[self.config.url.path().len()..]);
            if let Some(query) = absolute.query() {
                relative.push_str(&format!("?{query}"));
            }
            if let Some(fragment) = absolute.fragment() {
                relative.push_str(&format!("#{fragment}"));
            }
            return self.resolve(page, &relative, image, diagnostics);
        }
        let dummy = Url::parse("https://content.invalid/")?;
        let base = if destination
            .split(['?', '#'])
            .next()
            .is_some_and(|s| s.ends_with(".md"))
        {
            dummy.join(&page.source.to_string_lossy())?
        } else {
            dummy.join(page.route.trim_start_matches('/'))?
        };
        let input =
            if destination.starts_with(self.config.url.path()) && self.config.url.path() != "/" {
                format!("/{}", &destination[self.config.url.path().len()..])
            } else {
                destination.into()
            };
        let resolved = base.join(&input)?;
        let path = percent_decode_str(resolved.path())
            .decode_utf8()?
            .to_string();
        let fragment = resolved
            .fragment()
            .map(|value| {
                percent_decode_str(value)
                    .decode_utf8()
                    .map(|s| s.into_owned())
            })
            .transpose()?;
        let suffix = format!(
            "{}{}",
            resolved
                .query()
                .map(|q| format!("?{q}"))
                .unwrap_or_default(),
            resolved
                .fragment()
                .map(|f| format!("#{f}"))
                .unwrap_or_default()
        );
        let asset = Path::new(path.trim_start_matches('/'));
        if self.assets.contains_key(asset) {
            let converted = asset.with_file_name(format!(
                "{}.png",
                asset
                    .file_name()
                    .context("Missing asset filename.")?
                    .to_string_lossy()
            ));
            let path = if image
                && matches!(
                    asset
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("tif" | "tiff")
                )
                && self.assets.contains_key(&converted)
            {
                format!("/{}", converted.to_string_lossy())
            } else {
                path
            };
            return Ok(format!("{}{suffix}", self.config.href(&path)));
        }
        if image {
            issue(
                &self.config,
                diagnostics,
                &file,
                line,
                format!(
                    "Image {destination} does not exist in public/; retained the authored URL."
                ),
            )?;
            return Ok(format!("{}{suffix}", self.config.href(&path)));
        }
        if matches!(path.as_str(), "/search" | "/search/" | "/search/index.html") {
            ensure!(
                fragment.as_deref().is_none_or(str::is_empty),
                "{file}:{line}: Search page links do not support heading fragments."
            );
            return Ok(format!("{}{suffix}", self.config.href("/search/")));
        }
        let target = if path.ends_with(".md") {
            self.sources.get(asset).map(|&index| &self.pages[index])
        } else {
            let clean = path.strip_suffix("index.html").unwrap_or(&path);
            let clean = if clean.ends_with('/') {
                clean.into()
            } else {
                format!("{clean}/")
            };
            self.routes.get(&clean).map(|&index| &self.pages[index])
        };
        let Some(target) = target else {
            issue(
                &self.config,
                diagnostics,
                &file,
                line,
                format!(
                    "Link {destination} does not point to a published page or public file; retained the authored URL."
                ),
            )?;
            return Ok(format!("{}{suffix}", self.config.href(&path)));
        };
        if let Some(anchor) = &fragment
            && !anchor.is_empty()
            && !target.anchors.contains(anchor)
        {
            issue(
                &self.config,
                diagnostics,
                &file,
                line,
                format!(
                    "Link {destination}: heading #{anchor} does not exist in {}; retained the authored fragment.",
                    target.source.display()
                ),
            )?;
        }
        Ok(format!("{}{suffix}", self.config.href(&target.route)))
    }

    fn resolve_navigation(&self, items: &mut [crate::config::Navigation]) -> Result<()> {
        let home = self
            .pages
            .iter()
            .find(|page| page.route == "/")
            .context("Navigation requires a published home page.")?;
        for item in items {
            if let Some(path) = &mut item.path {
                let mut diagnostics = Vec::new();
                *path = self
                    .resolve(home, path, false, &mut diagnostics)
                    .with_context(|| format!("site.toml: navigation item {}", item.title))?;
                ensure!(
                    diagnostics.is_empty(),
                    "site.toml: navigation item {}: {}",
                    item.title,
                    diagnostics[0].message
                );
            }
            self.resolve_navigation(&mut item.children)?;
        }
        Ok(())
    }

    fn rewrite(&self, page: &Page, diagnostics: &mut Vec<Diagnostic>) -> Result<()> {
        let file = format!("content/{}", page.source.display());
        for node in page
            .document
            .select("a[href],img[src]")
            .map_err(|()| anyhow::anyhow!("Invalid link selector."))?
        {
            let image = node.name.local.as_ref() == "img";
            let attribute = if image { "src" } else { "href" };
            let destination = node
                .attributes
                .borrow()
                .get(attribute)
                .unwrap_or("")
                .to_string();
            let href = self.resolve(page, &destination, image, diagnostics)?;
            node.attributes.borrow_mut().insert(attribute, href.clone());
            if image {
                let line = page.line(&destination);
                let (alt, decorative) = {
                    let attrs = node.attributes.borrow();
                    (
                        attrs.get("alt").map(str::to_string),
                        attrs.get("title") == Some("decorative"),
                    )
                };
                if alt.as_deref().is_none_or(|value| value.trim().is_empty()) && !decorative {
                    issue(
                        &self.config,
                        diagnostics,
                        &file,
                        line,
                        "Describe this image, or use an empty description with title \"decorative\"; retained an empty alt attribute.",
                    )?;
                    node.attributes.borrow_mut().insert("alt", String::new());
                }
                let url = self.config.url.join(&href)?;
                let key = percent_decode_str(
                    url.path()
                        .strip_prefix(self.config.url.path())
                        .unwrap_or(""),
                )
                .decode_utf8()?;
                if let Some(asset) = self.assets.get(Path::new(key.as_ref())) {
                    let dimensions = asset.dimensions.or_else(|| {
                        imagesize::size(&asset.source)
                            .ok()
                            .map(|size| (size.width, size.height))
                    });
                    if let Some((width, height)) = dimensions.filter(|(w, h)| *w > 0 && *h > 0) {
                        let number = |name| {
                            node.attributes
                                .borrow()
                                .get(name)
                                .and_then(|value| {
                                    value.trim_end_matches("px").parse::<usize>().ok()
                                })
                                .filter(|value| *value > 0)
                        };
                        let authored_width = number("width");
                        let authored_height = number("height");
                        let rendered_width = authored_width.unwrap_or_else(|| {
                            authored_height
                                .map(|h| (h * width / height).max(1))
                                .unwrap_or(width)
                        });
                        let rendered_height = authored_height.unwrap_or_else(|| {
                            authored_width
                                .map(|w| (w * height / width).max(1))
                                .unwrap_or(height)
                        });
                        let mut attrs = node.attributes.borrow_mut();
                        attrs.insert("width", rendered_width.to_string());
                        attrs.insert("height", rendered_height.to_string());
                    } else {
                        issue(
                            &self.config,
                            diagnostics,
                            &file,
                            line,
                            format!("Could not determine image dimensions for {destination}."),
                        )?;
                    }
                    if fs::metadata(&asset.source)?.len() > 1_048_576 {
                        issue(
                            &self.config,
                            diagnostics,
                            &file,
                            line,
                            format!(
                                "Image {destination} exceeds 1 MiB. Resize or compress it before publishing."
                            ),
                        )?;
                    }
                }
                let mut attrs = node.attributes.borrow_mut();
                attrs.insert("loading", "lazy".into());
                attrs.insert("decoding", "async".into());
            }
        }
        for node in page
            .document
            .select("pre")
            .map_err(|()| anyhow::anyhow!("Invalid code selector."))?
        {
            let mut attrs = node.attributes.borrow_mut();
            attrs.insert("tabindex", "0".into());
            attrs.insert("role", "group".into());
            attrs.insert("aria-label", "Code example".into());
        }
        for node in page
            .document
            .select("th")
            .map_err(|()| anyhow::anyhow!("Invalid table selector."))?
        {
            if !node.attributes.borrow().contains("scope") {
                node.attributes.borrow_mut().insert("scope", "col".into());
            }
        }
        let tables = page
            .document
            .select("table")
            .map_err(|()| anyhow::anyhow!("Invalid table selector."))?
            .map(|node| node.as_node().clone())
            .collect::<Vec<_>>();
        for table in tables {
            let wrapper = markup::parse(
                "<div class=\"sds-table-container\" tabindex=\"0\" role=\"group\" aria-label=\"Scrollable table\"></div>",
            );
            let wrapper = wrapper
                .select_first("body > div")
                .map_err(|()| anyhow::anyhow!("Could not render table wrapper."))?;
            table.insert_before(wrapper.as_node().clone());
            wrapper.as_node().append(table);
        }
        Ok(())
    }
}
