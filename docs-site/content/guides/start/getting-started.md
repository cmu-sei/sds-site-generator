---
order: 2
navigation:
  title: Create your first site
---
# Getting started

Create a site, edit Markdown, and preview changes before publishing.

## Get the generator

Follow [Installation](installation.md) for your platform. Commands below
assume the executable is on PATH; otherwise use its full path.

::code-block
```sh
sds-site-generator --version
```
::

## Create your site

::code-block
```sh
sds-site-generator new my-docs --name "My documentation" --url "https://example.org/docs/"
cd my-docs
sds-site-generator preview .
```
::

`new` requires a directory that does not exist. It creates `site.toml`,
starter pages in `content/`, an empty `public/`, a README, and ignore rules.

Open the printed address: `http://127.0.0.1:4173/docs/` for this example.
Preview uses the publishing URL's subdirectory. To change the port:

::code-block
```sh
sds-site-generator preview . --port 8080
```
::

Edit `content/index.md`, save, wait for a successful rebuild, and refresh the
browser. Preview watches `content/`, `public/`, and `site.toml`; browser reload
is manual. Stop preview with Ctrl+C before running a separate build.

## Daily commands

| Command | Result |
| --- | --- |
| `sds-site-generator new DIRECTORY --name NAME --url URL` | Create a site in a new directory |
| `sds-site-generator preview [DIRECTORY] --port 4173` | Build, serve locally, and watch for edits |
| `sds-site-generator check [DIRECTORY]` | Validate without writing output |
| `sds-site-generator check [DIRECTORY] --strict` | Also fail on compatibility warnings |
| `sds-site-generator build [DIRECTORY]` | Generate `dist/` inside the site directory |

Square brackets above mean optional arguments; do not type the brackets.
Omitted directories default to the current directory except for `new`.
Quote paths and names containing spaces. Errors produce a nonzero exit status.
Use `sds-site-generator --help` for command help.

## Optional npm installation

Prefer npm? Follow [npm installation](installation.md#optional-npm-installation)
for authentication and shared-project setup.
