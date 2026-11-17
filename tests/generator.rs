use kuchiki::traits::TendrilSink;
use site_generator::{build, check, create};
use std::fs;
use tempfile::{TempDir, tempdir};

fn fixture() -> (TempDir, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let root = temp.path().join("website");
    create(&root, "Example <docs>", "https://example.org/docs/").unwrap();
    (temp, root)
}

#[test]
fn starter_builds_complete_portable_site() {
    let (_temp, root) = fixture();
    let report = build(&root).unwrap();
    assert_eq!(report.pages, 3);
    let home = fs::read_to_string(root.join("dist/index.html")).unwrap();
    assert!(home.contains("<title>Welcome | Example &lt;docs&gt;</title>"));
    assert!(home.contains("https://example.org/docs/"));
    assert!(home.contains("href=\"/docs/guides/\""));
    assert!(home.contains("id=\"start-here\""));
    assert!(home.contains("Skip to main content"));
    assert!(!home.contains("class=\"sds-breadcrumb\""));
    let publishing = fs::read_to_string(root.join("dist/guides/publishing/index.html")).unwrap();
    assert!(!publishing.contains("class=\"sds-breadcrumb\""));
    assert!(!publishing.contains("aria-label=\"Breadcrumb\""));
    for (route, has_sidebar) in [
        ("index.html", false),
        ("guides/index.html", true),
        ("guides/publishing/index.html", true),
        ("search/index.html", false),
        ("404.html", false),
    ] {
        let html = fs::read_to_string(root.join("dist").join(route)).unwrap();
        let document = kuchiki::parse_html().one(html);
        assert_eq!(
            document
                .select_first("#site-sidebar nav[aria-label='Documentation']")
                .is_ok(),
            has_sidebar,
            "Section navigation must match the current section in {route}"
        );
        assert!(
            document
                .select_first(".sds-app-header [popovertarget='site-sidebar']")
                .is_ok(),
            "Mobile navigation must be available in {route}"
        );
        assert!(
            document
                .select_first("#site-sidebar nav[aria-label='Documentation sections']")
                .is_ok(),
            "Mobile drawer must contain the primary navigation in {route}"
        );
        assert!(
            document
                .select_first("#site-search-dialog input[autofocus]")
                .is_ok()
        );
        assert!(document.select_first("#site-query[autofocus]").is_err());
        assert!(
            document.select_first("#site-sidebar footer").is_err(),
            "Unexpected sidebar footer in {route}"
        );
        assert!(
            document
                .select_first(".sds-app-masthead [data-search-open]")
                .is_ok(),
            "Missing masthead search control in {route}"
        );
        assert!(
            document
                .select_first("[data-sds-action='toggle-fullscreen']")
                .is_err(),
            "Unexpected fullscreen toggle in {route}"
        );
        let icons: serde_json::Value =
            serde_json::from_str(include_str!("../vendor/sds-lite/documentation-icons.json"))
                .unwrap();
        for (selector, name) in [
            (".sds-app-masthead [data-search-open] svg", "search"),
            (
                ".sds-app-masthead [data-sds-action='toggle-color-scheme'] svg:first-child",
                "dark-mode",
            ),
            (
                ".sds-app-masthead [data-sds-action='toggle-color-scheme'] svg:last-child",
                "light-mode",
            ),
            (".sds-app-header [popovertarget='site-sidebar'] svg", "menu"),
        ] {
            let expected = kuchiki::parse_html().one(icons[name].as_str().unwrap());
            assert_eq!(
                document
                    .select_first(selector)
                    .unwrap()
                    .as_node()
                    .to_string(),
                expected.select_first("svg").unwrap().as_node().to_string(),
                "Incorrect shared {name} icon in {route}"
            );
        }
        if let Ok(icon) = document.select_first(".sds-app-masthead a[aria-label='View source'] svg")
        {
            let expected = kuchiki::parse_html().one(icons["github"].as_str().unwrap());
            assert_eq!(
                icon.as_node().to_string(),
                expected.select_first("svg").unwrap().as_node().to_string()
            );
        }
        assert!(
            document
                .select_first("#site-main article.sds-prose[data-sds-tone='info']")
                .is_ok(),
            "Missing info-toned SDS prose in {route}"
        );
    }
    for class in [
        "sds-app-masthead",
        "sds-app-navigation",
        "sds-sei-wordmark",
        "sds-app-footer-top",
        "sds-app-footer-content",
        "sds-app-footer-brand",
        "sds-app-footer-middle",
        "sds-app-footer-legal",
    ] {
        assert!(
            home.contains(class),
            "Missing documentation shell region {class}"
        );
    }
    assert!(home.contains("id=\"site-search-dialog\""));
    assert!(home.contains("<h1 id=\"welcome\">Welcome</h1>"));
    assert!(!home.contains("sds-text-h3\" id=\"start-here"));
    let publishing_document = kuchiki::parse_html().one(publishing);
    let guides_document =
        kuchiki::parse_html().one(fs::read_to_string(root.join("dist/guides/index.html")).unwrap());
    let sidebar = guides_document
        .select_first("#site-sidebar nav[aria-label='Documentation']")
        .unwrap();
    assert!(!sidebar.as_node().text_contents().contains("Welcome"));
    assert!(sidebar.as_node().text_contents().contains("Publishing"));
    assert_eq!(
        guides_document
            .select("#site-sidebar summary a")
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        sidebar
            .as_node()
            .select("a[href='/docs/guides/']")
            .unwrap()
            .count(),
        0
    );
    assert!(
        guides_document
            .select_first("#site-sidebar header a[href='/docs/guides/']")
            .is_ok()
    );
    assert!(
        publishing_document
            .select_first(".sds-app-pagination a[rel='prev'][href='/docs/guides/']")
            .is_ok()
    );
    assert!(!home.contains("_site/site.css") && !home.contains("_site/components.js"));
    assert!(!root.join("dist/_site/site.css").exists());
    assert!(!root.join("dist/_site/components.js").exists());
    assert!(!home.contains("cdn.jsdelivr"));
    for file in [
        "search/index.html",
        "guides/publishing/index.html",
        "404.html",
        "sitemap.xml",
        "robots.txt",
        ".nojekyll",
        "_site/search/manifest.json",
        "_site/search/docs-0.json",
        "_site/search/terms-77.json",
        "_site/search-worker.js",
        "_site/sds-lite/documentation.js",
        "_site/sds-lite/documentation.css",
        "_site/sds-lite/package/assets/sei-wordmark.svg",
        "_site/sds-lite/LICENSE.txt",
    ] {
        assert!(root.join("dist").join(file).is_file(), "{file}");
    }
}

#[test]
fn documentation_toc_lists_only_sections_and_section_landing_links_are_not_duplicated() {
    let (_temp, root) = fixture();
    fs::write(root.join("content/guides/nested.md"), "# Nested\n\nNested sections.\n\n## First\n\nText.\n\n### Child\n\nText.\n\n#### Detail\n\nText.\n\n##### More detail\n\nText.\n\n###### Finest detail\n\nText.\n\n## Second\n\nText.").unwrap();
    fs::create_dir(root.join("content/guides/nested")).unwrap();
    fs::write(
        root.join("content/guides/nested/child.md"),
        "# Child page\n\nA nested child route.",
    )
    .unwrap();
    fs::write(
        root.join("content/about.md"),
        "# About\n\nA standalone page.",
    )
    .unwrap();
    build(&root).unwrap();
    let document = kuchiki::parse_html()
        .one(fs::read_to_string(root.join("dist/guides/nested/index.html")).unwrap());
    let toc = document.select_first("main .sds-app-toc").unwrap();
    assert_eq!(
        toc.as_node()
            .select("a")
            .unwrap()
            .map(|link| link.attributes.borrow().get("href").unwrap().to_owned())
            .collect::<Vec<_>>(),
        ["#first", "#second"]
    );
    assert_eq!(toc.as_node().select("ul ul").unwrap().count(), 0);
    for selector in [
        "h3#child",
        "h4#detail",
        "h5#more-detail",
        "h6#finest-detail",
    ] {
        assert!(document.select_first(selector).is_ok(), "{selector}");
    }
    assert!(
        document
            .select_first(".sds-app-pagination a[rel='prev']")
            .is_ok()
    );
    let home = kuchiki::parse_html().one(fs::read_to_string(root.join("dist/index.html")).unwrap());
    assert!(
        home.select_first("#site-sidebar nav[aria-label='Documentation']")
            .is_err()
    );
    let about =
        kuchiki::parse_html().one(fs::read_to_string(root.join("dist/about/index.html")).unwrap());
    assert!(
        about
            .select_first("#site-sidebar nav[aria-label='Documentation']")
            .is_err()
    );
    assert!(
        document
            .select_first("#site-sidebar li > a + details > summary")
            .is_ok()
    );
    assert_eq!(
        document.select("#site-sidebar summary a").unwrap().count(),
        0
    );
    assert_eq!(
        document
            .select("#site-sidebar header a[href='/docs/guides/']")
            .unwrap()
            .count(),
        1
    );
    let leaf = kuchiki::parse_html()
        .one(fs::read_to_string(root.join("dist/guides/nested/child/index.html")).unwrap());
    let sidebar_links = |document: &kuchiki::NodeRef| {
        document
            .select("#site-sidebar nav[aria-label='Documentation'] a")
            .unwrap()
            .map(|link| link.attributes.borrow().get("href").unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert!(!sidebar_links(&leaf).is_empty());
    assert_eq!(sidebar_links(&document), sidebar_links(&leaf));
    assert!(
        leaf.select_first(
            "#site-sidebar details[open] a[aria-current='page'][href='/docs/guides/nested/child/']"
        )
        .is_ok()
    );
}

#[test]
fn source_repository_is_optional_and_validated() {
    let (_temp, root) = fixture();
    let config = fs::read_to_string(root.join("site.toml")).unwrap();
    for repository in ["javascript:alert(1)", "https://user:pass@example.org/"] {
        fs::write(
            root.join("site.toml"),
            format!("{config}\nrepository = '{repository}'\n"),
        )
        .unwrap();
        assert!(check(&root).is_err());
    }
    fs::write(
        root.join("site.toml"),
        format!("{config}\nrepository = 'https://github.com/example/docs'\n"),
    )
    .unwrap();
    build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/index.html")).unwrap();
    assert!(html.contains("href=\"https://github.com/example/docs\" aria-label=\"View source\""));
}

#[test]
fn new_never_overwrites_a_directory() {
    let (_temp, root) = fixture();
    assert!(create(&root, "Replacement", "https://example.org").is_err());
    assert!(root.join("content/index.md").is_file());
}

#[test]
fn configured_primary_navigation_replaces_the_automatic_menu() {
    let (_temp, root) = fixture();
    let config = fs::read_to_string(root.join("site.toml")).unwrap();
    fs::write(
        root.join("site.toml"),
        format!(
            "{config}\n[[navigation]]\ntitle = 'Start'\npath = '/'\n\n[[navigation]]\ntitle = 'Learn & publish'\npath = '/guides'\n\n[[navigation.children]]\ntitle = 'Publishing'\npath = '/guides/publishing/index.html'\n\n[[navigation.children]]\ntitle = 'Reference'\nchildren = [{{ title = 'Headings', path = '/guides/#headings' }}]\n\n[[navigation]]\ntitle = 'External'\npath = 'https://example.net/?a=1&b=2'\n"
        ),
    )
    .unwrap();
    build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/guides/publishing/index.html")).unwrap();
    let document = kuchiki::parse_html().one(html);
    for primary in document
        .select("nav[aria-label='Documentation sections']")
        .unwrap()
    {
        let markup = primary.as_node().to_string();
        assert!(markup.contains(">Start</a>"), "{markup}");
        assert!(markup.contains("<summary"), "{markup}");
        let mobile = primary.as_node().ancestors().any(|node| {
            node.as_element().is_some_and(|element| {
                element.attributes.borrow().get("id") == Some("site-sidebar")
            })
        });
        if mobile {
            assert!(markup.contains("sds-navigation-panel"), "{markup}");
            assert!(
                markup.contains("data-sds-action=\"navigation-back\""),
                "{markup}"
            );
            assert!(!markup.contains("<sds-dropdown"), "{markup}");
            assert!(!markup.contains("popover="), "{markup}");
            assert!(!markup.contains("site-primary-overview"), "{markup}");
        } else {
            assert!(markup.contains("sds-disclosure"), "{markup}");
            assert!(markup.contains("<sds-dropdown"), "{markup}");
            assert!(markup.contains("href=\"/docs/guides/\""), "{markup}");
        }
        for dropdown in primary.as_node().select("sds-dropdown").unwrap() {
            let trigger = dropdown.as_node().select_first("button").unwrap();
            let target = trigger
                .attributes
                .borrow()
                .get("popovertarget")
                .unwrap()
                .to_string();
            assert_eq!(
                document
                    .select(&format!("[id='{target}']"))
                    .unwrap()
                    .count(),
                1
            );
            assert!(
                dropdown
                    .as_node()
                    .select_first(&format!(
                        "menu.sds-dropdown-menu[id='{target}'][role='menu'][popover='auto']"
                    ))
                    .is_ok()
            );
        }
        assert!(markup.contains("Learn &amp; publish"), "{markup}");
        assert!(
            markup.contains("href=\"/docs/guides/#headings\""),
            "{markup}"
        );
        assert!(
            markup.contains("href=\"https://example.net/?a=1&amp;b=2\""),
            "{markup}"
        );
        assert_eq!(
            primary
                .as_node()
                .select("a[href='/docs/guides/publishing/'][aria-current='page']")
                .unwrap()
                .count(),
            1
        );
        assert!(!markup.contains(">Home</a>"), "{markup}");
    }
}

#[test]
fn configured_navigation_rejects_invalid_entries_and_targets() {
    for navigation in [
        "title = ''\npath = '/'",
        "title = 'Empty group'",
        "title = 'Unsafe'\npath = 'javascript:alert(1)'",
        "title = 'Protocol relative'\npath = '//example.net/'",
        "title = 'Credentials'\npath = 'https://user:pass@example.net/'",
        "title = 'Missing page'\npath = '/missing/'",
        "title = 'Missing heading'\npath = '/guides/#missing'",
        "title = 'Typo'\npath = '/'\nchidlren = []",
    ] {
        let (_temp, root) = fixture();
        let mut config: toml::Value =
            toml::from_str(&fs::read_to_string(root.join("site.toml")).unwrap()).unwrap();
        config["compatibility"] = toml::Value::Boolean(true);
        let config = toml::to_string(&config).unwrap();
        fs::write(
            root.join("site.toml"),
            format!("{config}\n[[navigation]]\n{navigation}\n"),
        )
        .unwrap();
        assert!(check(&root).is_err(), "Accepted {navigation}");
    }
}

#[test]
fn configured_empty_navigation_hides_only_the_primary_menu() {
    let (_temp, root) = fixture();
    let config = fs::read_to_string(root.join("site.toml")).unwrap();
    fs::write(
        root.join("site.toml"),
        format!("{config}\nnavigation = []\n"),
    )
    .unwrap();
    build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/guides/index.html")).unwrap();
    let document = kuchiki::parse_html().one(html);
    assert_eq!(
        document
            .select("nav[aria-label='Documentation sections'] a")
            .unwrap()
            .count(),
        0
    );
    assert!(
        document
            .select("nav[aria-label='Documentation'] a[href='/docs/guides/publishing/']")
            .unwrap()
            .next()
            .is_some()
    );
}

#[test]
fn checks_do_not_write_output_and_failed_builds_preserve_previous_output() {
    let (_temp, root) = fixture();
    check(&root).unwrap();
    assert!(!root.join("dist").exists());
    build(&root).unwrap();
    let before = fs::read(root.join("dist/index.html")).unwrap();
    fs::write(
        root.join("content/broken.md"),
        "# Broken\n\n[Missing](missing.md)",
    )
    .unwrap();
    let error = format!("{:#}", build(&root).err().unwrap());
    assert!(error.contains("content/broken.md:3"), "{error}");
    assert_eq!(before, fs::read(root.join("dist/index.html")).unwrap());
}

#[test]
fn refuses_to_replace_unknown_output() {
    let (_temp, root) = fixture();
    fs::create_dir(root.join("dist")).unwrap();
    fs::write(root.join("dist/important.txt"), "keep").unwrap();
    assert!(build(&root).is_err());
    assert_eq!(
        fs::read_to_string(root.join("dist/important.txt")).unwrap(),
        "keep"
    );
}

#[test]
fn metadata_drafts_numbered_paths_and_duplicate_headings() {
    let (_temp, root) = fixture();
    fs::write(root.join("content/02.extra.md"), "+++\ntitle = \"Extra\"\norder = -1\n+++\n# Extra page\n\nIntro.\n\n## Same\n\nOne.\n\n## Same\n\nTwo.\n\n[Second](#same-2)\n").unwrap();
    fs::write(
        root.join("content/draft.md"),
        "+++\ndraft = true\n+++\n# Draft\n\n[Unfinished](missing.md)",
    )
    .unwrap();
    build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/extra/index.html")).unwrap();
    assert!(html.contains("id=\"same-2\""));
    assert!(html.contains("href=\"/docs/extra/#same-2\""));
    assert!(html.contains("<title>Extra |"));
    assert!(!root.join("dist/draft").exists());
}

#[test]
fn invalid_content_is_rejected_with_actionable_errors() {
    for (content, expected) in [
        ("# Test\n\n<script>alert(1)</script>", "unsafe <script>"),
        ("# Test\n\n[Bad](javascript:alert)", "unsafe href"),
        ("# Test\n\n[Bad](//example.org)", "https://"),
        ("# Test\n\n[Bad](#absent)", "heading #absent"),
        ("# Test\n\n### Skipped\n\nText.", "skips a level"),
        ("# Test\n\n# Another\n\nText.", "one main heading"),
        (
            "+++\ntitel = \"Typo\"\n+++\n# Test\n\nText.",
            "Invalid TOML",
        ),
        (
            "---\ntitle: [Unclosed\n---\n# Test\n\nText.",
            "Invalid YAML",
        ),
        (
            "# Test\n\nAn external image.\n\n![Remote](https://example.org/x.png)",
            "local files",
        ),
        ("# Test", "introductory paragraph"),
    ] {
        let (_temp, root) = fixture();
        fs::write(root.join("content/test.md"), content).unwrap();
        let error = format!("{:#}", check(&root).err().unwrap());
        assert!(error.contains(expected), "{expected}: {error}");
    }
}

#[test]
fn duplicate_routes_are_rejected() {
    let (_temp, root) = fixture();
    fs::write(root.join("content/1.same.md"), "# First\n\nText.").unwrap();
    fs::write(root.join("content/2.same.md"), "# Second\n\nText.").unwrap();
    assert!(format!("{:#}", check(&root).err().unwrap()).contains("duplicate URL"));
}

#[test]
fn public_assets_are_checked_and_images_have_dimensions() {
    let (_temp, root) = fixture();
    fs::write(
        root.join("public/pixel.png"),
        [
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0,
            5, 0, 1, 13, 10, 45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ],
    )
    .unwrap();
    fs::write(
        root.join("content/image.md"),
        "# Image\n\nA small illustration.\n\n![One pixel](/pixel.png)\n\n[Download](/pixel.png)",
    )
    .unwrap();
    build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/image/index.html")).unwrap();
    assert!(html.contains("width=\"1\"") && html.contains("height=\"1\""));
    assert!(html.contains("src=\"/docs/pixel.png\"") && html.contains("alt=\"One pixel\""));
    fs::write(
        root.join("content/image.md"),
        "# Image\n\nAn image.\n\n![](/pixel.png)",
    )
    .unwrap();
    assert!(
        format!("{:#}", check(&root).err().unwrap())
            .to_lowercase()
            .contains("describe this image")
    );
    fs::write(
        root.join("content/image.md"),
        "# Image\n\nAn image.\n\n![](/pixel.png \"decorative\")",
    )
    .unwrap();
    check(&root).unwrap();
}

#[test]
fn encoded_fragments_and_relative_links_work() {
    let (_temp, root) = fixture();
    fs::write(root.join("content/unicode.md"), "# Café\n\nIntroduction.\n\n## Résumé\n\n[Section](#r%C3%A9sum%C3%A9)\n\n[Guide](guides/index.md#headings)").unwrap();
    check(&root).unwrap();
}

#[test]
fn configuration_typos_and_invalid_urls_are_rejected() {
    for config in [
        "name = 'Docs'\nurl = 'javascript:alert(1)'",
        "name = 'Docs'\nurl = 'https://example.org/?bad=1'",
        "name = 'Docs'\nurl = 'https://user:pass@example.org/'",
        "name = 'Docs'\nurl = 'https://example.org/'\nlangauge = 'en'",
    ] {
        let (_temp, root) = fixture();
        fs::write(root.join("site.toml"), config).unwrap();
        assert!(check(&root).is_err());
    }
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_rejected() {
    use std::os::unix::fs::symlink;
    let (_temp, root) = fixture();
    symlink(root.join("site.toml"), root.join("public/config.txt")).unwrap();
    assert!(format!("{:#}", check(&root).err().unwrap()).contains("symbolic links"));
}

#[test]
fn generated_text_has_valid_precompressed_variants() {
    use std::io::Read;
    let (_temp, root) = fixture();
    build(&root).unwrap();
    for file in [
        "index.html",
        "_site/sds-lite/sds.css",
        "_site/search/manifest.json",
    ] {
        let bytes = fs::read(root.join("dist").join(format!("{file}.gz"))).unwrap();
        let mut decoder = flate2::read::GzDecoder::new(bytes.as_slice());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, fs::read(root.join("dist").join(file)).unwrap());
    }
}

#[test]
fn authored_search_links_are_valid_and_reserved_heading_ids_are_rejected() {
    let (_temp, root) = fixture();
    fs::write(
        root.join("content/search-help.md"),
        "# Find content\n\n[Search](/search/?q=writing)",
    )
    .unwrap();
    build(&root).unwrap();
    let html = fs::read_to_string(root.join("dist/search-help/index.html")).unwrap();
    assert!(html.contains("href=\"/docs/search/?q=writing\""));
    fs::write(root.join("content/search-help.md"), "# Site main\n\nText.").unwrap();
    assert!(format!("{:#}", check(&root).err().unwrap()).contains("reserved identifier"));
}

#[test]
fn search_indexes_cross_metadata_chunks_without_losing_document_ids() {
    let (_temp, root) = fixture();
    for index in 0..70 {
        let body = if index == 69 {
            "uniqueword"
        } else {
            "Ordinary content."
        };
        fs::write(
            root.join(format!("content/page-{index:02}.md")),
            format!("# Page {index}\n\n{body}"),
        )
        .unwrap();
    }
    build(&root).unwrap();
    let output = root.join("dist/_site/search");
    let bucket: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("terms-75.json")).unwrap()).unwrap();
    let (id, _) = bucket["uniqueword"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    let id: usize = id.parse().unwrap();
    assert!(id >= 64);
    let documents: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join(format!("docs-{}.json", id / 64))).unwrap())
            .unwrap();
    assert_eq!(documents[id % 64]["page"], "Page 69");
    assert!(
        documents[id % 64]["url"]
            .as_str()
            .unwrap()
            .starts_with("/docs/page-69/")
    );
}
