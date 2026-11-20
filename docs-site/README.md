# Documentation site

The end-user guide, maintained example site, and this repository's GitHub Pages site.

From the repository root:

```sh
npm ci
npm run check
npm run dev
npm run build
```

Preview at the printed `/sds-site-generator/` address; save and refresh after edits.
Stop the development server before building separately. To reuse the guide, copy it without
`dist/` and edit `site.toml`, or use `new` for a smaller starter.

Reproduce the Pages output with a deployment URL applied to a temporary copy:

```sh
npm ci --ignore-scripts
npm run build:generator
node scripts/package.mjs pages https://your-org.github.io/your-repository/
```

Only the contents of `pages-dist/` are deployed. See
[the publishing guide](content/guides/publishing/deployment.md) for author-facing instructions.
