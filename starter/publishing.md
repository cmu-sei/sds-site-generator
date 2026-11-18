# Publishing

Publish your site to a static host with HTTPS and directory index support.

## Set your address

In `site.toml`, set `url` to your public address, including any subdirectory:
`https://example.org/my-docs/`.

## Check and build

Stop preview with Ctrl+C, then run:

::code-block
```sh
sds-site-generator check .
sds-site-generator build .
```
::

Fix any errors, then upload the **contents of `dist/`** to your host.
For GitHub Pages, use GitHub Actions with `dist/` as the Pages artifact.

## Before sharing

- Check your content for accuracy and clarity.
- Check nested pages, links, images, downloads, search, and a nonexistent URL.
- Test the deployed site on a phone, with a keyboard, and with a screen reader.
- Run automated checks such as Lighthouse alongside these manual tests.
