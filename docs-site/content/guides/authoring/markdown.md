---
order: 2
navigation:
  title: Markdown and metadata
---
# Writing Markdown

Write Markdown with clear headings. Tables, strikethrough, smart punctuation,
and [component wrappers](components.md) are supported.

## A complete page

::code-block
```markdown
---
title: Install the toolkit
description: Set up the toolkit and verify your first installation.
order: 2
draft: false
navigation:
  title: Installation
---
# Install the toolkit

Set up the toolkit before following the tutorials.

## Requirements

You need an account on your team's documentation host.

### Supported systems

Ask your team which operating systems it supports.
```
::

Use one `#` heading, then `##` sections and `###` subsections without skipping
levels. A missing H1 is generated from the page title; metadata does not replace
an existing H1. All headings have anchors, but only `##` sections appear in
"On this page."

## Front matter

Put optional metadata first: YAML between `---` lines (above), or TOML between
`+++` lines:

::code-block
```toml
+++
title = "Install the toolkit"
description = "Set up the toolkit and verify your first installation."
order = 2
draft = false
navigation = { title = "Installation" }
+++
```
::

| Field | Default and effect |
| --- | --- |
| `title` | First H1, otherwise the filename with its order prefix removed |
| `description` | First nonempty paragraph, truncated to 160 characters |
| `order` | `0`; signed 32-bit integer used to sort sibling pages |
| `draft` | `false`; `true` excludes the page from output, navigation, sitemap, and search |
| `navigation` | Visible; `false` hides the page from navigation, or an object sets `title` |

Every published page needs a description or introductory paragraph. Hidden pages
are **still published and searchable**, not private. Links to drafts fail
validation; the home page cannot be a draft.

YAML accepts extra metadata such as `tags`; only the fields above affect output.
Unknown top-level TOML fields fail validation. Metadata is limited to 64 KiB.

## Text, lists, and quotations

::code-block
```markdown
Use **bold** for emphasis, *italics* for a term, and `code` for literals.
Use ~~strikethrough~~ sparingly.

- One unordered item
- Another item
  - A nested item

1. First step
2. Second step

> A quotation or a short note.
```
::

Separate blocks with blank lines. Smart punctuation converts prose quotes and
punctuation, leaving code literal. Escape Markdown with backslashes: `\*literal\*`.

## Tables

::code-block
```markdown
| Setting | Meaning |
| --- | --- |
| `name` | Website title |
| `url` | Public deployment URL |
```
::

Include a header row. Wide tables scroll horizontally; JavaScript adds
keyboard-accessible sorting to simple tables. Tables with merged cells or
interactive headers stay static. Use lists for non-tabular content.

## Code

Use inline backticks for short literals. Wrap language-tagged fences for
copy and fullscreen controls:

::code-block
````markdown
::code-block
```sh
sds-site-generator check . --strict
```
::
````
::

The [code-block reference](components.md#code-examples) covers titles, compliance
badges, and short/complete examples. Plain fences remain keyboard-focusable and
horizontally scrollable, without copy or fullscreen controls.

The fence's [language label](components.md#syntax-highlighting) selects bundled
highlighting. Use `text` for plain output; unknown or missing labels stay
unhighlighted. Code remains readable without JavaScript.

Enclose triple-backtick examples in four-backtick fences. Component syntax
inside a fence stays literal.

## HTML and unsupported extensions

Add semantic HTML such as `<sup>`, `<sub>`, or explicit heading IDs. HTML uses
the same heading, link, image, and search processing as Markdown.

Sanitization removes and diagnoses scripts, forms, iframes, event handlers,
unsafe attributes, and unsupported markup/styles. Only conservative inline
layout styles are retained, not arbitrary theme styling.

Use explicit links and [supported components](components.md).
Footnotes, task-list checkboxes, math rendering, heading attribute shorthand,
automatic bare-URL linking, includes, templates, and arbitrary Vue/MDC widgets
are unsupported.
