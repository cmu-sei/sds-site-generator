use crate::content::{Site, tokens};
use anyhow::Result;
use serde::Serialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Serialize)]
struct Document<'a> {
    title: &'a str,
    page: &'a str,
    url: String,
    excerpt: String,
}

pub(crate) fn write(site: &Site, output: &Path) -> Result<()> {
    type Posting = BTreeMap<usize, Vec<usize>>;
    let mut buckets: BTreeMap<String, BTreeMap<String, Posting>> = BTreeMap::new();
    let mut documents = Vec::new();
    for page in &site.pages {
        for section in &page.sections {
            if section.text.trim().is_empty() && section.heading.is_empty() {
                continue;
            }
            let id = documents.len();
            let mut position = 0;
            for field in [&page.title, &section.heading, &section.text] {
                for term in tokens(field) {
                    let bucket = format!("{:x}", term.chars().next().unwrap_or_default() as u32);
                    buckets
                        .entry(bucket)
                        .or_default()
                        .entry(term)
                        .or_default()
                        .entry(id)
                        .or_default()
                        .push(position);
                    position += 1;
                }
                // A phrase must not span unrelated title, heading, and body fields.
                position += 1;
            }
            documents.push(Document {
                title: if section.heading.is_empty() {
                    &page.title
                } else {
                    &section.heading
                },
                page: &page.title,
                url: format!(
                    "{}{}",
                    site.config.href(&page.route),
                    if section.anchor.is_empty() {
                        String::new()
                    } else {
                        format!("#{}", section.anchor)
                    }
                ),
                excerpt: section
                    .text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(240)
                    .collect(),
            });
        }
    }
    let directory = output.join("_site/search");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "count": documents.len(),
            "chunkSize": 64,
            "buckets": buckets.keys().collect::<Vec<_>>()
        }))?,
    )?;
    for (bucket, terms) in buckets {
        fs::write(
            directory.join(format!("terms-{bucket}.json")),
            serde_json::to_vec(&terms)?,
        )?;
    }
    for (index, chunk) in documents.chunks(64).enumerate() {
        fs::write(
            directory.join(format!("docs-{index}.json")),
            serde_json::to_vec(chunk)?,
        )?;
    }
    Ok(())
}
