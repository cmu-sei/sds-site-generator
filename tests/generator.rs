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

fn abandoned_workspace(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = root.join(format!(".site-generator-build-{name}"));
    fs::create_dir(&path).unwrap();
    fs::write(
        path.join(".site-generator-workspace"),
        "site-generator workspace v1\n",
    )
    .unwrap();
    path
}

#[test]
fn build_cleans_owned_abandoned_workspaces_and_leaves_unknown_directories_alone() {
    let (_temp, root) = fixture();
    let owned = abandoned_workspace(&root, "abandoned");
    fs::create_dir(owned.join("output")).unwrap();
    fs::write(owned.join("output/partial.html"), "partial build").unwrap();
    for name in [
        ".tmp-authored",
        ".site-generator-build-unmarked",
        ".site-generator-build-foreign",
    ] {
        fs::create_dir(root.join(name)).unwrap();
        fs::write(root.join(name).join("important.txt"), "keep").unwrap();
    }
    fs::write(
        root.join(".site-generator-build-foreign/.site-generator-workspace"),
        b"\xff",
    )
    .unwrap();

    let report = build(&root).unwrap();

    assert!(report.warnings.is_empty());
    assert!(!owned.exists());
    for name in [
        ".tmp-authored",
        ".site-generator-build-unmarked",
        ".site-generator-build-foreign",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(name).join("important.txt")).unwrap(),
            "keep"
        );
    }
    let mut workspaces: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().starts_with(".site-generator-build-"))
        .collect();
    workspaces.sort();
    assert_eq!(
        workspaces,
        [
            ".site-generator-build-foreign",
            ".site-generator-build-unmarked"
        ]
    );
}

#[test]
fn build_preserves_recovery_until_a_complete_published_build_exists() {
    for incomplete in [false, true] {
        let (_temp, root) = fixture();
        build(&root).unwrap();
        let before = fs::read(root.join("dist/index.html")).unwrap();
        let workspace = abandoned_workspace(&root, "recovery");
        let previous = workspace.join("previous");
        fs::rename(root.join("dist"), &previous).unwrap();
        if incomplete {
            fs::create_dir(root.join("dist")).unwrap();
            fs::write(
                root.join("dist/.site-generator-output"),
                "site-generator v1\n",
            )
            .unwrap();
        }
        fs::write(
            root.join("content/broken.md"),
            "# Broken\n\n[Missing](missing.md)",
        )
        .unwrap();
        let error = format!("{:#}", build(&root).err().unwrap());
        assert!(error.contains("content/broken.md"), "{error}");
        assert!(
            error.contains(&previous.canonicalize().unwrap().display().to_string()),
            "{error}"
        );
        assert_eq!(before, fs::read(previous.join("index.html")).unwrap());
        fs::remove_file(root.join("content/broken.md")).unwrap();

        let report = build(&root).unwrap();

        assert_eq!(before, fs::read(previous.join("index.html")).unwrap());
        assert!(report.warnings.iter().any(|warning| {
            warning
                .message
                .contains(&previous.canonicalize().unwrap().display().to_string())
        }));
        assert!(root.join("dist/index.html").is_file());
        let report = build(&root).unwrap();
        assert!(!workspace.exists());
        assert!(report.warnings.is_empty());
    }
}

#[test]
fn failed_build_never_discards_recovery_based_on_existing_output_markers() {
    let (_temp, root) = fixture();
    build(&root).unwrap();
    let before = fs::read(root.join("dist/index.html")).unwrap();
    let workspace = abandoned_workspace(&root, "incomplete-live-output");
    let previous = workspace.join("previous");
    fs::rename(root.join("dist"), &previous).unwrap();
    build(&root).unwrap();
    fs::remove_file(root.join("dist/index.html")).unwrap();
    fs::write(
        root.join("content/broken.md"),
        "# Broken\n\n[Missing](missing.md)",
    )
    .unwrap();

    let error = format!("{:#}", build(&root).err().unwrap());

    assert!(error.contains("content/broken.md"), "{error}");
    assert!(
        error.contains(&previous.canonicalize().unwrap().display().to_string()),
        "{error}"
    );
    assert_eq!(fs::read(previous.join("index.html")).unwrap(), before);
    fs::remove_file(root.join("content/broken.md")).unwrap();
    build(&root).unwrap();
    assert_eq!(fs::read(root.join("dist/index.html")).unwrap(), before);
    assert!(!workspace.exists());
}

#[test]
fn build_preserves_unrecognized_backup_files_even_with_valid_published_output() {
    let (_temp, root) = fixture();
    build(&root).unwrap();
    let workspace = abandoned_workspace(&root, "unknown-backup");
    fs::create_dir(workspace.join("previous")).unwrap();
    let important = workspace.join("previous/important.txt");
    fs::write(&important, "keep").unwrap();
    fs::write(workspace.join("previous/.site-generator-output"), b"\xff").unwrap();

    let report = build(&root).unwrap();

    assert_eq!(fs::read_to_string(important).unwrap(), "keep");
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.message.contains("backup is unrecognized"))
    );
}

#[test]
fn build_waits_for_the_site_lock_before_cleanup_or_publication() {
    use std::sync::mpsc::{RecvTimeoutError, channel};
    use std::time::Duration;

    let (_temp, root) = fixture();
    build(&root).unwrap();
    let before = fs::read(root.join("dist/index.html")).unwrap();
    let workspace = abandoned_workspace(&root, "waiting");
    fs::write(
        root.join("content/revision.md"),
        "# New revision\n\nUpdated documentation.",
    )
    .unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".site-generator.lock"))
        .unwrap();
    lock.lock().unwrap();
    let (sender, receiver) = channel();
    let site = root.clone();
    let thread = std::thread::spawn(move || sender.send(build(&site)).unwrap());
    let early = receiver.recv_timeout(Duration::from_millis(300));
    let waited = matches!(&early, Err(RecvTimeoutError::Timeout));
    let retained_workspace = workspace.exists();
    let during = fs::read(root.join("dist/index.html")).unwrap();
    drop(lock);
    let result = match early {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => receiver.recv_timeout(Duration::from_secs(30)).unwrap(),
        Err(error) => panic!("Build worker failed: {error}"),
    };
    thread.join().unwrap();
    result.unwrap();

    assert!(
        waited,
        "A build must not finish while another build holds the lock"
    );
    assert!(retained_workspace, "Cleanup must wait for the active build");
    assert_eq!(before, during);
    assert!(!workspace.exists());
    assert!(root.join("dist/revision/index.html").is_file());
}

#[cfg(unix)]
#[test]
fn build_cleanup_does_not_follow_workspace_marker_or_backup_symlinks() {
    use std::os::unix::fs::symlink;

    let (temp, root) = fixture();
    build(&root).unwrap();
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(
        outside.join(".site-generator-workspace"),
        "site-generator workspace v1\n",
    )
    .unwrap();
    fs::write(outside.join("important.txt"), "keep").unwrap();
    let link = root.join(".site-generator-build-linked");
    symlink(&outside, &link).unwrap();
    let marker_link = root.join(".site-generator-build-linked-marker");
    fs::create_dir(&marker_link).unwrap();
    symlink(
        outside.join(".site-generator-workspace"),
        marker_link.join(".site-generator-workspace"),
    )
    .unwrap();
    let backup_link = abandoned_workspace(&root, "linked-backup");
    symlink(&outside, backup_link.join("previous")).unwrap();

    let report = build(&root).unwrap();

    assert_eq!(
        fs::read_to_string(outside.join("important.txt")).unwrap(),
        "keep"
    );
    assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
    assert!(marker_link.exists());
    assert!(backup_link.exists());
    assert!(!report.warnings.is_empty());
    fs::remove_file(root.join(".site-generator.lock")).unwrap();
    symlink(
        outside.join("important.txt"),
        root.join(".site-generator.lock"),
    )
    .unwrap();
    let error = format!("{:#}", build(&root).err().unwrap());
    assert!(error.contains("Build lock"), "{error}");
    assert_eq!(
        fs::read_to_string(outside.join("important.txt")).unwrap(),
        "keep"
    );
}

#[cfg(unix)]
#[test]
fn build_reports_cleanup_permission_failures_and_retries_later() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let (_temp, root) = fixture();
    if fs::metadata(&root).unwrap().uid() == 0 {
        eprintln!("Permission-denial test is not applicable to a root-owned fixture.");
        return;
    }
    build(&root).unwrap();
    let before = fs::read(root.join("dist/index.html")).unwrap();
    let workspace = abandoned_workspace(&root, "readonly");
    let staging = workspace.join("output");
    fs::create_dir(&staging).unwrap();
    fs::write(staging.join("partial.html"), "partial build").unwrap();
    fs::set_permissions(&staging, fs::Permissions::from_mode(0o500)).unwrap();

    let result = build(&root);
    fs::set_permissions(&staging, fs::Permissions::from_mode(0o700)).unwrap();

    let error = format!("{:#}", result.err().unwrap());
    assert!(error.contains("Clean generator workspace"), "{error}");
    assert!(error.contains(&workspace.display().to_string()), "{error}");
    assert!(workspace.join(".site-generator-workspace").is_file());
    assert_eq!(before, fs::read(root.join("dist/index.html")).unwrap());
    build(&root).unwrap();
    assert!(!workspace.exists());
}

#[test]
#[cfg(unix)]
fn recovery_cleanup_errors_report_that_fresh_output_was_published_and_retry() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let (_temp, root) = fixture();
    if fs::metadata(&root).unwrap().uid() == 0 {
        eprintln!("Permission-denial test is not applicable to a root-owned fixture.");
        return;
    }
    build(&root).unwrap();
    let workspace = abandoned_workspace(&root, "recovery-cleanup-error");
    fs::rename(root.join("dist"), workspace.join("previous")).unwrap();
    build(&root).unwrap();
    let staging = workspace.join("output");
    fs::create_dir(&staging).unwrap();
    fs::write(staging.join("partial.html"), "partial build").unwrap();
    fs::set_permissions(&staging, fs::Permissions::from_mode(0o500)).unwrap();
    fs::write(
        root.join("content/revision.md"),
        "# Revision\n\nFresh output must survive recovery cleanup errors.",
    )
    .unwrap();

    let result = build(&root);
    fs::set_permissions(&staging, fs::Permissions::from_mode(0o700)).unwrap();

    let error = format!("{:#}", result.err().unwrap());
    assert!(error.contains("build was published"), "{error}");
    assert!(error.contains(&workspace.display().to_string()), "{error}");
    assert!(workspace.join(".site-generator-workspace").is_file());
    assert!(root.join("dist/revision/index.html").is_file());
    build(&root).unwrap();
    assert!(!workspace.exists());
}

#[test]
fn highlighting_is_requested_only_by_pages_with_code_examples() {
    let (_temp, root) = fixture();
    fs::write(
        root.join("content/plain.md"),
        "# Plain\n\nNo code examples.\n",
    )
    .unwrap();
    fs::write(
        root.join("content/code.md"),
        "# Code\n\nA literal example.\n\n```c\nint answer = 42;\n```\n",
    )
    .unwrap();
    build(&root).unwrap();
    for route in ["plain/index.html", "search/index.html", "404.html"] {
        let html = fs::read_to_string(root.join("dist").join(route)).unwrap();
        assert!(
            !html.contains("src=\"/docs/_site/highlight.js\""),
            "{route}"
        );
    }
    let code = fs::read_to_string(root.join("dist/code/index.html")).unwrap();
    assert!(code.contains("src=\"/docs/_site/highlight.js\""));
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
    assert!(!home.contains("_site/components.js"));
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
    let icon = toc.as_node().select_first("strong > svg").unwrap();
    let attributes = icon.attributes.borrow();
    assert_eq!(attributes.get("aria-hidden"), Some("true"));
    assert_eq!(attributes.get("viewBox"), Some("0 0 24 24"));
    assert_eq!(attributes.get("fill"), Some("currentColor"));
    assert_eq!(
        icon.as_node()
            .select_first("path")
            .unwrap()
            .attributes
            .borrow()
            .get("d"),
        Some(
            "M9 19v-2h12v2zm0-6v-2h12v2zm0-6V5h12v2zM5 20q-.825 0-1.412-.587T3 18t.588-1.412T5 16t1.413.588T7 18t-.587 1.413T5 20m0-6q-.825 0-1.412-.587T3 12t.588-1.412T5 10t1.413.588T7 12t-.587 1.413T5 14M3.588 7.413Q3 6.825 3 6t.588-1.412T5 4t1.413.588T7 6t-.587 1.413T5 8t-1.412-.587"
        )
    );
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
    let workspace = abandoned_workspace(&root, "check-only");
    check(&root).unwrap();
    assert!(!root.join("dist").exists());
    assert!(!root.join(".site-generator.lock").exists());
    assert!(workspace.exists());
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
    assert!(fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".site-generator-build-")
    }));
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
fn nested_public_assets_validate_each_filename_component() {
    let (_temp, root) = fixture();
    fs::create_dir_all(root.join("public/downloads")).unwrap();
    fs::write(root.join("public/downloads/readme.txt"), "Download").unwrap();
    fs::write(
        root.join("content/index.md"),
        "# Home\n\n[Download](/downloads/readme.txt)",
    )
    .unwrap();
    build(&root).unwrap();
    assert_eq!(
        fs::read_to_string(root.join("dist/downloads/readme.txt")).unwrap(),
        "Download"
    );
    let html = fs::read_to_string(root.join("dist/index.html")).unwrap();
    assert!(html.contains("href=\"/docs/downloads/readme.txt\""));

    let mut unsafe_paths = vec!["downloads/.private/notes.txt", "downloads/read#me.txt"];
    if cfg!(unix) {
        unsafe_paths.push("downloads/read\\me.txt");
    }
    for relative in unsafe_paths {
        let path = root.join("public").join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "Unsafe").unwrap();
        let error = format!("{:#}", check(&root).err().unwrap());
        assert!(
            error.contains("unsafe asset filename"),
            "{relative}: {error}"
        );
        fs::remove_file(path).unwrap();
    }
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
