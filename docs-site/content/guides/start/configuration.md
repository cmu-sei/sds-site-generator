---
order: 3
---
# Site configuration

Place `site.toml` in the site root, beside `content/` and `public/`.
Use TOML, not YAML. Unknown top-level settings are rejected.

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

Set site-level values **before** any `[[navigation]]` declarations. In TOML,
values after a table declaration belong to that table until another
declaration changes the context.

## Choose the publishing URL

::code-block
```toml
# Project site:
url = "https://your-org.github.io/my-docs/"
```
::

For a domain-root site, use a root URL such as `https://docs.example.org/`.
The URL controls canonical links, sitemap entries, asset paths, and preview's
local mount point. Use the final public URL, not the localhost preview address.
A missing trailing slash is added automatically.

URLs cannot contain credentials, query strings, or fragments. Deployment
subdirectory segments may contain only ASCII letters, numbers, hyphens, and
underscores. Use HTTPS for production. The scaffold's `https://example.com/`
is a placeholder, not a deployment destination.

`repository` also rejects credentials. `language` must consist of nonempty
alphanumeric segments separated by hyphens; choose a meaningful language tag.

## Language and translations

`language` sets the document-wide HTML language, helping browsers and assistive
technology interpret the content. It does not translate page content, search
messages, or interface labels, and it does not create language-specific menus
or alternate-language metadata.

There is no per-page language configuration or automatic translation workflow.
For separately maintained language editions, build separate sites with their
own configuration and publishing URL, then link between them explicitly.
Do not assume changing this setting localizes the English interface.

## Metadata and discoverability

The generator derives metadata from your content; it is not configured through
arbitrary front-matter keys:

| Output | Source |
| --- | --- |
| Browser tab title and social title | Page `title`, followed by the site `name` |
| Page description and social description | Page `description`, or the introductory paragraph fallback |
| Canonical URL and social URL | Site `url` plus the generated page route |
| Sitemap destinations | Published page routes under the configured site `url` |
| HTML language | Site `language` |

Use a specific title and a concise, accurate description for important pages.
The site-level `description` describes the search page; it does not replace
individual page descriptions.

Basic Open Graph, a Twitter summary-card declaration, and WebPage structured
data are generated. There are no supported fields for custom social-preview
images, arbitrary head tags, per-page canonical overrides, or per-page
`noindex`. Extra YAML keys do not enable these features.
Search engines and social platforms decide how to display or cache previews;
the generator cannot guarantee an exact snippet or immediate indexing.

## Main navigation

Omit `navigation` for an automatic menu based on top-level content.
To replace it, append ordered entries:

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

The local destinations above assume those pages exist in your site.
Each item requires a nonempty `title` and either a `path` or nonempty `children`.
Paths must be site-root paths or absolute HTTP(S) URLs without credentials.
Relative paths, `mailto:` menu destinations, and protocol-relative URLs are
not supported.

Groups with children become header dropdowns and expandable mobile groups.
A group's optional `path` adds a landing-page link inside it. Add deeper
`[[navigation.children.children]]` entries to nest groups. Order follows the
configuration.

Local paths omit the deployment prefix; it is added automatically. Page and
fragment targets are validated, even in compatibility mode. Main-menu
configuration does not replace or reorder the documentation sidebar tree.

To hide the main menu entirely, put this among the top-level settings and
remove all `[[navigation]]` tables:

::code-block
```toml
navigation = []
```
::

Page-specific navigation labels and visibility belong in
[front matter](../authoring/markdown.md#front-matter), not in this file. There are no
additional supported theme, plugin, output-directory, or redirect settings.
