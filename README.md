# SDS Site Generator

A Markdown-first documentation site generator written in Rust, with the
[SDS Lite documentation shell](https://github.com/cmu-sei/sds-lite).

Authors need one executable: no Node.js, Rust, accounts, or search service.
Sites build and run offline with bundled assets and browser-based search.
Output is static HTML, CSS, JavaScript, and JSON.

## Quick start

A local macOS executable is included in `bin/sds-site-generator`.
Preview the documentation from this directory:

```sh
./bin/sds-site-generator preview docs-site
```

Open the printed address. Save edits and refresh the browser; Ctrl+C stops
preview. Use `--port 8080` to choose another port.

Create, validate, and build your own site:

```sh
./bin/sds-site-generator new ~/work/my-docs \
  --name "My documentation" \
  --url https://example.org/docs/
./bin/sds-site-generator preview ~/work/my-docs
./bin/sds-site-generator check ~/work/my-docs --strict
./bin/sds-site-generator build ~/work/my-docs
```

`new` requires a directory that does not already exist. Other commands default
to the current directory. Set the site's publishing URL before deployment;
preview uses the same URL subdirectory locally.

Public binary and npm downloads are not available yet. Release workflows
package macOS, Linux, and Windows executables for x64 and ARM64, plus an
optional npm installer requiring Node.js 22.19+.

## Writing and publishing

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
      diagram.png
```

- Write pages in `content/`; `index.md` is a directory's landing page.
- Use one main heading, sequential heading levels, and an introductory
  paragraph or metadata description.
- Link to other Markdown files with relative paths. Put images and downloads
  in `public/` and reference them with site-root paths such as `/images/diagram.png`.
- Configure the site name, publishing URL, and optional navigation in `site.toml`.
- Publish the **contents** of `dist/` to a static host with HTTPS and directory
  index support. Preview is a development server, not a production server.

Validation failures preserve the last good build. Keep hand-authored files
outside `dist/` and avoid simultaneous builds of the same site.

Content and navigation work without JavaScript; search requires JavaScript
and HTTP(S). Arbitrary scripts and Vue components are not executed.
Accessibility depends on authored content and hosting; conformance and audit
scores are not guaranteed.

## Documentation

The [documentation site](docs-site/README.md) is the complete authoring reference
and GitHub Pages home:

- [Start here](docs-site/content/guides/start/index.md): installation and configuration.
- [Install the generator](docs-site/content/guides/start/installation.md): end-user setup for macOS, Linux, and Windows.
- [Write content](docs-site/content/guides/authoring/index.md): Markdown, metadata, components, and assets.
- [Publish and operate](docs-site/content/guides/publishing/index.md): deployment, CI, search, and hosting.
- [Maintain and migrate](docs-site/content/guides/maintenance/index.md): imports, troubleshooting, and stable URLs.

The `new` command creates a small starter. `docs-site/` is the full guide;
`example/` demonstrates legacy compatibility.

## Development

Use stable Rust to build and test:

```sh
cargo build --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

The executable is `target/release/sds-site-generator`. For the optional npm
contributor workflow, use the Node version in `.nvmrc` and npm 12.2.1:

```sh
npm ci --ignore-scripts
npm run build
npm test
npm run check:docs
npm run preview:docs
```

`npm run build` stages the current platform only; packaging requires all six
release binaries. GitHub Actions runs checks, packages tagged releases, and
deploys the documentation site to Pages.

SDS Lite assets are pinned and bundled locally. See
[provenance and update details](vendor/sds-lite/PROVENANCE.md).

## License

[SEI Design System license](LICENSE). Bundled third-party software retains
its own terms. See [licensing and publication rights](docs-site/content/guides/publishing/licensing.md)
for software terms and rights to authored content.
