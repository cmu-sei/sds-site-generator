---
order: 1
---
# Pages and links

Your directory layout defines page URLs and navigation.

## Arrange the files

::code-block
```text
my-docs/
  site.toml
  content/
    1.index.md
    2.guides/
      1.index.md
      2.installation.md
      3.publishing.md
    3.reference.md
  public/
    images/
      diagram.svg
```
::

Use UTF-8 files with lowercase `.md` extensions in `content/`; put other files
in `public/`. Do not use symbolic links in either directory or for `site.toml`.

| Source under `content/` | Site route |
| --- | --- |
| `1.index.md` | `/` |
| `2.guides/1.index.md` | `/guides/` |
| `2.guides/2.installation.md` | `/guides/installation/` |
| `3.reference.md` | `/reference/` |

Routes drop numeric prefixes followed by a dot, lowercase words, and replace
spaces and underscores with hyphens: `API Reference.md` becomes `/api-reference/`.
See [compatibility mode](../start/configuration.md#compatibility-mode) for legacy routes.

`index.md` is a directory's landing page and is required at the content root.
Other directories can group pages without one; adding one supplies a group
title and destination.

Routes must be unique: `guide.md` and `guide/index.md` collide. `/search/`,
`/404/`, and routes under `/_site/` are reserved. Before renaming published
pages, review [URL migrations](../maintenance/url-migrations.md).

## Control ordering and labels

Siblings sort by front-matter `order` (smallest first), then source
path. Filename prefixes sort as text, not numbers: use `01.`, `02.`, ..., `10.`
consistently, or set `order`.

Set group order on each directory's index page. Use
[front matter](markdown.md#front-matter) for page labels and visibility.

Top-level content supplies the default main menu; section descendants supply
the sidebar and previous/next links. [Custom main navigation](../start/configuration.md#main-navigation)
changes the menu, not the content tree.

## Link to pages

Use source-relative Markdown links with the real filenames and numeric prefixes.
From `content/2.guides/1.index.md` in the tree above:

::code-block
```markdown
[Installation](2.installation.md)
[Publishing](3.publishing.md)
[Home](../1.index.md)
[Requirements](2.installation.md#requirements)
```
::

`.md` links resolve from the source file; other relative links resolve from
the generated page URL.

Site-root links are often clearer:

::code-block
```markdown
[Installation](/guides/installation/)
[Search](/search/)
[Requirements](/guides/installation/#requirements)
[Contact](mailto:docs@example.org)
[External reference](https://example.org/reference/)
```
::

Omit the deployment prefix: `/guides/installation/` automatically becomes
`/docs/guides/installation/` when the site's URL ends in `/docs/`.
Validation checks local pages, files, and fragments, not external availability.
Use `https://`, not `//`, for web links.

## Link to headings

`## Before you begin` gets the anchor `#before-you-begin`. Repeated headings
receive `-2`, `-3`, and so on. Use the generated heading link to confirm its ID.
If you rename a heading, update links to it.

For an explicit stable anchor, semantic HTML can supply an ID:

::code-block
```html
<h2 id="requirements">Before you begin</h2>
```
::

Do not use the shell's reserved IDs: `site-main`, `site-sidebar`, `site-search`,
`site-query`, `site-results`, `site-status`, `site-search-dialog`,
`site-search-title`, `site-dialog-search`, `site-dialog-query`,
`site-dialog-status`, or `site-dialog-results`.
