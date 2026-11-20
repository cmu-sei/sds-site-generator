# SDS Site Generator

A Markdown-first documentation site generator written in Rust, with the
[SDS Lite documentation shell](https://github.com/cmu-sei/sds-lite).

One executable builds a static site with bundled assets and browser search.
No Node.js, Rust, account, or external search service required.

## Quick start

1. [Install the generator](docs-site/content/guides/start/installation.md) for your platform.
2. [Create and preview your site](docs-site/content/guides/start/getting-started.md).
3. [Write content](docs-site/content/guides/authoring/index.md), then
   [build and deploy](docs-site/content/guides/publishing/deployment.md).

See the [command reference](docs-site/content/guides/start/getting-started.md#daily-commands)
for daily use, or [npm installation](docs-site/content/guides/start/installation.md#optional-npm-installation)
for shared projects.

## Documentation

The [documentation site](docs-site/README.md) covers:

- [Start here](docs-site/content/guides/start/index.md): installation and configuration.
- [Write content](docs-site/content/guides/authoring/index.md): Markdown, metadata, components, and assets.
- [Publish and operate](docs-site/content/guides/publishing/index.md): deployment, CI, search, and hosting.
- [Maintain and migrate](docs-site/content/guides/maintenance/index.md): imports, troubleshooting, and stable URLs.

The `new` command creates a small starter. `docs-site/` is the full guide and
the maintained example site.

## Development

Use [stable Rust for source builds](docs-site/content/guides/start/installation.md#build-from-source).
Run the development checks with:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

For the npm contributor workflow, install stable Rust 1.89 or newer and use the Node version
in `.nvmrc` and npm 12.2.1. After cloning, run:

```sh
npm ci
npm run dev
```

Unqualified commands target the documentation site or the core test suites.
Specialized commands use `verb:target`; there are no duplicate aliases.

| Command | Purpose |
| --- | --- |
| `npm run dev` | Build the generator, then serve and watch the documentation site |
| `npm run build` | Build the generator, then generate `docs-site/dist/` |
| `npm run check` | Build the generator, then strictly validate the documentation site |
| `npm test` | Build the generator, then run generator and package tests |
| `npm run build:generator` | Build and stage only the current platform's generator executable |
| `npm run test:generator` | Run the Rust test suites |
| `npm run test:package` | Run npm packaging and launcher tests against staged executables |
| `npm run check:release` | Validate the complete six-platform release bundle |

`prepack` is an automatic npm lifecycle hook, not another developer command.
It runs `check:release` before packing. Package tests do not rebuild executables,
so release checks retain the target-specific binaries produced by CI.
The development server watches documentation files; restart it after changing
the generator's Rust source.

`build:generator` and `test:generator` find Cargo on `PATH`, in the standard
Cargo installation directory, or in an installed rustup toolchain. They set the subprocess's
`PATH` and toolchain library search paths as needed without changing your shell
configuration. Custom `CARGO_HOME` and `RUSTUP_HOME` locations and
`RUSTUP_TOOLCHAIN` selections are supported;
without Rust proxies, it uses rustup's configured default or a single installed
stable toolchain. Missing or ambiguous installations produce setup instructions.
Node.js, npm, and Rust still need to be installed; the build does not install them.

Developer builds support macOS, Linux, and Windows on x64 or ARM64. They also
need a native compiler/linker: Xcode Command Line Tools on macOS, a C toolchain
such as GCC on Linux, or Visual Studio C++ Build Tools and the Windows SDK for
MSVC Rust on Windows. The first install/build needs network access unless npm
and Cargo dependencies are already cached. Nonstandard Rust locations must be
exposed through `PATH`, `CARGO_HOME`, or `RUSTUP_HOME`; arbitrary installation
directories cannot be discovered automatically. Custom Cargo output directories
or cross-compilation targets are not supported by the generator build command,
which stages `target/release/` for the current Node.js platform and architecture.

The generator build stages the current platform only; packaging requires all six release
binaries. GitHub Actions runs `npm run build`, verifies Rust discovery with
Cargo removed from `PATH`, and runs the package tests on all six supported
OS/architecture combinations. It also packages tagged releases and deploys the
documentation site to Pages.

Browser and asset checks live in `tools/`. CI requires the search, component,
highlighting, and starter-site browser audits. After installing its
dependencies with `npm ci --prefix tools`, use
`npm --prefix tools run <command>`:

| Command | Purpose |
| --- | --- |
| `build:highlight` | Rebuild bundled syntax highlighting |
| `test:search` | Test search queries against a generated index |
| `test:search-ui` | Test search interaction in a browser |
| `test:components` | Test interactive documentation components |
| `test:highlight` | Test syntax highlighting |
| `audit:site` | Audit a served site's accessibility and performance |
| `audit:cert` | Audit the retained CERT documentation site |

Browser checks require Chromium; served-site checks also require a running
site configured with `SITE_URL` where appropriate.

SDS Lite assets are pinned and bundled locally. See
[provenance and update details](vendor/sds-lite/PROVENANCE.md).

## License

[SEI Design System license](LICENSE). Bundled third-party software retains
its own terms. See [licensing and publication rights](docs-site/content/guides/publishing/licensing.md)
for software terms and rights to authored content.
