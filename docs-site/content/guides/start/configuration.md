---
order: 3
---
# Site configuration

Edit `site.toml` beside `content/` and `public/`. Unknown settings fail validation.

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
| `compatibility` | No; `false` | Enable legacy import behavior; see [Compatibility mode](#compatibility-mode) |
| `repository` | No | Absolute HTTP(S) URL for the masthead source link |
| `navigation` | No; automatic | Ordered custom main-menu entries; see below |

Put site-level settings **before** `[[navigation]]`; later values belong to that table.

## Choose the publishing URL

::code-block
```toml
# Project site:
url = "https://your-org.github.io/my-docs/"
```
::

Use the final public URL, not localhost; for a domain-root site, use
`https://docs.example.org/`. It controls canonical links, sitemaps, asset paths,
and preview's mount point. A trailing slash is added automatically.

URLs cannot contain credentials, query strings, or fragments. Deployment
subdirectory segments may contain only ASCII letters, numbers, hyphens, and
underscores. Use HTTPS for production.

`repository` also rejects credentials.

## Compatibility mode

Compatibility mode helps migrate Markdown and assets from sites built with the
[SDS Documentation Site template](https://github.com/cmu-sei/sds-docs-template),
which uses the `@cmu-sei/sds-documentation-site` Nuxt Layer. It preserves legacy
link conventions and reports selected import problems as warnings.

**Leave it off for new sites.** To enable it site-wide, set this top-level value:

::code-block
```toml
compatibility = true
```
::

### What changes

| Area | With `compatibility = true` |
| --- | --- |
| Page routes | Preserve underscores instead of converting them to hyphens |
| Generated heading fragments | Use Nuxt-style slugging and duplicate numbering, preserving underscores and prefixing numeric IDs with `_`; collisions get unique IDs without taking another legacy destination |
| Relative download links | Try the public root only when no page-relative file or page matches |
| Legacy callouts | Accept old color-based properties and report migration warnings; see [Legacy callouts](#legacy-callouts) |
| Selected validation failures | Continue with warnings for missing links or fragments, missing image descriptions, remote or oversized images, heading repairs, missing descriptions, unsupported markup or files, and sanitization changes |

Warnings may leave broken links or omit unsupported files. Use current authoring
syntax and root-relative download URLs for new content.

### What it does not do

It does not run Nuxt/Vue widgets, bindings, plugins, or scripts, or add custom
themes. YAML metadata, semantic HTML, anchors, attachments, and documented
wrappers already work in normal mode.

Invalid metadata, configuration, or menu targets; duplicate page URLs; unsafe
public filenames; output collisions; and invalid callout properties still fail.
Unsafe markup is always sanitized.

### Review diagnostics

The terminal shows up to 20 warnings. Builds write every file, line, and message
to `dist/_site/diagnostics.json`:

::code-block
```sh
sds-site-generator build .
sds-site-generator check . --strict
```
::

`--strict` fails on remaining warnings. Fix the source issues and review the
site before publishing; see [Import an existing site](../maintenance/importing.md#import-an-existing-site).

### Legacy callouts

Compatibility mode renders this legacy syntax with a migration warning:

::code-block
```markdown
::callout{variant="orange" type="outline" size="sm"}
**Warning:** Check local links.
::
```
::

Convert it to the [current callout syntax](../authoring/components.md#callouts):

::code-block
```markdown
::callout{tone="warning" variant="outline" size="sm"}
**Warning:** Check local links.
::
```
::

Map old color variants to tones: `blue`/`indigo` to `info`, `green`/`teal` to
`success`, `orange` to `warning`, `red` to `danger`, `gray` to `neutral`, and
`purple` to `accent`. Rename `type` to `variant`, omitting it when the old
type was `subtle`. Keep `size` unchanged.

Convert all legacy callouts to pass strict validation. Never mix `tone` with
legacy properties; a bare `::callout` is neutral in either mode.

### Disable it deliberately

After repairing the import, set `compatibility = false` and rerun strict
validation. This can change underscore routes and generated heading fragments
even when validation passes. Keep it enabled if those public URLs must remain,
or plan a [URL migration](../maintenance/url-migrations.md) before disabling it.

## Language and translations

`language` sets the HTML language. Use nonempty alphanumeric segments separated
by hyphens, such as `en-US`. Interface labels remain English.
For translations, build separate sites with their own language and URL, then
link between editions.

## Metadata and discoverability

Generated metadata uses site settings and page content:

| Output | Source |
| --- | --- |
| Browser tab title and social title | Page `title`, followed by the site `name` |
| Page description and social description | [Page description](../authoring/markdown.md#front-matter) |
| Canonical URL and social URL | Site `url` plus the generated page route |
| Sitemap destinations | Published page routes under the configured site `url` |
| HTML language | Site `language` |

The output includes Open Graph, a Twitter summary-card declaration, and
WebPage structured data. Custom social images, arbitrary head tags, per-page
canonical overrides, and per-page `noindex` settings are unsupported.

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

[[navigation]]
title = "Community"

[[navigation.children]]
title = "Source repository"
path = "https://github.com/example/my-docs"
```
::

Use destinations that exist in your site. Each entry needs a nonempty `title`
and either a `path` or nonempty `children`. Paths must be site-root paths
or absolute HTTP(S) URLs without credentials.

Entries follow configuration order. Children create dropdowns and mobile
groups; add a group `path` for a landing-page link. Nest further with
`[[navigation.children.children]]`.

Omit the deployment prefix; it is added automatically. Local pages and fragments
are validated. This menu does not change the sidebar tree.

To hide the menu, remove all `[[navigation]]` tables and add this top-level setting:

::code-block
```toml
navigation = []
```
::

Set page labels and visibility in [front matter](../authoring/markdown.md#front-matter).
Theme, plugin, output-directory, and redirect settings are unsupported.
