---
order: 4
---
# Images and downloads

Put images and attachments in `public/`. Link with `/images/...`, not
`/public/images/...`; deployment prefixes are added automatically.

## Add an image

For `public/images/authoring-flow.svg`, write:

::code-block
```markdown
![Write content, check it, then publish the generated site](/images/authoring-flow.svg)
```
::

![Write content, check it, then publish the generated site](/images/authoring-flow.svg)

Describe the image's purpose. For decorative images, use empty alt text and
the title `decorative`:

::code-block
```markdown
![](/images/decoration.svg "decorative")
```
::

![](/images/decoration.svg "decorative")

Referenced local images need readable dimensions and must be at most
**1 MiB (1,048,576 bytes)** each. The generator adds dimensions and lazy loading.

Use local files in the [supported formats](#allowed-files); remote images produce
diagnostics. TIFF references get PNG aliases; originals remain downloadable.

## Add a download

For `public/downloads/author-checklist.txt`, write:

::code-block
```markdown
[Download the author checklist](/downloads/author-checklist.txt)
```
::

[Download the author checklist](/downloads/author-checklist.txt).
Browser and host headers determine whether it opens or downloads.

For imported download links, see
[compatibility mode's link resolution](../start/configuration.md#what-changes).

## Allowed files

| Category | Extensions |
| --- | --- |
| Images and icons | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.avif`, `.svg`, `.ico`, `.tif`, `.tiff` |
| Documents and data | `.pdf`, `.txt`, `.csv`, `.html`, `.xml`, `.url` |
| Archives | `.zip`, `.jar`, `.tar`, `.gz` |
| Source attachments | `.c`, `.cpp`, `.h`, `.java` |
| Fonts | `.woff2` |

Unsupported extensions, including author-provided JavaScript and CSS, fail
normal validation. SVG and HTML are sanitized; HTML attachments are standalone
files, not documentation pages. See [Search and hosting](../publishing/search-and-hosting.md#write-for-discoverability)
for attachment discoverability.

## Names and output collisions

Use simple lowercase filenames with hyphens. Spaces and Unicode are allowed,
but hidden path segments, control characters, backslashes, `:`, `?`, and `#`
are not. Percent-encode spaces in Markdown URLs, or enclose the destination in
angle brackets.

`public/_site/` and `public/robots.txt` are reserved. Builds reject collisions
with generated output, including page `index.html` files, `sitemap.xml`, and
`.gz` variants.

The bundled theme and favicon cannot be overridden through public assets.
