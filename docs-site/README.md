# Documentation site

This end-user guide is the GitHub Pages home and a working example of Markdown,
navigation, components, and assets.

From the repository root:

```sh
cargo run -- check docs-site --strict
cargo run -- preview docs-site
cargo run -- build docs-site
```

Open the printed address under `/sds-site-generator/`. Save and refresh to see
changes; stop preview before building separately. To reuse this guide, copy it
without `dist/` and edit `site.toml`. The `new` command creates a smaller starter.

The Pages workflow sets the deployment URL in a temporary copy, preserving the
checked-in configuration. After `npm run build`, reproduce its output with:

```sh
node scripts/package.mjs pages https://your-org.github.io/your-repository/
```

Only the contents of `pages-dist/` are deployed. See
[the publishing guide](content/guides/publishing/deployment.md) for author-facing instructions.
