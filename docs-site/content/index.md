# Publish documentation from Markdown

SDS Site Generator turns Markdown and local assets into a
searchable, static website using the SEI Design System's SDS Lite interface.
Authors need one executable, not a web framework or a search service.

## Start here

1. [Install the generator](guides/start/installation.md) for macOS, Linux, or Windows.
2. [Create and preview a site](guides/start/getting-started.md).
3. Set the name, publishing URL, and menu in [site.toml](guides/start/configuration.md).
4. [Organize pages and links](guides/authoring/structure.md).
5. [Write Markdown](guides/authoring/markdown.md) and add [images and downloads](guides/authoring/assets.md).
6. [Validate and publish](guides/publishing/deployment.md).

Already have a Markdown site? Read [importing existing content](guides/maintenance/importing.md).
The [complete user guide](guides/index.md) is a quick reference to every topic.

## Go further

Explore [Publish and operate](guides/publishing/index.md) for team workflows,
CI, search, and hosting. Use [Maintain and migrate](guides/maintenance/index.md)
when importing content or changing published URLs.
The [user guide](guides/index.md) groups every topic by task in an expandable
sidebar. [Write content](guides/authoring/index.md#how-this-sidebar-is-organized)
shows the directory structure behind it.

## A small input, a complete site

::code-block
```text
my-docs/
  site.toml
  content/
    index.md
    guides/
      index.md
      publishing.md
  public/
    images/
      diagram.svg
```
::

The output contains rendered pages, navigation, heading links, a table of
contents, browser-based search, metadata, a sitemap, and a 404 page. Publish
the generated files to any suitable static host.

::callout{tone="info"}
**This guide is a working example.** Its pages, menu, code examples, image,
and download are ordinary generator inputs. Browse the `docs-site/` directory
in the source repository to reuse them.
::

## What you do not need

Standalone generation, preview, and local search do not require Node.js, npm,
Rust, an account, or an internet connection. Downloading an executable and
deploying to a host are separate steps. An optional npm installer is available
when a release has been published.

The generator does not execute Vue components, provide a browser editor,
optimize general image content, or deploy your website automatically.
Search requires JavaScript and HTTP(S); the pages and navigation remain
readable without JavaScript.
