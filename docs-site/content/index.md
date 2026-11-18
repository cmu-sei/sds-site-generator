# Publish documentation from Markdown

Turn Markdown, images, and downloads into a documentation website with
navigation and search, using the SEI Design System's SDS Lite interface.

You need one executable: no Node.js, npm, Rust, account, or search service.

## 1. Get the generator

[Install the generator](guides/start/installation.md) for macOS, Linux, or
Windows. Add it to your PATH so the commands below work from any directory.

## 2. Create and preview your site

In a terminal, create a site in a directory that does not already exist:

::code-block
```sh
sds-site-generator new my-docs --name "My documentation" --url "https://example.org/docs/"
cd my-docs
sds-site-generator preview .
```
::

Change the example name and URL in `site.toml` before publishing.

Open the printed address (`http://127.0.0.1:4173/docs/` for this example).
Keep preview running while you edit.

## 3. Add your content

Edit `content/index.md`, save, wait for a successful rebuild, and refresh
your browser.

| What you want to change | Where it goes | Guide |
| --- | --- | --- |
| Home page and other pages | Markdown files in `content/` | [Write Markdown](guides/authoring/markdown.md) |
| Page order and sections | Files and directories in `content/` | [Organize pages](guides/authoring/structure.md) |
| Images and downloadable files | `public/` | [Add images and downloads](guides/authoring/assets.md) |
| Site name, publishing URL, and menu | `site.toml` | [Configure your site](guides/start/configuration.md) |

Already have Markdown content? Follow [Import an existing site](guides/maintenance/importing.md).

## 4. Check, build, and publish

Stop preview with **Ctrl+C**. From your `my-docs` directory, run:

::code-block
```sh
sds-site-generator check . --strict
sds-site-generator build .
```
::

Fix any errors, then upload the **contents of `dist/`** to a static host with
HTTPS and directory index support.

See [Build and deploy](guides/publishing/deployment.md) for hosting instructions
and a pre-publication checklist.

## Find more help

- [Create your first site](guides/start/getting-started.md): preview options and daily commands.
- [User guide](guides/index.md): all topics, including CI, troubleshooting, and URL migrations.

This guide is a working example. Browse `docs-site/` in the
[source repository](https://github.com/cmu-sei/sds-site-generator/tree/main/docs-site)
to reuse its pages, configuration, and assets.

Pages and navigation work without JavaScript. Search requires JavaScript and
HTTP(S); use preview or your host, not local HTML files.
