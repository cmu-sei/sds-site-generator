use anyhow::{Context, Result, ensure};
use kuchiki::{NodeRef, traits::*};
use pulldown_cmark::{Options, Parser, html};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

pub(crate) fn code_icon(name: &str) -> String {
    static ICONS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("../assets/code-icons.json"))
            .expect("Bundled Lucide icons must be valid JSON")
    });
    ICONS[name].replacen("<svg", "<svg aria-hidden=\"true\"", 1)
}

pub(crate) fn documentation_icon(name: &str) -> &'static str {
    static ICONS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("../vendor/sds-lite/documentation-icons.json"))
            .expect("Bundled SDS documentation icons must be valid JSON")
    });
    ICONS[name].as_str()
}

pub(crate) fn parse(html: &str) -> NodeRef {
    kuchiki::parse_html().one(html)
}

pub(crate) fn body_html(document: &NodeRef) -> Result<String> {
    let body = document
        .select_first("body")
        .map_err(|()| anyhow::anyhow!("HTML document has no body."))?;
    Ok(body
        .as_node()
        .children()
        .map(|node| node.to_string())
        .collect())
}

pub(crate) fn text(node: &NodeRef) -> String {
    node.text_contents()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn safe_style(value: &str) -> String {
    value
        .split(';')
        .filter_map(|declaration| {
            let (property, value) = declaration.split_once(':')?;
            let property = property.trim().to_ascii_lowercase();
            let value = value.trim();
            let allowed = matches!(
                property.as_str(),
                "display"
                    | "flex"
                    | "flex-wrap"
                    | "flex-direction"
                    | "gap"
                    | "row-gap"
                    | "column-gap"
                    | "align-items"
                    | "justify-content"
                    | "grid-template-columns"
                    | "width"
                    | "height"
                    | "min-width"
                    | "max-width"
                    | "min-height"
                    | "max-height"
                    | "margin"
                    | "margin-top"
                    | "margin-bottom"
                    | "margin-left"
                    | "margin-right"
                    | "padding"
                    | "padding-top"
                    | "padding-bottom"
                    | "padding-left"
                    | "padding-right"
                    | "text-align"
                    | "vertical-align"
                    | "font-weight"
                    | "font-style"
                    | "font-size"
                    | "color"
                    | "background-color"
                    | "border"
                    | "border-width"
                    | "border-color"
                    | "border-style"
                    | "border-collapse"
                    | "white-space"
                    | "overflow-wrap"
                    | "text-decoration"
            );
            let lower = value.to_ascii_lowercase();
            if !allowed
                || value.is_empty()
                || lower.contains("url")
                || lower.contains("expression")
                || lower.contains("javascript")
                || !value
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || " -_#.%(),/".contains(c))
            {
                return None;
            }
            Some(format!("{property}:{value}"))
        })
        .collect::<Vec<_>>()
        .join(";")
}

pub(crate) fn sanitize(input: &str) -> (String, Vec<String>) {
    let document = parse(input);
    let mut changes = Vec::new();
    if let Ok(nodes) = document.select("*") {
        for node in nodes {
            let name = node.name.local.as_ref();
            if matches!(
                name,
                "script" | "style" | "iframe" | "object" | "embed" | "form" | "input"
            ) {
                changes.push(format!("Removed unsafe <{name}> markup."));
            }
            for (name, attribute) in &node.attributes.borrow().map {
                let name = name.local.as_ref();
                if name.starts_with("on")
                    || (matches!(name, "href" | "src")
                        && attribute
                            .value
                            .trim()
                            .to_ascii_lowercase()
                            .starts_with("javascript:"))
                {
                    changes.push(format!("Removed unsafe {name} attribute."));
                }
                if name == "style" {
                    let declarations = attribute
                        .value
                        .split(';')
                        .filter(|part| !part.trim().is_empty())
                        .count();
                    let clean = safe_style(&attribute.value);
                    if clean.split(';').filter(|part| !part.is_empty()).count() < declarations {
                        changes.push(
                            "Removed unsupported or unsafe inline style declarations.".into(),
                        );
                    }
                }
            }
        }
    }
    let mut builder = ammonia::Builder::default();
    builder
        .add_tags(&["site-code-block", "aside", "u", "sup", "sub"])
        .add_generic_attributes(&[
            "id",
            "class",
            "style",
            "role",
            "aria-label",
            "aria-labelledby",
            "aria-hidden",
            "aria-describedby",
            "tabindex",
        ])
        .add_tag_attributes("a", &["name"])
        .add_tag_attributes("img", &["width", "height", "loading", "decoding"])
        .add_tag_attributes("th", &["scope"])
        .add_tag_attributes("site-code-block", &["quality", "title"])
        .add_tag_attributes("div", &["data-code-slot", "data-sds-code-slot"])
        .add_tag_attributes(
            "aside",
            &[
                "data-sds-tone",
                "data-sds-variant",
                "data-sds-size",
                "data-sds-callout-color",
            ],
        )
        .link_rel(None)
        .attribute_filter(|_, attribute, value| {
            if attribute == "style" {
                let clean = safe_style(value);
                if clean.is_empty() {
                    None
                } else {
                    Some(Cow::Owned(clean))
                }
            } else {
                Some(Cow::Borrowed(value))
            }
        });
    let clean = builder.clean(input).to_string();
    let cleaned = parse(&clean);
    let mut retained_tags = BTreeSet::new();
    let mut retained_attributes = BTreeSet::new();
    for node in cleaned.descendants() {
        if let Some(element) = node.as_element() {
            let tag = element.name.local.to_string();
            retained_tags.insert(tag.clone());
            for (name, attribute) in &element.attributes.borrow().map {
                retained_attributes.insert((
                    tag.clone(),
                    name.local.to_string(),
                    attribute.value.clone(),
                ));
            }
        }
    }
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let tag = element.name.local.to_string();
            if !retained_tags.contains(&tag) {
                changes.push(format!("Removed unsupported or unsafe <{tag}> markup."));
                continue;
            }
            for (name, attribute) in &element.attributes.borrow().map {
                let name = name.local.to_string();
                if name != "style"
                    && !retained_attributes.contains(&(
                        tag.clone(),
                        name.clone(),
                        attribute.value.clone(),
                    ))
                {
                    changes.push(format!(
                        "Removed unsupported or unsafe {name} attribute from <{tag}>."
                    ));
                }
            }
        }
    }
    changes.sort();
    changes.dedup();
    (clean, changes)
}

pub(crate) fn markdown(input: &str) -> Result<(NodeRef, Vec<String>)> {
    let mut prepared = String::new();
    let mut fence: Option<(char, usize)> = None;
    let mut blocks: Vec<(&str, &str)> = Vec::new();
    let mut code_blocks = Vec::new();
    let mut callouts = Vec::new();
    let mut changes = Vec::new();
    for (index, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        let first = trimmed.chars().next();
        if matches!(first, Some('`' | '~')) {
            let character = first.unwrap_or_default();
            let length = trimmed.chars().take_while(|&c| c == character).count();
            if length >= 3 {
                match fence {
                    None => fence = Some((character, length)),
                    Some((current, minimum))
                        if current == character
                            && length >= minimum
                            && trimmed[length..].trim().is_empty() =>
                    {
                        fence = None
                    }
                    _ => {}
                }
            }
            prepared.push_str(line);
        } else if fence.is_none()
            && trimmed.strip_prefix("::callout").is_some_and(|properties| {
                properties.is_empty()
                    || properties.starts_with('{')
                    || properties.starts_with(char::is_whitespace)
            })
        {
            let properties = trimmed.trim_start_matches("::callout").trim();
            let properties = if properties.is_empty() {
                ""
            } else {
                properties
                    .strip_prefix('{')
                    .and_then(|value| value.strip_suffix('}'))
                    .with_context(|| format!("line {}: invalid callout properties.", index + 1))?
            };
            let properties = parse(&format!("<aside {properties}></aside>"));
            let element = properties
                .select_first("aside")
                .map_err(|()| anyhow::anyhow!("Invalid callout properties."))?;
            let attributes = element.attributes.borrow();
            ensure!(
                attributes
                    .map
                    .keys()
                    .all(|name| matches!(name.local.as_ref(), "type" | "variant" | "size")),
                "line {}: unsupported callout properties. Use type, variant, and size.",
                index + 1
            );
            let variant = attributes.get("type").unwrap_or("subtle");
            ensure!(
                matches!(variant, "subtle" | "outline" | "bold"),
                "line {}: callout type must be subtle, outline, or bold.",
                index + 1
            );
            let size = attributes.get("size").unwrap_or("md");
            ensure!(
                matches!(size, "xs" | "sm" | "md" | "lg"),
                "line {}: invalid callout size.",
                index + 1
            );
            let color = attributes.get("variant").unwrap_or("blue");
            let tone = match color {
                "blue" | "indigo" => "info",
                "green" | "teal" => "success",
                "orange" => "warning",
                "red" => "danger",
                "gray" => "neutral",
                "purple" => "accent",
                _ => anyhow::bail!("line {}: unsupported callout variant {color}.", index + 1),
            };
            prepared.push_str(&format!("<!--site-callout-start:{}-->", callouts.len()));
            callouts.push(format!("<aside class=\"sds-callout\" role=\"note\" data-sds-tone=\"{tone}\" data-sds-variant=\"{variant}\" data-sds-size=\"{size}\" data-sds-callout-color=\"{color}\">"));
            blocks.push(("callout", "default"));
        } else if fence.is_none()
            && trimmed
                .strip_prefix("::code-block")
                .is_some_and(|properties| {
                    properties.is_empty()
                        || properties.starts_with('{')
                        || properties.starts_with(char::is_whitespace)
                })
        {
            let properties = trimmed.trim_start_matches("::code-block").trim();
            let properties = if properties.is_empty() {
                ""
            } else {
                properties
                    .strip_prefix('{')
                    .and_then(|value| value.strip_suffix('}'))
                    .with_context(|| {
                        format!("line {}: invalid code-block properties.", index + 1)
                    })?
            };
            let properties = parse(&format!("<site-code-block {properties}></site-code-block>"));
            let element = properties.select_first("site-code-block").map_err(|()| {
                anyhow::anyhow!("line {}: invalid code-block properties.", index + 1)
            })?;
            let attributes = element.attributes.borrow();
            ensure!(
                attributes
                    .map
                    .keys()
                    .all(|name| matches!(name.local.as_ref(), "quality" | "title")),
                "line {}: unsupported code-block properties. Use title and quality.",
                index + 1
            );
            let quality = attributes.get("quality").unwrap_or("average");
            ensure!(
                matches!(quality, "good" | "average" | "bad"),
                "line {}: code-block quality must be good, average, or bad.",
                index + 1
            );
            prepared.push_str(&format!("<!--site-code-start:{}-->", code_blocks.len()));
            code_blocks.push((
                quality.to_string(),
                attributes.get("title").unwrap_or("").to_string(),
            ));
            blocks.push(("code-block", "default"));
        } else if fence.is_none()
            && blocks.last().is_some_and(|block| block.0 == "code-block")
            && matches!(trimmed, "#default" | "#complete")
        {
            let slot = &trimmed[1..];
            if blocks.last().map(|block| block.1) != Some(slot) {
                prepared.push_str(&format!("<!--site-code-slot:{slot}-->"));
                blocks.last_mut().unwrap().1 = if slot == "complete" {
                    "complete"
                } else {
                    "default"
                };
            }
        } else if fence.is_none()
            && !blocks.is_empty()
            && (trimmed == "::"
                || trimmed
                    .strip_prefix("::")
                    .is_some_and(|tail| tail.starts_with(char::is_whitespace)))
        {
            let block = blocks.pop().unwrap();
            prepared.push_str(if block.0 == "code-block" {
                "<!--site-code-end-->"
            } else {
                "<!--site-callout-end-->"
            });
            if !trimmed[2..].trim().is_empty() {
                prepared.push_str("\n\n");
                prepared.push_str(trimmed[2..].trim());
            }
        } else {
            if fence.is_none()
                && trimmed.starts_with("::")
                && trimmed.chars().nth(2).is_some_and(char::is_alphabetic)
            {
                changes.push(format!(
                    "line {}: unsupported MDC component {}; rendered as text.",
                    index + 1,
                    trimmed
                ));
            }
            prepared.push_str(line);
        }
        prepared.push('\n');
    }
    if !blocks.is_empty() {
        for block in blocks.iter().rev() {
            changes.push(format!("Unclosed ::{}; closed it at the end of the page. End each component with a line containing ::.", block.0));
            prepared.push_str(if block.0 == "code-block" {
                "\n<!--site-code-end-->\n"
            } else {
                "\n<!--site-callout-end-->\n"
            });
        }
    }
    let mut output = String::new();
    html::push_html(
        &mut output,
        Parser::new_ext(
            &prepared,
            Options::ENABLE_TABLES
                | Options::ENABLE_STRIKETHROUGH
                | Options::ENABLE_SMART_PUNCTUATION,
        ),
    );
    for (index, (quality, title)) in code_blocks.iter().enumerate() {
        output = output.replace(
            &format!("<!--site-code-start:{index}-->"),
            &format!("<site-code-block quality=\"{quality}\" title=\"{}\"><div data-sds-code-slot=\"default\">", crate::content::escape(title)),
        );
    }
    for slot in ["default", "complete"] {
        output = output.replace(
            &format!("<!--site-code-slot:{slot}-->"),
            &format!("</div><div data-sds-code-slot=\"{slot}\">"),
        );
    }
    output = output.replace("<!--site-code-end-->", "</div></site-code-block>");
    for (index, opening) in callouts.iter().enumerate() {
        output = output.replace(&format!("<!--site-callout-start:{index}-->"), opening);
    }
    output = output.replace("<!--site-callout-end-->", "</aside>");
    let (clean, removed) = sanitize(&output);
    changes.extend(removed);
    let document = parse(&clean);
    if let Ok(nodes) = document.select("site-code-block") {
        for node in nodes {
            let quality = node
                .attributes
                .borrow()
                .get("quality")
                .unwrap_or("average")
                .to_string();
            let label = match quality.as_str() {
                "good" => "Compliant code",
                "bad" => "Non-compliant code",
                _ => "Code example",
            };
            node.attributes
                .borrow_mut()
                .insert("class", "sds-code-block".into());
            node.attributes.borrow_mut().insert(
                "data-sds-tone",
                match quality.as_str() {
                    "good" => "info",
                    "bad" => "danger",
                    _ => "neutral",
                }
                .into(),
            );
            let title = node
                .attributes
                .borrow()
                .get("title")
                .unwrap_or("")
                .to_string();
            node.attributes.borrow_mut().remove("title");
            let title = if title.is_empty() {
                String::new()
            } else {
                format!("<strong>{}</strong>", crate::content::escape(&title))
            };
            let quality = if quality == "average" {
                String::new()
            } else {
                format!(
                    "<span class=\"sds-badge\" data-sds-tone=\"{}\" role=\"img\" title=\"{label}\" aria-label=\"{label}\">{}</span>",
                    if quality == "bad" { "danger" } else { "info" },
                    code_icon(if quality == "bad" {
                        "triangle-alert"
                    } else {
                        "circle-check"
                    })
                )
            };
            let complete = if node
                .as_node()
                .select_first("[data-sds-code-slot='complete']")
                .is_ok()
            {
                format!(
                    "<button class=\"sds-button\" type=\"button\" data-sds-shape=\"icon\" data-sds-density=\"compact\" data-sds-variant=\"text\" data-sds-action=\"toggle-complete-code\" aria-label=\"Toggle complete code\" title=\"Toggle complete code\" aria-pressed=\"false\" hidden>{}{}</button>",
                    code_icon("eye-off"),
                    code_icon("eye")
                )
            } else {
                String::new()
            };
            let header = parse(&format!(
                "<header>{title}{quality}<nav aria-label=\"Code actions\">{complete}<button class=\"sds-button\" type=\"button\" data-sds-shape=\"icon\" data-sds-density=\"compact\" data-sds-variant=\"text\" data-sds-action=\"copy-code\" aria-label=\"Copy to clipboard\" title=\"Copy to clipboard\" hidden>{}</button><button class=\"sds-button\" type=\"button\" data-sds-shape=\"icon\" data-sds-density=\"compact\" data-sds-variant=\"text\" data-sds-action=\"toggle-code-fullscreen\" aria-label=\"Toggle fullscreen\" title=\"Enter fullscreen\" aria-pressed=\"false\" hidden>{}{}</button></nav></header>",
                code_icon("copy"),
                code_icon("maximize-2"),
                code_icon("minimize-2")
            ));
            let header = header
                .select_first("body > header")
                .map_err(|()| anyhow::anyhow!("Could not render code example header."))?;
            node.as_node().prepend(header.as_node().clone());
            let status = parse("<output class=\"sds-sr-only\"></output>");
            node.as_node()
                .append(status.select_first("output").unwrap().as_node().clone());
            for slot in node
                .as_node()
                .select("[data-sds-code-slot='complete']")
                .into_iter()
                .flatten()
            {
                slot.attributes.borrow_mut().insert("hidden", String::new());
            }
        }
    }
    if let Ok(tables) = document.select("table") {
        for table in tables {
            let template = parse(&format!(
                "<template data-sds-sort-icon>{}</template>",
                code_icon("chevron-right").replacen("<svg", "<svg data-sds-sort-chevron", 1)
            ));
            if let Ok(icon) = template.select_first("template") {
                if let Some(contents) = &icon.as_node().as_element().unwrap().template_contents {
                    for child in contents.children().collect::<Vec<_>>() {
                        icon.as_node().append(child);
                    }
                }
                table.as_node().append(icon.as_node().clone());
            }
        }
    }
    Ok((document, changes))
}

pub(crate) struct SanitizedSvg {
    pub bytes: Vec<u8>,
    pub changes: Vec<String>,
    pub dimensions: Option<(usize, usize)>,
}

pub(crate) fn svg(input: &[u8]) -> Result<SanitizedSvg> {
    let mut document = xmltree::Element::parse(input).context("Invalid SVG XML")?;
    ensure!(document.name == "svg", "SVG must have an svg root element.");
    let mut changes = Vec::new();
    fn clean(node: &mut xmltree::Element, changes: &mut Vec<String>) {
        node.attributes.retain(|name, value| {
            let lower = value.to_ascii_lowercase();
            let keep = !name.to_ascii_lowercase().starts_with("on")
                && (!name.ends_with("href") || value.starts_with('#'))
                && !lower.contains("javascript:")
                && !lower.contains("://")
                && !value.contains('\\')
                && !lower.contains('@')
                && !lower.contains("expression")
                && local_svg_urls(&lower);
            if !keep {
                changes.push(format!("Removed unsafe SVG attribute {name}."));
            }

            fn local_svg_urls(value: &str) -> bool {
                let mut remaining = value;
                while let Some((_, tail)) = remaining.split_once("url(") {
                    let Some((reference, tail)) = tail.split_once(')') else {
                        return false;
                    };
                    let reference = reference.trim().trim_matches(['\'', '"']);
                    if !reference.strip_prefix('#').is_some_and(|id| {
                        !id.is_empty()
                            && id
                                .chars()
                                .all(|c| c.is_alphanumeric() || "_-:.".contains(c))
                    }) {
                        return false;
                    }
                    remaining = tail;
                }
                true
            }
            keep
        });
        node.children.retain_mut(|child| {
            if let xmltree::XMLNode::Element(element) = child {
                if matches!(
                    element.name.as_str(),
                    "script"
                        | "foreignObject"
                        | "iframe"
                        | "animate"
                        | "set"
                        | "animateMotion"
                        | "animateTransform"
                ) {
                    changes.push(format!("Removed unsafe SVG element {}.", element.name));
                    return false;
                }
                if element.name == "style" {
                    let text = element.get_text().unwrap_or_default().to_ascii_lowercase();
                    if text.contains('@')
                        || text.contains('\\')
                        || text.contains("url")
                        || text.contains("expression")
                    {
                        changes.push("Removed unsupported SVG stylesheet.".into());
                        return false;
                    }
                }
                clean(element, changes);
            }
            true
        });
    }
    clean(&mut document, &mut changes);
    let dimension = |name| {
        document
            .attributes
            .get(name)
            .and_then(|value| value.trim_end_matches("px").parse::<f64>().ok())
            .filter(|value| *value > 0.0 && *value <= 100_000.0)
            .map(|value| value.ceil() as usize)
    };
    let dimensions = dimension("width").zip(dimension("height")).or_else(|| {
        let values = document
            .attributes
            .get("viewBox")?
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(str::parse::<f64>)
            .collect::<std::result::Result<Vec<_>, _>>()
            .ok()?;
        (values.len() == 4
            && values[2] > 0.0
            && values[3] > 0.0
            && values[2] <= 100_000.0
            && values[3] <= 100_000.0)
            .then(|| (values[2].ceil() as usize, values[3].ceil() as usize))
    });
    let mut output = Vec::new();
    document.write(&mut output)?;
    Ok(SanitizedSvg {
        bytes: output,
        changes,
        dimensions,
    })
}
