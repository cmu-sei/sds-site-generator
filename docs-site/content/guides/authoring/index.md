---
order: 2
---
# Write content

Organize pages, write clear Markdown, and add reusable presentation elements
and public assets.

| Guide | What you will learn |
| --- | --- |
| [Pages and links](structure.md) | Folders, routes, ordering, sidebar groups, and heading links |
| [Markdown and metadata](markdown.md) | Page structure, front matter, tables, code, and safe HTML |
| [Callouts and code blocks](components.md) | Notes, warnings, compliance labels, and short/complete examples |
| [Images and downloads](assets.md) | Public paths, image descriptions, size limits, and allowed files |
| [Accessible authoring](accessibility.md) | Useful descriptions, clear links, simple tables, and manual review |

## How this sidebar is organized

Each sidebar group is a directory with an index page and related guides:

::code-block
```text
content/
  guides/
    index.md
    start/
      index.md
      getting-started.md
      configuration.md
    authoring/
      index.md
      structure.md
      markdown.md
      components.md
      assets.md
      accessibility.md
    publishing/
      index.md
      deployment.md
      team-workflows.md
      search-and-hosting.md
      licensing.md
    maintenance/
      index.md
      importing.md
      url-migrations.md
```
::

Index-page titles name the groups; their `order` values place them in sequence.
Each child has its own order and may use a shorter `navigation.title`.
Expand a group to browse, or visit a child to open its group automatically.
On small screens, use **Open site navigation** to see the same tree.

The generator builds this navigation from your directory structure.
Configure the header menu separately in
[site.toml](../start/configuration.md#main-navigation).

When your content is ready, continue to
[Publish and operate](../publishing/index.md).
