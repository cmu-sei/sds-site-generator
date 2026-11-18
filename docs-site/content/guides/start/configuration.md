---
order: 3
---
# Site configuration

Configure your site in `site.toml`, beside `content/` and `public/`.
Use TOML; unknown top-level settings fail validation.

## All site settings

::code-block
```toml
name = "My documentation"
url = "https://example.org/docs/"
language = "en"
description = "Guides and reference for our toolkit."
compatibility = false
repository = "https://github.com/example/my-docs"
```
::

| Setting | Required? | Meaning |
| --- | --- | --- |
| `name` | Yes | Nonempty site name displayed by the shell |
| `url` | Yes | Absolute HTTP(S) publishing URL, including any deployment subdirectory |
| `language` | No; `en` | HTML language tag, such as `en-US` |
| `description` | No; empty | Description for the generated search page, not a default for content pages |
| `compatibility` | No; `false` | Turn selected import problems into warnings and preserve underscores in routes |
| `repository` | No | Absolute HTTP(S) URL for the masthead source link |
| `navigation` | No; automatic | Ordered custom main-menu entries; see below |

Set site-level values **before** `[[navigation]]`; values after a TOML table
declaration belong to that table.

## Choose the publishing URL

::code-block
```toml
# Project site:
url = "https://your-org.github.io/my-docs/"
```
::

Use your final public URL, not localhost. For a domain-root site, use
`https://docs.example.org/`. The URL controls canonical links, sitemaps, asset
paths, and preview's mount point. A trailing slash is added if missing.

URLs cannot contain credentials, query strings, or fragments. Deployment
subdirectory segments may contain only ASCII letters, numbers, hyphens, and
underscores. Use HTTPS for production.

`repository` also rejects credentials. `language` must consist of nonempty
alphanumeric segments separated by hyphens; choose a meaningful language tag.

## Language and translations

`language` sets the HTML language for browsers and assistive technology.
Interface labels and search messages are in English.

Maintain each translation as a separate site with its own `language` and
publishing URL. Link between editions.

## Metadata and discoverability

Metadata comes from these settings and page content:

| Output | Source |
| --- | --- |
| Browser tab title and social title | Page `title`, followed by the site `name` |
| Page description and social description | Page `description`, or the introductory paragraph fallback |
| Canonical URL and social URL | Site `url` plus the generated page route |
| Sitemap destinations | Published page routes under the configured site `url` |
| HTML language | Site `language` |

Give pages specific titles and concise descriptions. The site-level
`description` applies only to the search page.

The output includes Open Graph, a Twitter summary-card declaration, and
WebPage structured data. Custom social images, arbitrary head tags, per-page
canonical overrides, and `noindex` are unsupported. Search and social platforms
control preview display and caching.

## Main navigation

Omit `navigation` for an automatic top-level menu, or append ordered entries:

::code-block
```toml
[[navigation]]
title = "Home"
path = "/"

[[navigation]]
title = "Guides"
path = "/guides/"

[[navigation.children]]
title = "Publishing"
path = "/guides/publishing/"

[[navigation.children]]
title = "Requirements"
path = "/guides/start/getting-started/#get-the-generator"

[[navigation]]
title = "Community"

[[navigation.children]]
title = "Source repository"
path = "https://github.com/example/my-docs"
```
::

Create the example destinations in your site before using this menu.
Each item needs a nonempty `title` and either a `path` or nonempty `children`.
Paths must be site-root paths or absolute HTTP(S) URLs without credentials;
relative, `mailto:`, and protocol-relative menu links are unsupported.

Children create header dropdowns and expandable mobile groups. A group's
optional `path` adds a landing-page link. Nest further with
`[[navigation.children.children]]`. Items follow configuration order.

Omit the deployment prefix; it is added automatically. Local pages and
fragments are validated, even in compatibility mode. This menu does not
reorder or replace the sidebar tree.

To hide the menu, remove all `[[navigation]]` tables and add this top-level setting:

::code-block
```toml
navigation = []
```
::

Set page labels and visibility in [front matter](../authoring/markdown.md#front-matter).
Theme, plugin, output-directory, and redirect settings are unsupported.
