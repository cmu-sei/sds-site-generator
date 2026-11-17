# Writing guide

Every Markdown file in the content directory becomes a page. An index file is
the home page for its directory.

## Headings

Start with one main heading using a single hash. Use two hashes for sections,
and three hashes for subsections. Do not skip heading levels.

## Links

Link to other Markdown files using their relative filenames:

::code-block
```markdown
[Publishing](publishing.md)
[Home](../index.md)
[This section](#links)
```
::

The generator checks page and heading links before publishing.

## Images

Place images in the public directory and describe them in the image text:

::code-block
```markdown
![A diagram showing the review process](/images/review.png)
```
::

For a purely decorative image, use an empty description and the title
`"decorative"`. Local PNG, JPEG, GIF, WebP, AVIF, and sanitized SVG images are supported.
Image dimensions are added automatically. Large images must be resized before
publishing; the generator does not optimize images for you.

## Optional metadata

You do not need metadata for normal pages. YAML front matter between `---`
lines and TOML between `+++` lines are both supported:

::code-block
```yaml
---
title: Writing guide
description: A helpful summary for search engines.
navigation:
  title: Writing
tags: [documentation]
---
```
::

YAML accepts additional metadata such as tags. Use `navigation: false` to hide
a page from navigation without unpublishing it. The TOML equivalent is:

::code-block
```toml
+++
title = "A shorter navigation title"
description = "A helpful summary for search engines."
order = 2
draft = false
+++
```
::

Unknown TOML settings cause an error. Fenced YAML examples in the page body
are ordinary code examples, not metadata.

## Code examples

The examples above use the shared code-block element. Wrap a fenced code block
with `::code-block` and a closing `::` line. The default treatment is neutral;
use `quality="good"` or `quality="bad"` for compliant/noncompliant examples.
Examples render without JavaScript; the shared SDS runtime adds copy and
fullscreen controls.

Put a lowercase language label on the opening fence to select syntax
highlighting, for example `markdown`, `toml`, `powershell`, or `ini`.
The bundled grammars also cover C, C++, Java, Perl, free-form Fortran,
JavaScript, TypeScript, Python, shell, HTML, XML, JSON, YAML, and diff, with
CSS, GLSL, SQL, regular-expression, and C++ macro dependency grammars.
Use `text` or no label for plain text; unknown labels are not highlighted.
Highlighting requires JavaScript and uses local assets, not an external service.

## Callouts

Use semantic tones and visual variants matching SDS Lite:

::code-block
```markdown
::callout{tone="warning" variant="outline" size="sm"}
**Warning:** Check local links before publishing.
::
```
::

Tones are `neutral` (default), `accent`, `info`, `success`, `warning`, and
`danger`. Omit `variant` for a subtle surface, or use `outline` or `bold`.
Sizes are `xs`, `sm`, `md` (default), and `lg`.

## Importing existing content

Safe semantic HTML, tables, named anchors, and legacy code-example wrappers
are supported. Scripts and unsafe attributes are removed. Set
`compatibility = true` in the site configuration to report legacy content
issues as visible warnings instead of failing immediately. Review the report
in `dist/_site/diagnostics.json`; `sds-site-generator check --strict` fails if
warnings remain. Duplicate page URLs always stop the build.

## Next steps

[Publish your website](publishing.md).
