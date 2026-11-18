---
order: 4
---
# Images and downloads

Put images and attachments in `public/`; they copy to the output root.
Link with `/images/...`, not `/public/images/...`, to work across page nesting
and deployment prefixes.

## Add an image

For `public/images/authoring-flow.svg`, write:

::code-block
```markdown
![Write content, check it, then publish the generated site](/images/authoring-flow.svg)
```
::

![Write content, check it, then publish the generated site](/images/authoring-flow.svg)

Describe the image's purpose, not just its appearance. For a purely decorative
image, explicitly use empty alt text and the title `decorative`:

::code-block
```markdown
![](/images/decoration.png "decorative")
```
::

Referenced local images need readable dimensions and must be at most
**1 MiB (1,048,576 bytes)** each. Resize and compress them first. The generator
adds dimensions, lazy loading, and asynchronous decoding.

Use local PNG, JPEG, GIF, WebP, AVIF, or sanitized SVG. Remote images produce
diagnostics: their availability and dimensions cannot be validated. Convert
TIFF to a modern format; legacy TIFF references get PNG aliases and retain
the originals as downloads.

## Add a download

For `public/downloads/author-checklist.txt`, write:

::code-block
```markdown
[Download the author checklist](/downloads/author-checklist.txt)
```
::

[Download the author checklist](/downloads/author-checklist.txt).
Browser and host headers determine whether it opens or downloads.

## Allowed files

| Category | Extensions |
| --- | --- |
| Images and icons | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.avif`, `.svg`, `.ico`, `.tif`, `.tiff` |
| Documents and data | `.pdf`, `.txt`, `.csv`, `.html`, `.xml`, `.url` |
| Archives | `.zip`, `.jar`, `.tar`, `.gz` |
| Source attachments | `.c`, `.cpp`, `.h`, `.java` |
| Fonts | `.woff2` |

Author-provided JavaScript and CSS are not published; unsupported extensions
fail in normal mode. SVG and HTML are sanitized with diagnostics for removals.
HTML attachments are standalone documents, excluded from navigation and search.

## Names and output collisions

Use simple lowercase filenames with hyphens. Spaces and Unicode are allowed,
but hidden path segments, control characters, backslashes, `:`, `?`, and `#`
are not. Percent-encode spaces in Markdown URLs, or enclose the destination in
angle brackets.

`public/_site/` and `public/robots.txt` are reserved. Builds reject collisions
with generated output, including page `index.html` files, `sitemap.xml`, and
`.gz` variants.

Bundled styling and the favicon are fixed. Public assets do not override the
theme; custom themes, favicon selection, and scripts are unsupported.
