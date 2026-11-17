---
order: 4
---
# Images and downloads

Put images and attachments in `public/`. They are copied to the output root,
not to a `/public/` URL. Use site-root links to keep them independent of the
page's nesting and deployment prefix.

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

The generator adds dimensions when it can read them, plus lazy loading and
asynchronous decoding. Referenced local images must have readable dimensions
and be at most **1 MiB (1,048,576 bytes)** each. Resize and compress images
yourself: the generator is not a general image optimizer.

Use local PNG, JPEG, GIF, WebP, AVIF, or sanitized SVG. Remote images produce
diagnostics because their availability and dimensions cannot be validated.
Prefer converting TIFF to a modern format; legacy TIFF references get PNG
aliases, while the original files remain downloadable.

## Add a download

For `public/downloads/author-checklist.txt`, write:

::code-block
```markdown
[Download the author checklist](/downloads/author-checklist.txt)
```
::

[Download the author checklist](/downloads/author-checklist.txt).
Whether a file opens or downloads depends on the browser and host headers.
The generator does not force a download.

## Allowed files

| Category | Extensions |
| --- | --- |
| Images and icons | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.avif`, `.svg`, `.ico`, `.tif`, `.tiff` |
| Documents and data | `.pdf`, `.txt`, `.csv`, `.html`, `.xml`, `.url` |
| Archives | `.zip`, `.jar`, `.tar`, `.gz` |
| Source attachments | `.c`, `.cpp`, `.h`, `.java` |
| Fonts | `.woff2` |

Author-provided JavaScript and CSS are not published. Unsupported extensions
are errors in normal mode. SVG and HTML attachments are sanitized; removals
are diagnosed. HTML attachments are downloads/standalone documents, not
Markdown pages in the navigation or search index.

## Names and output collisions

Use simple lowercase filenames with hyphens. Spaces and Unicode are allowed,
but hidden path segments, control characters, backslashes, `:`, `?`, and `#`
are not. Percent-encode spaces in Markdown URLs, or enclose the destination in
angle brackets.

Do not put files under `public/_site/` or provide `public/robots.txt`.
Avoid all generated-output collisions, including page `index.html` files,
`sitemap.xml`, and generated `.gz` variants. The build rejects collisions
instead of overwriting output.

The shell uses its bundled favicon and styling. Uploading `public/favicon.svg`
does not replace the shell's favicon, and publishing a font does not apply it
automatically. There are no configuration fields for custom themes, favicon
selection, or custom scripts.
