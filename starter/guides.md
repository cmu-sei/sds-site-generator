# Writing guide

Every Markdown file in the content directory becomes a page. An index file is
the home page for its directory.

## Headings

Start with one main heading using a single hash. Use two hashes for sections,
and three hashes for subsections. Do not skip heading levels.

## Links

Link to other Markdown files using their relative filenames:

::code-block{quality="good"}
```markdown
[Publishing](publishing.md)
[Home](../index.md)
[This section](#links)
```
::

The generator checks page and heading links before publishing.

## Images

Place images in the public directory and describe them in the image text:

```markdown
![A diagram showing the review process](/images/review.png)
```

For a purely decorative image, use an empty description and the title
`"decorative"`. Local PNG, JPEG, GIF, WebP, AVIF, and sanitized SVG images are supported.
Image dimensions are added automatically. Large images must be resized before
publishing; the generator does not optimize images for you.

## Optional metadata

You do not need metadata for normal pages. YAML front matter between `---`
lines and TOML between `+++` lines are both supported:

```yaml
---
title: Writing guide
description: A helpful summary for search engines.
navigation:
  title: Writing
tags: [documentation]
---
```

YAML accepts additional metadata such as tags. Use `navigation: false` to hide
a page from navigation without unpublishing it. The TOML equivalent is:

```toml
+++
title = "A shorter navigation title"
description = "A helpful summary for search engines."
order = 2
draft = false
+++
```

Unknown TOML settings cause an error. Fenced YAML examples in the page body
are ordinary code examples, not metadata.

## Code examples

The link example above uses a built-in code-example element. Wrap a fenced
code block with `::code-block{quality="good"}` and a closing `::` line.
Use `quality="bad"` for a noncompliant example. Examples render without
JavaScript; a small first-party web component adds the copy button.

## Importing existing content

Safe semantic HTML, tables, named anchors, and legacy code-example wrappers
are supported. Scripts and unsafe attributes are removed. Set
`compatibility = true` in the site configuration to report legacy content
issues as visible warnings instead of failing immediately. Review the report
in `dist/_site/diagnostics.json`; `sds-site-generator check --strict` fails if
warnings remain. Duplicate page URLs always stop the build.

## Next steps

[Publish your website](publishing.md).
