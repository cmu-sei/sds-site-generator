use kuchiki::traits::TendrilSink;
use std::{fs, path::Path};

#[test]
fn semantic_html_retains_named_anchors_and_row_header_scope() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/table.md"), "# Table\n\nA semantic HTML table.\n\n<a name=\"legacy-anchor\"></a>\n\n[Existing anchor](#legacy-anchor)\n\n<table><tr><th scope=\"row\">Name</th><td>Value</td></tr></table>\n").unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let html = fs::read_to_string(root.join("dist/table/index.html")).unwrap();
    assert!(html.contains("scope=\"row\""));
    assert!(html.contains("href=\"/docs/table/#legacy-anchor\""));
    assert!(html.contains("id=\"legacy-anchor\""));
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn yaml_front_matter_and_existing_semantic_html_work_without_rewriting() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("site");
    site_generator::create(&root, "Compatibility", "https://example.org/docs/").unwrap();
    fs::write(root.join("content/existing.md"), "---\ntitle: Existing page\ndescription: An existing documentation page.\ntags:\n  - documentation\nnavigation:\n  title: Existing\n---\n# Existing page\n\n<div class=\"legacy-class\"><p>Existing <strong>HTML</strong> content.</p></div>\n").unwrap();
    site_generator::build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/existing/index.html")).unwrap();
    assert!(html.contains("Existing <strong>HTML</strong> content."));
    assert!(!html.contains("<p>---"));
    assert!(html.contains(">Existing</a>"));
}

fn compatible_fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("content")).unwrap();
    fs::create_dir(temp.path().join("public")).unwrap();
    fs::write(
        temp.path().join("site.toml"),
        "name = 'Compatibility'\nurl = 'https://example.org/docs/'\ncompatibility = true\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("content/index.md"),
        "# Home\n\nA compatibility test website.",
    )
    .unwrap();
    temp
}

#[test]
fn legacy_relative_downloads_use_public_root_only_after_local_resolution() {
    for compatibility in [false, true] {
        let temp = compatible_fixture();
        let root = temp.path();
        fs::write(
            root.join("site.toml"),
            format!("name = 'Downloads'\nurl = 'https://example.org/docs/'\ncompatibility = {compatibility}\n"),
        )
        .unwrap();
        fs::create_dir_all(root.join("public/attachments")).unwrap();
        fs::create_dir_all(root.join("public/guide/attachments")).unwrap();
        fs::write(root.join("public/attachments/read me.txt"), "Root download").unwrap();
        fs::write(root.join("public/attachments/manual.txt"), "Root manual").unwrap();
        fs::write(
            root.join("public/guide/attachments/manual.txt"),
            "Local manual",
        )
        .unwrap();
        fs::write(
            root.join("content/guide.md"),
            "# Guide\n\nImported downloads.\n\n[Root](attachments/read%20me.txt?download=1#notes)\n\n[Local](attachments/manual.txt)\n\n[Missing](attachments/missing.txt)\n",
        )
        .unwrap();
        let result = site_generator::build(root);
        if !compatibility {
            assert!(
                format!("{:#}", result.err().unwrap())
                    .contains("Link attachments/read%20me.txt?download=1#notes")
            );
            continue;
        }
        let report = result.unwrap();
        let document = kuchiki::parse_html()
            .one(fs::read_to_string(root.join("dist/guide/index.html")).unwrap());
        let links = document
            .select("article a[href*='attachments/']")
            .unwrap()
            .map(|node| node.attributes.borrow().get("href").unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            links,
            [
                "/docs/attachments/read%20me.txt?download=1#notes",
                "/docs/guide/attachments/manual.txt",
                "/docs/guide/attachments/missing.txt",
            ]
        );
        assert_eq!(report.warnings.len(), 1);
        assert!(
            report.warnings[0]
                .message
                .contains("attachments/missing.txt")
        );
    }
}

#[test]
fn legacy_heading_fragments_preserve_punctuation_underscores_and_duplicate_order() {
    let temp = compatible_fixture();
    fs::write(
        temp.path().join("content/anchors.md"),
        "# Heading_V\n\nAn imported page.\n\n## Noncompliant Code Example\n\nFirst.\n\n## Noncompliant Code Example\n\nSecond.\n\n## Noncompliant Code Example\n\nThird.\n\n## C++ / C: Rules\n\nLanguage rules.\n\n## 2013\n\nA year.\n\n## Conference/Workshop Proceedings\n\nProceedings.\n",
    )
    .unwrap();
    site_generator::build(temp.path()).unwrap();
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(temp.path().join("dist/anchors/index.html")).unwrap());
    let ids = document
        .select("article h1,article h2")
        .unwrap()
        .map(|node| node.attributes.borrow().get("id").unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
            "heading_v",
            "noncompliant-code-example",
            "noncompliant-code-example-1",
            "noncompliant-code-example-2",
            "c-c-rules",
            "_2013",
            "conferenceworkshop-proceedings",
        ]
    );
}

#[test]
fn raw_html_code_in_tables_remains_literal_while_prose_gets_smart_punctuation() {
    for compatibility in [false, true] {
        let temp = compatible_fixture();
        fs::write(
            temp.path().join("site.toml"),
            format!("name = 'Code'\nurl = 'https://example.org/docs/'\ncompatibility = {compatibility}\n"),
        )
        .unwrap();
        fs::write(
            temp.path().join("content/code.md"),
            "# Code\n\n\"Prose\" ... is typeset.\n\n| Example |\n| --- |\n| <pre><code>if (a &lt; b) ...<br />\"literal\" -- &amp; &#39;quotes&#39;</code></pre> |\n\nInline <code>\"raw\" ... --</code> and `\"fenced\" ... --`.\n",
        )
        .unwrap();
        site_generator::build(temp.path()).unwrap();
        let document = kuchiki::parse_html()
            .one(fs::read_to_string(temp.path().join("dist/code/index.html")).unwrap());
        assert_eq!(
            document
                .select_first("article pre code")
                .unwrap()
                .text_contents(),
            "if (a < b) ...\"literal\" -- & 'quotes'"
        );
        assert_eq!(
            document
                .select_first("article p code")
                .unwrap()
                .text_contents(),
            "\"raw\" ... --"
        );
        assert!(
            document
                .select_first("article p")
                .unwrap()
                .text_contents()
                .contains('\u{201c}')
        );
    }
}

#[test]
fn legacy_raw_slug_collisions_do_not_steal_later_unique_fragments() {
    let temp = compatible_fixture();
    fs::write(
        temp.path().join("content/collisions.md"),
        "# Collisions\n\nAn imported page.\n\n## Example\n\nFirst.\n\n## Example ( `<` )\n\nSecond.\n\n## Example ( `<>` )\n\nThird.\n",
    )
    .unwrap();
    site_generator::build(temp.path()).unwrap();
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(temp.path().join("dist/collisions/index.html")).unwrap());
    let ids = document
        .select("article h2")
        .unwrap()
        .map(|node| node.attributes.borrow().get("id").unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["example", "example-duplicate-1", "example-1"]);
}

#[test]
fn callouts_render_shared_recipes_and_preserve_nested_code_and_fenced_literals() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/callouts.md"), "# Callouts\n\nSupported shared components.\n\n::callout{variant='outline' tone='warning' size='sm'}\n**Warning:** [Home](index.md).\n\n::code-block{quality='good'}\n```text\n::callout{tone=\"danger\"}\nLiteral example.\n::\n```\n::\n\n::callout{tone='success' variant='bold'}\nNested note.\n::\n\nAfter nested components.\n::\n\nAfter the callout.\n").unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let html = fs::read_to_string(root.join("dist/callouts/index.html")).unwrap();
    let document = kuchiki::parse_html().one(html.as_str());
    let warning = document.select_first("aside.sds-callout[data-sds-tone='warning'][data-sds-variant='outline'][data-sds-size='sm']").unwrap();
    assert!(warning.as_node().select_first("strong").is_ok());
    assert!(warning.as_node().select_first("a[href='/docs/']").is_ok());
    assert!(
        warning
            .as_node()
            .select_first(".sds-code-block pre code")
            .is_ok()
    );
    assert!(
        warning
            .as_node()
            .select_first("aside.sds-callout[data-sds-tone='success']")
            .is_ok()
    );
    assert!(html.contains("::callout{tone="));
    assert_eq!(document.select("aside.sds-callout").unwrap().count(), 2);
    assert!(!warning.text_contents().contains("After the callout."));
}

#[test]
fn callouts_support_all_semantic_tones_variants_and_sizes() {
    let temp = compatible_fixture();
    let mut source = String::from("# Callout recipes\n\nSupported shared recipes.\n\n");
    let tones = ["neutral", "accent", "info", "success", "warning", "danger"];
    for tone in tones {
        for variant in ["", "outline", "bold"] {
            for size in ["xs", "sm", "md", "lg"] {
                let property = if variant.is_empty() {
                    String::new()
                } else {
                    format!(" variant='{variant}'")
                };
                source.push_str(&format!(
                    "::callout{{tone=\"{tone}\"{property} size='{size}'}}\n**{tone}** note.\n::\n\n"
                ));
            }
        }
    }
    fs::write(temp.path().join("content/recipes.md"), source).unwrap();
    for compatibility in [false, true] {
        fs::write(
            temp.path().join("site.toml"),
            format!("name = 'Callouts'\nurl = 'https://example.org/docs/'\ncompatibility = {compatibility}\n"),
        )
        .unwrap();
        let report = site_generator::build(temp.path()).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(temp.path().join("dist/recipes/index.html")).unwrap());
    assert_eq!(
        document
            .select("aside.sds-callout[role='note']")
            .unwrap()
            .count(),
        72
    );
    for tone in tones {
        assert_eq!(
            document
                .select(&format!("aside[data-sds-tone='{tone}']"))
                .unwrap()
                .count(),
            12
        );
    }
    assert!(document.select_first("[data-sds-callout-color]").is_err());
}

#[test]
fn callouts_default_to_the_native_neutral_subtle_medium_recipe() {
    let temp = compatible_fixture();
    fs::write(
        temp.path().join("content/default.md"),
        "# Defaults\n\nNative defaults.\n\n::callout\nA neutral note.\n::\n",
    )
    .unwrap();
    site_generator::build(temp.path()).unwrap();
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(temp.path().join("dist/default/index.html")).unwrap());
    let callout = document.select_first("aside.sds-callout").unwrap();
    let attributes = callout.attributes.borrow();
    assert_eq!(attributes.get("data-sds-tone"), Some("neutral"));
    assert_eq!(attributes.get("data-sds-size"), Some("md"));
    assert!(!attributes.contains("data-sds-variant"));
    assert!(!attributes.contains("data-sds-callout-color"));
}

#[test]
fn callouts_reject_unknown_properties_and_invalid_recipes() {
    for properties in [
        "onclick='alert(1)'",
        "style='color:red'",
        "variant='unknown'",
        "tone='unknown'",
        "tone='blue'",
        "variant='subtle'",
        "type='unknown'",
        "tone='warning' type='outline'",
        "tone='warning' variant='orange'",
        "size='unknown'",
    ] {
        let temp = compatible_fixture();
        fs::write(
            temp.path().join("content/invalid.md"),
            format!("# Invalid\n\nInvalid properties.\n\n::callout{{{properties}}}\nNote.\n::\n"),
        )
        .unwrap();
        let error = site_generator::build(temp.path())
            .err()
            .expect("invalid callout must fail");
        assert!(format!("{error:#}").contains("callout"), "{error:#}");
    }
}

#[test]
fn legacy_callouts_require_compatibility_and_report_migration_warnings() {
    for compatibility in [false, true] {
        let temp = compatible_fixture();
        fs::write(
            temp.path().join("site.toml"),
            format!("name = 'Callouts'\nurl = 'https://example.org/docs/'\ncompatibility = {compatibility}\n"),
        )
        .unwrap();
        let colors = [
            ("blue", "info"),
            ("indigo", "info"),
            ("green", "success"),
            ("teal", "success"),
            ("orange", "warning"),
            ("red", "danger"),
            ("gray", "neutral"),
            ("purple", "accent"),
        ];
        let mut source = String::from("# Legacy callouts\n\nImported content.\n\n");
        for (color, _) in colors {
            for variant in ["subtle", "outline", "bold"] {
                source.push_str(&format!("::callout{{variant='{color}' type='{variant}' size='sm'}}\nA legacy note.\n::\n\n"));
            }
        }
        source.push_str("::callout{type='outline'}\nDefault legacy color.\n::\n\n::callout{variant='orange'}\nDefault legacy appearance.\n::\n");
        fs::write(temp.path().join("content/legacy.md"), source).unwrap();
        let result = site_generator::build(temp.path());
        if !compatibility {
            let error = result
                .err()
                .expect("normal mode must reject legacy callouts");
            let message = format!("{error:#}");
            assert!(message.contains("compatibility"), "{message}");
            continue;
        }
        let report = result.unwrap();
        assert_eq!(report.warnings.len(), 26);
        assert!(
            report
                .warnings
                .iter()
                .all(|warning| warning.message.contains("Legacy callout"))
        );
        let document = kuchiki::parse_html()
            .one(fs::read_to_string(temp.path().join("dist/legacy/index.html")).unwrap());
        let callouts = document
            .select("aside.sds-callout")
            .unwrap()
            .collect::<Vec<_>>();
        for ((color, tone), recipes) in colors.into_iter().zip(callouts.chunks(3)) {
            for (variant, callout) in [None, Some("outline"), Some("bold")]
                .into_iter()
                .zip(recipes)
            {
                let attributes = callout.attributes.borrow();
                assert_eq!(attributes.get("data-sds-tone"), Some(tone), "{color}");
                assert_eq!(attributes.get("data-sds-variant"), variant, "{color}");
                assert_eq!(attributes.get("data-sds-size"), Some("sm"));
                assert_eq!(
                    attributes.get("data-sds-callout-color"),
                    matches!(color, "indigo" | "teal").then_some(color)
                );
            }
        }
        let default_color = callouts[24].attributes.borrow();
        assert_eq!(default_color.get("data-sds-tone"), Some("info"));
        assert_eq!(default_color.get("data-sds-variant"), Some("outline"));
        let default_appearance = callouts[25].attributes.borrow();
        assert_eq!(default_appearance.get("data-sds-tone"), Some("warning"));
        assert!(!default_appearance.contains("data-sds-variant"));
        let status = std::process::Command::new(env!("CARGO_BIN_EXE_sds-site-generator"))
            .args(["check", "--strict"])
            .arg(temp.path())
            .output()
            .unwrap();
        assert!(!status.status.success());
        assert!(String::from_utf8_lossy(&status.stderr).contains("Strict validation failed"));
    }
}

#[test]
fn documentation_examples_use_shared_code_blocks() {
    let temp = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs-site");
    fs::copy(source.join("site.toml"), temp.path().join("site.toml")).unwrap();
    for directory in ["content", "public"] {
        copy_tree(&source.join(directory), &temp.path().join(directory));
    }
    let report = site_generator::build(temp.path()).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    fn check_pages(directory: &Path) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                check_pages(&path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "html")
            {
                let document = kuchiki::parse_html().one(fs::read_to_string(&path).unwrap());
                for code in document.select(".sds-prose pre").unwrap() {
                    assert!(
                        code.as_node().ancestors().any(|node| {
                            node.as_element().is_some_and(|element| {
                                element.name.local.as_ref() == "site-code-block"
                            })
                        }),
                        "{}: code example is missing its code-block wrapper",
                        path.display()
                    );
                }
            }
        }
    }
    check_pages(&temp.path().join("dist"));
}

#[test]
fn plain_fences_and_tables_emit_shared_component_markup() {
    let temp = compatible_fixture();
    fs::write(temp.path().join("content/plain.md"), "# Plain components\n\nStandard Markdown components.\n\n```text\nPlain code.\n```\n\n| Name | Score |\n| --- | --- |\n| Beta | 9.12 |\n| Alpha | 9.5 |\n").unwrap();
    let report = site_generator::build(temp.path()).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(temp.path().join("dist/plain/index.html")).unwrap());
    assert!(
        document
            .select_first(".sds-prose pre[tabindex='0'] code")
            .is_ok()
    );
    let template = document
        .select_first("table template[data-sds-sort-icon]")
        .unwrap();
    let contents = template
        .as_node()
        .as_element()
        .unwrap()
        .template_contents
        .as_ref()
        .unwrap();
    assert!(contents.select_first("svg[data-sds-sort-chevron]").is_ok());
    assert_eq!(document.select("table tbody tr").unwrap().count(), 2);
    assert_eq!(document.select("table button").unwrap().count(), 0);
}

#[test]
fn compatibility_preserves_underscores_in_imported_page_routes() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::create_dir(root.join("content/01.analyzer_versions")).unwrap();
    fs::write(
        root.join("content/01.analyzer_versions/02.CodeSonar_V.md"),
        "# CodeSonar_V\n\nAn imported analyzer page.\n\n[Analyzer](/analyzer_versions/codesonar_v)\n\n## Version_details\n",
    )
    .unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let html =
        fs::read_to_string(root.join("dist/analyzer_versions/codesonar_v/index.html")).unwrap();
    assert!(html.contains("href=\"/docs/analyzer_versions/codesonar_v/\""));
    assert!(html.contains("id=\"version_details\""));
}

#[test]
fn legacy_code_examples_preserve_fences_and_enhance_without_vue() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/examples.md"), "# Examples\n\nExamples remain useful without JavaScript.\n\n::code-block{quality=\"bad\"}\n```html\n<script>literal code, not executable markup</script>\n::code-block{quality=\"good\"}\n```\n:: Text after the example.\n\n::code-block{quality=\"good\"}\n```c\nreturn 0;\n```\n::\n").unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let html = fs::read_to_string(root.join("dist/examples/index.html")).unwrap();
    assert_eq!(html.matches("<site-code-block ").count(), 2);
    assert!(html.contains("Non-compliant code"));
    assert!(html.contains("Compliant code"));
    assert!(
        kuchiki::parse_html()
            .one(html.as_str())
            .select_first(
                ".sds-badge[role='img'][aria-label='Compliant code'][title='Compliant code']"
            )
            .is_ok()
    );
    assert!(html.contains("&lt;script&gt;literal code, not executable markup&lt;/script&gt;"));
    assert!(html.contains("::code-block{quality="));
    assert!(html.contains("Text after the example."));
    assert!(html.contains("data-sds-action=\"copy-code\""));
    assert!(html.contains("sds-lite/documentation.js"));
}

#[test]
fn code_examples_preserve_titles_quality_and_complete_code_slots() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(
        root.join("content/examples.md"),
        "# Examples\n\nInteractive imported examples.\n\n::code-block{title='Validate <parameters>' quality='good'}\n#default\n```c\nreturn 0;\n```\n#complete\n```c\nint main(void) {\n  return 0;\n}\n```\n::\n\n::code-block{quality='average'}\n```text\nA neutral example.\n```\n::\n",
    )
    .unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let html = fs::read_to_string(root.join("dist/examples/index.html")).unwrap();
    assert!(html.contains("Validate &lt;parameters&gt;"));
    assert!(html.contains("Compliant code"));
    assert!(html.contains("quality=\"average\""));
    assert!(html.contains("data-sds-code-slot=\"default\""));
    assert!(html.contains("data-sds-code-slot=\"complete\""));
    assert!(html.contains("data-sds-action=\"toggle-complete-code\""));
    assert!(html.contains("data-sds-action=\"toggle-code-fullscreen\""));
    assert!(html.contains("aria-label=\"Copy to clipboard\""));
    assert!(!html.contains("id=\"complete\""));
    assert!(!html.contains("id=\"default\""));
}

#[test]
fn compatibility_reports_repairs_and_strict_cli_fails() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/legacy.md"), "# Legacy\n\nAn imported page.\n\n### Skipped\n\n<script>alert(1)</script><div onclick=\"alert(1)\">Retained safe text.</div>\n\n::unknown-widget\n").unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.message.contains("skips a level"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.message.contains("script"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.message.contains("unknown-widget"))
    );
    let html = fs::read_to_string(root.join("dist/legacy/index.html")).unwrap();
    assert!(!html.contains("<script>alert"));
    assert!(!html.contains("onclick="));
    assert!(html.contains("Retained safe text."));
    assert!(html.contains("<h2"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_sds-site-generator"))
        .args(["check", "--strict"])
        .arg(root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("warnings"));
    let diagnostics: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("dist/_site/diagnostics.json")).unwrap())
            .unwrap();
    assert_eq!(diagnostics.as_array().unwrap().len(), report.warnings.len());
}

#[test]
fn svg_is_sanitized_and_tiff_images_get_browser_readable_aliases() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("public/diagram.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="8" onload="alert(1)"><script>alert(1)</script><rect width="12" height="8" fill="url(#safe) url(//external.example/image)"/><path fill="url(#safe)"/></svg>"##).unwrap();
    image::RgbImage::new(2, 3)
        .save(root.join("public/scan.tif"))
        .unwrap();
    fs::write(root.join("content/images.md"), "# Images\n\nImported diagrams.\n\n![A diagram](/diagram.svg)\n\n![A scanned diagram](/scan.tif)\n\n[Original TIFF](/scan.tif)\n").unwrap();
    let report = site_generator::build(root).unwrap();
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.message.contains("SVG"))
    );
    let svg = fs::read_to_string(root.join("dist/diagram.svg")).unwrap();
    assert!(!svg.contains("<script"));
    assert!(!svg.contains("onload"));
    assert!(!svg.contains("external.example"));
    assert!(svg.contains("url(#safe)"));
    let html = fs::read_to_string(root.join("dist/images/index.html")).unwrap();
    assert!(html.contains("src=\"/docs/scan.tif.png\""));
    assert!(html.contains("href=\"/docs/scan.tif\""));
    assert!(html.contains("width=\"2\"") && html.contains("height=\"3\""));
    assert!(root.join("dist/scan.tif").is_file());
    assert!(root.join("dist/scan.tif.png").is_file());
}

#[test]
fn yaml_can_hide_a_page_from_navigation_without_unpublishing_it() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/hidden.md"), "---\ntitle: Hidden page\nnavigation: false\ntags: [legacy, documentation]\n---\n# Hidden page\n\nStill published and searchable.\n").unwrap();
    site_generator::build(root).unwrap();
    let html = fs::read_to_string(root.join("dist/index.html")).unwrap();
    assert!(!html.contains(">Hidden page</a>"));
    assert!(root.join("dist/hidden/index.html").is_file());
    assert!(
        fs::read_to_string(root.join("dist/sitemap.xml"))
            .unwrap()
            .contains("/hidden/")
    );
    fs::create_dir(root.join("content/hidden")).unwrap();
    fs::write(
        root.join("content/hidden/child.md"),
        "# Visible child\n\nA visible child page.",
    )
    .unwrap();
    site_generator::build(root).unwrap();
    let html = fs::read_to_string(root.join("dist/index.html")).unwrap();
    assert!(!html.contains("href=\"/docs/hidden/\""));
    assert!(html.contains("href=\"/docs/hidden/child/\""));
}

#[test]
fn compatibility_never_overwrites_duplicate_routes_or_precompressed_attachments() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/1.same.md"), "# First\n\nFirst page.").unwrap();
    fs::write(root.join("content/2.same.md"), "# Second\n\nSecond page.").unwrap();
    assert!(format!("{:#}", site_generator::build(root).err().unwrap()).contains("duplicate URL"));
    fs::remove_file(root.join("content/2.same.md")).unwrap();
    fs::write(root.join("public/notes.txt"), "An attachment.").unwrap();
    fs::write(
        root.join("public/notes.txt.gz"),
        "Do not replace an authored file.",
    )
    .unwrap();
    assert!(
        format!("{:#}", site_generator::build(root).err().unwrap())
            .contains("Precompressed output")
    );
    assert!(!root.join("dist").exists());
}

#[test]
#[ignore = "Requires SITE_GENERATOR_COMPAT_SOURCE pointing to the existing Nuxt project"]
fn original_secure_coding_content_builds_after_explicit_duplicate_filename_fix() {
    let source =
        std::env::var_os("SITE_GENERATOR_COMPAT_SOURCE").expect("Set SITE_GENERATOR_COMPAT_SOURCE");
    let source = Path::new(&source);
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("site.toml"), "name = 'CERT Secure Coding'\nurl = 'https://cmu-sei.github.io/secure-coding-standards/'\ncompatibility = true\n").unwrap();
    copy_tree(&source.join("content"), &root.join("content"));
    copy_tree(&source.join("public"), &root.join("public"));
    let error = site_generator::build(root)
        .err()
        .expect("Duplicate URLs must remain fatal in compatibility mode");
    assert!(format!("{error:#}").contains("duplicate URL"));
    fs::rename(
        root.join("content/4.sei-cert-c-coding-standard/04.back-matter/7.ff-related-guidelines/10.2013.md"),
        root.join("content/4.sei-cert-c-coding-standard/04.back-matter/7.ff-related-guidelines/10.2013-alternate.md"),
    ).unwrap();
    fs::rename(
        root.join("content/6.sei-cert-oracle-coding-standard-for-java/6.admin/6.todo-list.md"),
        root.join(
            "content/6.sei-cert-oracle-coding-standard-for-java/6.admin/6.todo-list-alternate.md",
        ),
    )
    .unwrap();
    let report = site_generator::build(root).unwrap();
    assert_eq!(report.pages, 1265);
    assert!(!report.warnings.is_empty());
    assert!(root.join("dist/_site/diagnostics.json").is_file());
}
