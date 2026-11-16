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
fn callouts_render_shared_recipes_and_preserve_nested_code_and_fenced_literals() {
    let temp = compatible_fixture();
    let root = temp.path();
    fs::write(root.join("content/callouts.md"), "# Callouts\n\nSupported imported components.\n\n::callout{type='outline' variant='orange' size='sm'}\n**Warning:** [Home](index.md).\n\n::code-block{quality='good'}\n```text\n::callout{variant=\"red\"}\nLiteral example.\n::\n```\n::\n\n::callout{variant='green' type='bold'}\nNested note.\n::\n\nAfter nested components.\n::\n\nAfter the callout.\n").unwrap();
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
    assert!(html.contains("::callout{variant="));
    assert_eq!(document.select("aside.sds-callout").unwrap().count(), 2);
    assert!(!warning.text_contents().contains("After the callout."));
}

#[test]
fn callouts_support_all_reference_colors_types_and_sizes() {
    let temp = compatible_fixture();
    let mut source = String::from("# Callout recipes\n\nSupported shared recipes.\n\n");
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
    for (color, _) in colors {
        for variant in ["subtle", "outline", "bold"] {
            for size in ["xs", "sm", "md", "lg"] {
                source.push_str(&format!("::callout{{variant=\"{color}\" type='{variant}' size='{size}'}}\n**{color}** note.\n::\n\n"));
            }
        }
    }
    fs::write(temp.path().join("content/recipes.md"), source).unwrap();
    let report = site_generator::build(temp.path()).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(temp.path().join("dist/recipes/index.html")).unwrap());
    assert_eq!(
        document
            .select("aside.sds-callout[role='note']")
            .unwrap()
            .count(),
        96
    );
    for (color, tone) in colors {
        assert_eq!(
            document
                .select(&format!(
                    "aside[data-sds-callout-color='{color}'][data-sds-tone='{tone}']"
                ))
                .unwrap()
                .count(),
            12
        );
    }
}

#[test]
fn callouts_reject_unknown_properties_and_invalid_recipes() {
    for properties in [
        "onclick='alert(1)'",
        "style='color:red'",
        "variant='unknown'",
        "type='unknown'",
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
    assert!(html.contains("id=\"version-details\""));
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
    assert!(!html.contains("_site/site.css"));
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
