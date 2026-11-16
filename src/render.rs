use crate::content::{Page, Site, escape};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const SDS_CSS: &[u8] = include_bytes!("../vendor/sds-lite/sds.css");
const SDS_BRAND: &[u8] = include_bytes!("../vendor/sds-lite/brand.css");
const SDS_JS: &[u8] = include_bytes!("../vendor/sds-lite/auto.js");
const SDS_DOCUMENTATION_CSS: &[u8] = include_bytes!("../vendor/sds-lite/documentation.css");
const SDS_DOCUMENTATION_JS: &[u8] = include_bytes!("../vendor/sds-lite/documentation.js");
const SDS_LICENSE: &[u8] = include_bytes!("../vendor/sds-lite/LICENSE");
const SDS_WORDMARK: &[u8] = include_bytes!("../vendor/sds-lite/package/assets/sei-wordmark.svg");
const LIST_ORDERED: &str = r#"<svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 5h10"/><path d="M11 12h10"/><path d="M11 19h10"/><path d="M4 4h1v5"/><path d="M4 9h2"/><path d="M6.5 20H3.4c0-1 2.6-1.925 2.6-3.5a1.5 1.5 0 0 0-2.6-1.02"/></svg>"#;
const SHIELD: &str = r#"<svg aria-hidden="true" viewBox="0 0 16 16"><path d="M14.5625 2.625C15.125 2.875 15.5 3.40625 15.5 4C15.5 12 9.59375 16 7.96875 16C6.40625 16 0.5 12.0625 0.5 4C0.5 3.40625 0.84375 2.875 1.40625 2.625L7.40625 0.125C7.5625 0.0625 7.8125 0.03125 8 0.03125C8.15625 0.03125 8.40625 0.0625 8.5625 0.125L14.5625 2.625ZM11 6.25C11 6.03125 10.8125 5.5 10.25 5.5C10.0312 5.5 9.8125 5.5625 9.65625 5.75L7.1875 8.625L6.25 7.71875C6.125 7.5625 5.9375 7.5 5.75 7.5C5.3125 7.5 5 7.84375 5 8.25C5 8.46875 5.0625 8.65625 5.21875 8.78125L6.71875 10.2812C6.84375 10.4062 7.03125 10.5 7.25 10.5C7.3125 10.5 7.59375 10.5 7.8125 10.25L10.8125 6.75C10.9375 6.625 11 6.4375 11 6.25Z" fill="currentColor"></path></svg>"#;

pub(crate) fn write(site: &Site, output: &Path) -> Result<()> {
    let assets = output.join("_site");
    fs::create_dir_all(assets.join("sds-lite/package/assets"))?;
    for (name, bytes) in [
        ("sds-lite/sds.css", SDS_CSS),
        ("sds-lite/brand.css", SDS_BRAND),
        ("sds-lite/auto.js", SDS_JS),
        ("sds-lite/documentation.css", SDS_DOCUMENTATION_CSS),
        ("sds-lite/documentation.js", SDS_DOCUMENTATION_JS),
        (
            "sds-lite/documentation-icons.json",
            include_bytes!("../vendor/sds-lite/documentation-icons.json").as_slice(),
        ),
        ("sds-lite/LICENSE.txt", SDS_LICENSE),
        ("sds-lite/package/assets/sei-wordmark.svg", SDS_WORDMARK),
        (
            "highlight.js",
            include_bytes!("../assets/highlight.js").as_slice(),
        ),
        (
            "search.js",
            include_bytes!("../assets/search.js").as_slice(),
        ),
        (
            "search-worker.js",
            include_bytes!("../assets/search-worker.js").as_slice(),
        ),
        (
            "favicon.svg",
            include_bytes!("../assets/favicon.svg").as_slice(),
        ),
    ] {
        fs::write(assets.join(name), bytes)?;
    }
    fs::write(
        assets.join("code-controls-LICENSE.txt"),
        format!(
            "SDS documentation icons\n{}\n\nLucide icons\n{}\n\nShiki syntax highlighting\n{}",
            crate::markup::documentation_icon("_attribution"),
            include_str!("../vendor/lucide-LICENSE"),
            include_str!("../assets/highlight-LICENSE.txt")
        ),
    )?;
    for page in &site.pages {
        let destination = output.join(page.route.trim_matches('/'));
        fs::create_dir_all(&destination)?;
        let body = format!(
            "<article class=\"sds-prose\" data-sds-tone=\"info\">{}{}</article>",
            page.html,
            page_navigation(site, page)
        );
        fs::write(
            destination.join("index.html"),
            shell(
                site,
                &page.title,
                &page.description,
                &page.route,
                &body,
                Some(page),
                false,
            ),
        )?;
    }
    let search = format!(
        r#"<article class="sds-prose" data-sds-tone="info"><h1 class="sds-text-h2">Search</h1>
<p>Search page titles, headings, and content. Put a phrase in quotation marks for an exact match.</p>
{}
<noscript><p>Search requires JavaScript. You can still browse every page using the site navigation.</p></noscript>
</article>"#,
        search_form(site, "site", true)
    );
    fs::create_dir_all(output.join("search"))?;
    fs::write(
        output.join("search/index.html"),
        shell(
            site,
            "Search",
            if site.config.description.is_empty() {
                "Search this documentation website."
            } else {
                &site.config.description
            },
            "/search/",
            &search,
            None,
            true,
        ),
    )?;
    let not_found = format!(
        "<article class=\"sds-prose\" data-sds-tone=\"info\"><h1 class=\"sds-text-h2\">Page not found</h1><p>This address does not match a page on this website.</p><p><a href=\"{}\">Go to the home page</a> or <a href=\"{}\">search the website</a>.</p></article>",
        escape(&site.config.href("/")),
        escape(&site.config.href("/search/"))
    );
    fs::write(
        output.join("404.html"),
        shell(
            site,
            "Page not found",
            "The requested page could not be found.",
            "/404.html",
            &not_found,
            None,
            true,
        ),
    )?;
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in &site.pages {
        sitemap.push_str(&format!(
            "<url><loc>{}</loc></url>\n",
            escape(&site.config.canonical(&page.route))
        ));
    }
    sitemap.push_str("</urlset>\n");
    fs::write(output.join("sitemap.xml"), sitemap)?;
    fs::write(
        output.join("robots.txt"),
        format!(
            "User-agent: *\nAllow: {}\nSitemap: {}\n",
            site.config.url.path(),
            site.config.canonical("/sitemap.xml")
        ),
    )?;
    fs::write(output.join(".nojekyll"), "")?;
    Ok(())
}

fn shell(
    site: &Site,
    title: &str,
    description: &str,
    route: &str,
    body: &str,
    page: Option<&Page>,
    noindex: bool,
) -> String {
    let config = &site.config;
    let asset = |name: &str| escape(&config.href(&format!("/_site/{name}")));
    let canonical = escape(&config.canonical(route));
    let title = escape(&format!("{title} | {}", config.name));
    let name = escape(&config.name);
    let section_route = format!(
        "/{}/",
        route
            .split('/')
            .find(|part| !part.is_empty())
            .unwrap_or_default()
    );
    let section_name = escape(
        site.pages
            .iter()
            .find(|page| page.route == section_route)
            .map(|page| page.navigation_title.as_str())
            .unwrap_or(&config.name),
    );
    let section_href = escape(
        &site
            .pages
            .iter()
            .find(|page| page.route == section_route)
            .map(|page| config.href(&page.route))
            .unwrap_or_else(|| config.href("/")),
    );
    let description = escape(description);
    let home = escape(&config.href("/"));
    let repository_link = config.repository.as_ref().map(|repository| format!(
        r#"<a class="sds-button" data-sds-shape="icon" data-sds-size="sm" data-sds-density="compact" data-sds-variant="text" href="{}" aria-label="View source" title="View source">{}</a>"#,
        escape(repository.as_str()), if repository.host_str() == Some("github.com") { crate::markup::documentation_icon("github").to_owned() } else { crate::markup::code_icon("code-xml") }
    )).unwrap_or_default();
    let navigation = if page.is_some() {
        navigation(site, route)
    } else {
        String::new()
    };
    let primary = primary_navigation(site, route, "site-header-menu", false);
    let mobile_primary = primary_navigation(site, route, "site-sidebar-menu", true);
    let toc = page.map(toc).unwrap_or_default();
    let has_sidebar = !navigation.is_empty();
    let sidebar_toggle = format!(
        r#"<button class="sds-button" type="button" data-sds-shape="icon" popovertarget="site-sidebar" aria-label="Open site navigation">{}</button>"#,
        crate::markup::documentation_icon("menu")
    );
    let section_navigation = if has_sidebar {
        format!(r#"<nav aria-label="Documentation">{navigation}</nav>"#)
    } else {
        String::new()
    };
    let sidebar = format!(
        r#"<aside id="site-sidebar" class="sds-sidebar" popover="auto" aria-label="Site navigation">
<header><a href="{section_href}">{section_name}</a><button class="sds-sidebar-close sds-button" type="button" data-sds-shape="icon" data-sds-density="compact" data-sds-variant="text" popovertarget="site-sidebar" popovertargetaction="hide" aria-label="Close site navigation">&times;</button></header>
<nav class="sds-app-navigation" aria-label="Documentation sections">{mobile_primary}</nav>
{section_navigation}
{toc}
</aside>"#
    );
    let robots = if noindex {
        "<meta name=\"robots\" content=\"noindex,follow\">"
    } else {
        ""
    };
    let dialog_form = search_form(site, "site-dialog", false);
    let structured = if let Some(page) = page {
        let value = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": page.title,
            "description": page.description,
            "url": config.canonical(&page.route),
            "inLanguage": config.language,
            "isPartOf": { "@type": "WebSite", "name": config.name, "url": config.url.as_str() }
        });
        format!(
            "<script type=\"application/ld+json\">{}</script>",
            value
                .to_string()
                .replace('<', "\\u003c")
                .replace('>', "\\u003e")
                .replace('&', "\\u0026")
        )
    } else {
        String::new()
    };
    format!(
        r##"<!doctype html>
<html lang="{language}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta name="description" content="{description}">
<meta name="generator" content="site-generator">
<title>{title}</title>
<link rel="canonical" href="{canonical}">
<meta property="og:type" content="website">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{canonical}">
<meta property="og:site_name" content="{name}">
<meta name="twitter:card" content="summary">
<link rel="icon" href="{favicon}" type="image/svg+xml">
<link rel="stylesheet" href="{sds}">
<link rel="stylesheet" href="{brand}">
<link rel="stylesheet" href="{documentation_css}">
<script type="module" src="{auto}"></script>
<script type="module" src="{search_script}"></script>
<script type="module" src="{documentation_script}"></script>
<script type="module" src="{highlight_script}"></script>
{structured}{robots}
</head>
<body data-sds-root class="sds-document" data-sds-theme="forge" data-sds-color-scheme="system">
<a class="sds-skip-link" href="#site-main">Skip to main content</a>
<div class="sds-app" data-sds-variant="documentation" data-sds-theme="forge">
<div class="sds-app-masthead" role="region" aria-label="SEI masthead">
<a href="https://sei.cmu.edu" aria-label="Software Engineering Institute"><span class="sds-sei-wordmark" aria-hidden="true"></span></a>
<div class="sds-cluster" data-sds-gap="sm">
{repository_link}
<button class="sds-button" type="button" data-sds-shape="icon" data-sds-size="sm" data-sds-density="compact" data-sds-variant="text" data-sds-action="toggle-color-scheme" aria-pressed="false" aria-label="Switch to dark mode" title="Switch to dark mode">{theme_icons}</button>
<a class="sds-button" href="{search_url}" data-sds-shape="icon" data-sds-size="sm" data-sds-density="compact" data-sds-variant="text" data-search-open aria-haspopup="dialog" aria-controls="site-search-dialog" aria-label="Search documentation" title="Search documentation">
{search_icon}
</a>
</div>
</div>
<header class="sds-app-header">
{sidebar_toggle}
<a class="sds-app-brand" href="{home}">{shield}<span>{name}</span></a>
<nav class="sds-app-navigation" aria-label="Documentation sections">{primary}</nav>
</header>
<div class="sds-app-layout">
{sidebar}
<div class="sds-app-body">
<main id="site-main" class="sds-app-main" tabindex="-1">
<div class="sds-app-content">{body}{toc}</div>
</main>
</div>
</div>
<footer class="sds-app-footer">
<div class="sds-app-footer-top" aria-hidden="true">{shield}</div>
<div class="sds-app-footer-content">
<div class="sds-app-footer-brand"><a href="https://sei.cmu.edu" aria-label="Software Engineering Institute"><span class="sds-sei-wordmark" aria-hidden="true"></span></a></div>
<nav class="sds-app-footer-middle" aria-label="Documentation footer"><ul><li><a href="https://www.cmu.edu/">www.cmu.edu</a></li></ul></nav>
<div class="sds-app-footer-legal"><p>&copy; <time datetime="2026">2026</time> Carnegie Mellon University</p><p><a href="https://sei.cmu.edu/legal/">Legal</a> | <a href="https://sei.cmu.edu/legal/privacy-notice/">Privacy Policy</a></p></div>
</div>
<p data-sds-margin="none" data-sds-padding-block-start="lg">{name} was created using the SEI Design System. <a href="{license}">Design system license and acknowledgments</a>.</p>
</footer>
<dialog id="site-search-dialog" class="sds-dialog" data-sds-width="lg" closedby="any" aria-labelledby="site-search-title">
<div class="sds-dialog-header"><h2 id="site-search-title" class="sds-text-h3">Search</h2>
<button class="sds-button" type="button" data-sds-shape="icon" data-sds-variant="text" commandfor="site-search-dialog" command="close" data-search-close aria-label="Close documentation search">&times;</button></div>
{dialog_form}
</dialog>
</div>
</body>
</html>
"##,
        language = escape(&config.language),
        favicon = asset("favicon.svg"),
        sds = asset("sds-lite/sds.css"),
        brand = asset("sds-lite/brand.css"),
        documentation_css = asset("sds-lite/documentation.css"),
        auto = asset("sds-lite/auto.js"),
        license = asset("sds-lite/LICENSE.txt"),
        search_script = asset("search.js"),
        search_url = escape(&config.href("/search/")),
        documentation_script = asset("sds-lite/documentation.js"),
        highlight_script = asset("highlight.js"),
        shield = SHIELD,
        theme_icons = format_args!(
            "{}{}",
            crate::markup::documentation_icon("dark-mode"),
            crate::markup::documentation_icon("light-mode")
        ),
        search_icon = crate::markup::documentation_icon("search")
    )
}

fn search_form(site: &Site, prefix: &str, sync_url: bool) -> String {
    format!(
        r#"<form id="{prefix}-search" class="sds-form" action="{action}" method="get" role="search" data-site-search data-search-prefix="{prefix}" data-sync-url="{sync_url}">
<div class="sds-field"><label class="sds-sr-only" for="{prefix}-query">Search all pages</label>
<input class="sds-input" id="{prefix}-query" name="q" type="text" role="searchbox" inputmode="search" maxlength="200" required autocomplete="off" {autofocus} placeholder="Search all pages" aria-controls="{prefix}-results" aria-describedby="{prefix}-status"></div>
</form>
<p id="{prefix}-status" role="status" aria-live="polite">Type a query in the search box above to display a list of results.</p>
<nav id="{prefix}-results" class="sds-search-results" aria-label="Search results"></nav>"#,
        action = escape(&site.config.href("/search/")),
        autofocus = if sync_url { "" } else { "autofocus" }
    )
}

fn primary_navigation(site: &Site, current: &str, prefix: &str, mobile: bool) -> String {
    if let Some(items) = &site.config.navigation {
        return configured_navigation(site, items, current, prefix, false, mobile);
    }
    let mut output = format!(
        "<ul><li><a href=\"{}\"{}>Home</a></li>",
        escape(&site.config.href("/")),
        if current == "/" {
            " aria-current=\"page\""
        } else {
            ""
        }
    );
    let mut seen = std::collections::BTreeSet::new();
    for page in site.pages.iter().filter(|page| page.in_navigation) {
        let Some(segment) = page.route.split('/').find(|s| !s.is_empty()) else {
            continue;
        };
        if !seen.insert(segment) {
            continue;
        }
        let root = format!("/{segment}/");
        let landing = site
            .pages
            .iter()
            .find(|p| p.in_navigation && p.route == root)
            .unwrap_or(page);
        let label = if landing.route == root {
            landing.navigation_title.clone()
        } else {
            segment.replace('-', " ")
        };
        output.push_str(&format!(
            "<li><a href=\"{}\"{}>{}</a></li>",
            escape(&site.config.href(&landing.route)),
            if current.starts_with(&root) {
                " aria-current=\"page\""
            } else {
                ""
            },
            escape(&label)
        ));
    }
    output.push_str("</ul>");
    output
}

fn configured_navigation(
    site: &Site,
    items: &[crate::config::Navigation],
    current: &str,
    prefix: &str,
    menu: bool,
    mobile: bool,
) -> String {
    let mut output = if menu {
        format!(
            "<menu id=\"{prefix}\" class=\"sds-dropdown-menu\" popover=\"auto\" role=\"menu\" aria-orientation=\"vertical\">"
        )
    } else {
        String::from("<ul>")
    };
    let current_href = site.config.href(current);
    for (index, item) in items.iter().enumerate() {
        let dropdown = !item.children.is_empty()
            && item.children.iter().all(|child| child.children.is_empty());
        let link = item.path.as_ref().map(|path| {
            format!(
                "<a href=\"{}\"{}{}>{}</a>",
                escape(path),
                if menu || dropdown {
                    " role=\"menuitem\""
                } else {
                    ""
                },
                if path.split(['?', '#']).next() == Some(current_href.as_str()) {
                    " aria-current=\"page\""
                } else {
                    ""
                },
                escape(&item.title)
            )
        });
        if item.children.is_empty() {
            output.push_str(&format!(
                "<li{}>{}</li>",
                if menu { " role=\"none\"" } else { "" },
                link.unwrap_or_default()
            ));
        } else if mobile {
            let id = format!("{prefix}-{index}");
            let children = configured_navigation(site, &item.children, current, &id, false, true);
            output.push_str(&format!(
                "<li><details class=\"sds-navigation-panel\"><summary{}>{}</summary><div><button type=\"button\" data-sds-action=\"navigation-back\">{}Go Back</button><strong>{}</strong>{children}</div></details></li>",
                if item.path.as_ref().is_some_and(|path| path.split(['?', '#']).next() == Some(current_href.as_str())) { " data-sds-active" } else { "" },
                escape(&item.title),
                crate::markup::code_icon("chevron-left"),
                escape(&item.title),
            ));
        } else {
            let id = format!("{prefix}-{index}");
            let mut children =
                configured_navigation(site, &item.children, current, &id, dropdown, false);
            if let Some(link) = link {
                let start = children.find('>').unwrap() + 1;
                children.insert_str(
                    start,
                    &format!(
                        "<li class=\"site-primary-overview\"{}>{link}</li>",
                        if dropdown { " role=\"none\"" } else { "" }
                    ),
                );
            }
            if dropdown {
                output.push_str(&format!(
                    "<li><sds-dropdown width=\"lg\" placement=\"block-end-end\"><button class=\"sds-button\" type=\"button\" data-sds-density=\"compact\" data-sds-variant=\"text\" popovertarget=\"{id}\" aria-controls=\"{id}\" aria-expanded=\"false\" aria-haspopup=\"menu\">{}</button>{children}</sds-dropdown></li>",
                    escape(&item.title)
                ));
            } else {
                output.push_str(&format!(
                    "<li><details class=\"sds-disclosure site-primary-group\"><summary>{}</summary>{children}</details></li>",
                    escape(&item.title)
                ));
            }
        }
    }
    output.push_str(if menu { "</menu>" } else { "</ul>" });
    output
}

fn toc(page: &Page) -> String {
    let headings: Vec<_> = page
        .headings
        .iter()
        .filter(|heading| heading.level > 1)
        .collect();
    let Some(first) = headings.first() else {
        return String::new();
    };
    let base = first.level;
    let mut depth = base;
    let mut result = format!(
        "<nav class=\"sds-app-toc\" aria-label=\"On this page\"><strong>{LIST_ORDERED}On this page</strong><ul>",
    );
    for (index, heading) in headings.iter().enumerate() {
        let level = heading.level.max(base).min(depth + 1);
        if index > 0 {
            if level > depth {
                result.push_str("<ul>");
            } else {
                result.push_str("</li>");
                for _ in level..depth {
                    result.push_str("</ul></li>");
                }
            }
        }
        depth = level;
        result.push_str(&format!(
            "<li><a href=\"#{}\">{}</a>",
            escape(&heading.id),
            escape(&heading.text)
        ));
    }
    result.push_str("</li>");
    for _ in base..depth {
        result.push_str("</ul></li>");
    }
    result.push_str("</ul></nav>");
    result
}

#[derive(Default)]
struct Nav<'a> {
    page: Option<&'a Page>,
    children: BTreeMap<String, Nav<'a>>,
}

fn navigation_tree(site: &Site) -> Nav<'_> {
    let mut root = Nav::default();
    for page in site.pages.iter().filter(|page| page.in_navigation) {
        let mut node = &mut root;
        for segment in page.route.split('/').filter(|s| !s.is_empty()) {
            node = node.children.entry(segment.into()).or_default();
        }
        node.page = Some(page);
    }
    root
}

fn collect_navigation_pages<'a>(node: &Nav<'a>, pages: &mut Vec<&'a Page>) {
    if let Some(page) = node.page {
        pages.push(page);
    }
    let mut children: Vec<_> = node.children.values().collect();
    children.sort_by_key(|child| child.page.map(|page| (page.order, page.source.clone())));
    for child in children {
        collect_navigation_pages(child, pages);
    }
}

fn page_navigation(site: &Site, current: &Page) -> String {
    let root = navigation_tree(site);
    let section = current
        .route
        .split('/')
        .find(|part| !part.is_empty())
        .and_then(|segment| root.children.get(segment))
        .filter(|section| !section.children.is_empty())
        .unwrap_or(&root);
    let mut pages = Vec::new();
    collect_navigation_pages(section, &mut pages);
    let Some(index) = pages.iter().position(|page| page.route == current.route) else {
        return String::new();
    };
    let previous = index
        .checked_sub(1)
        .map(|index| {
            format!(
                "<a href=\"{}\" rel=\"prev\">{}Previous page</a>",
                escape(&site.config.href(&pages[index].route)),
                crate::markup::code_icon("chevron-left")
            )
        })
        .unwrap_or_default();
    let next = pages
        .get(index + 1)
        .map(|page| {
            format!(
                "<a href=\"{}\" rel=\"next\">Next page{}</a>",
                escape(&site.config.href(&page.route)),
                crate::markup::code_icon("chevron-right")
            )
        })
        .unwrap_or_default();
    format!(
        "<nav class=\"sds-app-pagination sds-not-prose\" aria-label=\"Page navigation\">{previous}{next}</nav>"
    )
}

fn navigation(site: &Site, current: &str) -> String {
    let root = navigation_tree(site);
    let Some((segment, section)) = current
        .split('/')
        .find(|part| !part.is_empty())
        .and_then(|segment| root.children.get(segment).map(|section| (segment, section)))
        .filter(|(_, section)| !section.children.is_empty())
    else {
        return String::new();
    };
    let mut output = String::from("<ul>");
    nav_children(site, section, &format!("/{segment}/"), current, &mut output);
    output.push_str("</ul>");
    output
}

fn nav_children(site: &Site, node: &Nav<'_>, prefix: &str, current: &str, output: &mut String) {
    let mut children: Vec<_> = node.children.iter().collect();
    children.sort_by_key(|(_, child)| child.page.map(|p| (p.order, p.source.clone())));
    for (segment, child) in children {
        let route = format!("{prefix}{segment}/");
        let label = child
            .page
            .map(|p| p.navigation_title.clone())
            .unwrap_or_else(|| {
                let mut name = segment.replace('-', " ");
                if let Some(first) = name.get_mut(..1) {
                    first.make_ascii_uppercase();
                }
                name
            });
        if child.children.is_empty() {
            if let Some(page) = child.page {
                output.push_str(&format!("<li>{}</li>", nav_link(site, page, current)));
            }
        } else {
            output.push_str(&format!(
                "<li>{}<details{}><summary>{}</summary>",
                child
                    .page
                    .map(|page| nav_link(site, page, current))
                    .unwrap_or_default(),
                if current.starts_with(&route) {
                    " open"
                } else {
                    ""
                },
                child
                    .page
                    .map(|_| format!(
                        "<span class=\"sds-sr-only\">Toggle {} navigation</span>",
                        escape(&label)
                    ))
                    .unwrap_or_else(|| escape(&label))
            ));
            output.push_str("<ul>");
            nav_children(site, child, &route, current, output);
            output.push_str("</ul></details></li>");
        }
    }
}

fn nav_link(site: &Site, page: &Page, current: &str) -> String {
    format!(
        "<a href=\"{}\"{}>{}</a>",
        escape(&site.config.href(&page.route)),
        if page.route == current {
            " aria-current=\"page\""
        } else {
            ""
        },
        escape(&page.navigation_title)
    )
}
