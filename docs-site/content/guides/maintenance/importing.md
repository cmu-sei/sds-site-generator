---
order: 1
---
# Importing and troubleshooting

Import Markdown and assets, then use diagnostics to resolve migration issues.

## Import an existing site

1. Run `sds-site-generator new my-docs`, then `cd my-docs`.
2. Replace its starter `content/` with the imported Markdown.
3. Copy images and downloads into `public/`.
4. Set the real name and publishing URL in `site.toml`.
5. If necessary, enable [compatibility mode](../start/configuration.md#compatibility-mode)
   while repairing the import.
6. Build and review the diagnostics; then run `check . --strict`.

See [diagnostics](../start/configuration.md#review-diagnostics) and
[disabling compatibility mode](../start/configuration.md#disable-it-deliberately)
before publishing.

## Audit the CERT import

From the generator checkout, compare the import against its existing static
output, including headings, code, links, attachments, sitemap, and search:

::code-block
```sh
cargo run --locked -- build cert-example
python3 tools/audit-dist.py \
  --reference /path/to/secure-coding-standards/dist \
  --candidate cert-example/dist \
  --base-url https://cmu-sei.github.io/secure-coding-standards/ \
  --output tools/reports/cert-dist.json
node tools/audit-cert.mjs \
  --reference /path/to/secure-coding-standards/dist \
  --candidate cert-example/dist \
  --output tools/reports/cert-browser.json
```
::

The browser audit needs `tools/` dependencies and Playwright Chromium. It
manages its own loopback servers and checks all six language landing pages,
a rule, and a large analyzer table at desktop/mobile sizes, including search,
code controls, sorting, dark mode, keyboard use, and reading without JavaScript.

Both tools write JSON and fail when their gates fail. The reference build is
the inherited-defect baseline, compared by page and destination, not warning
counts. New broken destinations remain failures; accessibility findings are
compared at the same page and viewport. Review source/output differences with
content owners and retain the reference build and reports with the release.

## Legacy callouts

See [Site configuration: Legacy callouts](../start/configuration.md#legacy-callouts)
for conversion examples and the color-to-tone mapping.

## Common problems

| Message or symptom | What to do |
| --- | --- |
| Missing `site.toml` | Run from the site root, or pass its directory explicitly |
| Duplicate URL | Review [route rules](../authoring/structure.md#arrange-the-files) before renaming |
| Missing home page | Add a published `content/index.md` (a numeric prefix is also allowed) |
| Missing description | Add a [description or introductory paragraph](../authoring/markdown.md#front-matter) |
| Heading skips a level | Follow the [page heading structure](../authoring/markdown.md#a-complete-page) |
| Missing page or fragment | Check [link paths and heading IDs](../authoring/structure.md#link-to-pages) |
| Image validation fails | Check the [image requirements](../authoring/assets.md#add-an-image) |
| Unsupported component/property | Use the [supported syntax](../authoring/components.md) |
| Refusing to replace `dist/` | Move hand-authored output elsewhere |
| Preview shows old content | Read the terminal for a failed rebuild; refresh after a successful one |
| Port already in use | Choose a different `--port` |
| Pages load without styles | Check `url`, the deployment prefix, and which directory was uploaded |
| Search fails | Follow [search diagnostics](../publishing/search-and-hosting.md#diagnose-search-failures) |

Use preview or configure your local server to match the `url` deployment prefix.

## Getting help

Report generator bugs in the
[issue tracker](https://github.com/cmu-sei/sds-site-generator/issues).
For hosting/account issues, contact your host. For generator bugs, include:

::code-block
```text
Generator version: (output of sds-site-generator --version)
Operating system and architecture:
Installation method: standalone archive / npm / source build
Exact command and working directory:
Expected behavior:
Actual behavior and full terminal error:
Smallest site.toml and Markdown example that reproduces it:
```
::

For preview or hosting issues, add the URL, deployment prefix, browser errors,
and relevant diagnostics. Use a minimal example without private content or
credentials. Report sensitive security issues through a private channel.

## Review beyond validation

Use the [pre-publication checklist](../publishing/deployment.md#before-sharing-the-url)
for checks beyond automated validation.
