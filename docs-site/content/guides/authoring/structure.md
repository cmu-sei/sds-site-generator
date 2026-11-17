---
order: 1
---
# Pages and links

The directory layout defines page URLs and the documentation tree. Keep source
filenames predictable so links remain stable as the site grows.

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

Only lowercase `.md` extensions are accepted in `content/`; place all other
files in `public/`. Use UTF-8 text. Symbolic links are not allowed in the content
or public trees, and `site.toml` cannot be a symbolic link.

| Source under `content/` | Site route |
| --- | --- |
| `1.index.md` | `/` |
| `2.guides/1.index.md` | `/guides/` |
| `2.guides/2.installation.md` | `/guides/installation/` |
| `3.reference.md` | `/reference/` |

Numeric prefixes followed by a dot are removed from URLs. Other words are
lowercased and separated by hyphens; underscores also become hyphens in normal
mode. For example, `API Reference.md` becomes `/api-reference/`.
Compatibility mode preserves underscores.

An `index.md` is a directory's landing page. The home route must have a
published page. Directories group descendants even without a landing page,
but adding an index gives the group a useful title and destination.

Routes must be unique: `guide.md` and `guide/index.md` collide. `/search/`,
`/404/`, and routes under `/_site/` are reserved. There is no redirect setting;
renaming a source may require redirects supplied by your host.

## Control ordering and labels

Sibling pages sort by front-matter `order` first (default `0`), then source
path. Smaller orders come first. Numeric filename prefixes participate in the
source-path ordering, not numeric comparison: use `01.`, `02.`, ..., `10.`
consistently, or set explicit `order` values.

For predictable group ordering, give each directory an index page with an
`order`. Page titles label navigation by default; override the label with
`navigation.title`, or use `navigation: false` to hide the page.

The default top menu comes from top-level content. Within a section, a sidebar
shows its descendants. Previous/next links follow the visible section tree.
[Custom main navigation](../start/configuration.md#main-navigation) changes the top
menu, not the content tree.

## Link to pages

Prefer source-relative Markdown links. Include the real source filename,
including any numeric prefix:

::code-block
```markdown
[Installation](2.guides/2.installation.md)
[Publishing](../3.publishing.md)
[Section](2.guides/2.installation.md#requirements)
```
::

These are illustrative paths; adjust them relative to the file you are editing.
Links ending in `.md` resolve relative to the source file. Other relative links
resolve relative to the generated page URL, which may be different.

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

Do not include the deployment prefix: with `url = "https://example.org/docs/"`,
`/guides/installation/` becomes `/docs/guides/installation/` automatically.
Local pages, files, and heading fragments are checked; external availability
is not. Use explicit `https://`, not protocol-relative `//` URLs.

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
