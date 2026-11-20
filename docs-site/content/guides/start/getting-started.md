---
order: 2
navigation:
  title: Create your first site
---
# Getting started

Create a site and preview your first edit.

## Get the generator

Follow [Installation](installation.md) for standalone or npm setup.

## Create your site

::code-block
```sh
sds-site-generator new my-docs --name "My documentation" --url "https://example.org/docs/"
cd my-docs
sds-site-generator preview .
```
::

`new` requires a directory that does not exist. It creates `site.toml`, starter
pages in `content/`, `public/`, a README, and ignore rules.

Open the printed address: `http://127.0.0.1:4173/docs/` for this example.
Preview uses the publishing URL's subdirectory. To change the port:

::code-block
```sh
sds-site-generator preview . --port 8080
```
::

Edit `content/index.md`, save, and refresh after a successful rebuild.
Preview watches `content/`, `public/`, and `site.toml`. Stop it with Ctrl+C
before running a separate build.

## Daily commands

| Command | Result |
| --- | --- |
| `sds-site-generator new DIRECTORY --name NAME --url URL` | Create a site in a new directory |
| `sds-site-generator preview [DIRECTORY] --port 4173` | Build, serve locally, and watch for edits |
| `sds-site-generator check [DIRECTORY]` | Validate without changing files |
| `sds-site-generator check [DIRECTORY] --strict` | Also fail on compatibility warnings |
| `sds-site-generator build [DIRECTORY]` | Generate `dist/` inside the site directory |

Brackets mark optional arguments; omit the brackets when typing. Directories
default to the current directory except for `new`. Quote paths and names with
spaces. Errors return a nonzero exit status. Use `sds-site-generator --help`
for command help.

## Next steps

[Configure your site](configuration.md), [write content](../authoring/index.md),
then [build and deploy](../publishing/deployment.md).
