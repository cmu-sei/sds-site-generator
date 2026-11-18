---
order: 1
---
# Pages and links

Your directory layout defines page URLs and the documentation tree.
Use predictable filenames to keep links stable.

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
Compatibility mode keeps underscores.

`index.md` is a directory's landing page. The home route needs a published page.
Other directories group descendants without an index, but adding one gives
the group a title and destination.

Routes must be unique: `guide.md` and `guide/index.md` collide. `/search/`,
`/404/`, and routes under `/_site/` are reserved. There is no redirect setting;
renaming a source may require redirects supplied by your host.

## Control ordering and labels

Siblings sort by front-matter `order` (default `0`, smallest first), then source
path. Filename prefixes sort as text, not numbers: use `01.`, `02.`, ..., `10.`
consistently, or set `order`.

Set group order on each directory's index page. Page titles label navigation;
override with `navigation.title`, or hide a page with `navigation: false`.

Top-level content supplies the default main menu; section descendants supply
the sidebar and previous/next links. [Custom main navigation](../start/configuration.md#main-navigation)
changes the menu, not the content tree.

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

Adjust these example paths to your files. `.md` links resolve from the source
file; other relative links resolve from the generated page URL.

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
Validation checks local pages, files, and fragments, not external availability.
Use `https://`, not protocol-relative `//` URLs.

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
