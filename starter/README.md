# Your documentation website

Edit Markdown files in `content/`. Put images and downloads in `public/`.
Set your website name and publishing URL in `site.toml`.
Both YAML (`---`) and TOML (`+++`) front matter are supported.

```sh
sds-site-generator preview .
sds-site-generator check .
sds-site-generator build .
```

Preview prints a local address and rebuilds when you save a file. Refresh your
browser after a successful rebuild. Press Ctrl+C to stop the server.

For imported content, `compatibility = true` in `site.toml` enables visible
warnings and a complete `dist/_site/diagnostics.json` report. Run
`sds-site-generator check --strict .` to fail on remaining warnings. Duplicate
page URLs always require fixing the source filenames.

Publish the contents of `dist/` to a static website host. Keep that directory
for generated output only; the generator replaces it on a successful build.

This website was created using the SEI Design System. Its license and
acknowledgments are included in every generated website.
