---
order: 2
---
# Stable URLs and migrations

Separate navigation changes from URL changes. A page's route comes from its
source path, not its title, navigation label, or `order`.

## Prefer changes that keep links working

| Change | Effect |
| --- | --- |
| Edit front-matter `order` | Reorders siblings without changing the page route |
| Edit `navigation.title` | Changes its navigation label, not its route |
| Edit page title or H1 | Keeps the page route; changed heading text may change its anchor |
| Change a numeric filename prefix | Keeps the normalized route; source-relative links need the new filename |
| Move a file to another directory | Changes its route |
| Rename `installation.md` to `setup.md` | Changes `/installation/` to `/setup/` |
| Change the publishing URL | Changes canonical URLs and possibly the deployment prefix |

Keep a stable filename when only the page's visible wording changes.
For a frequently linked section, use an
[explicit heading ID](../authoring/structure.md#link-to-headings) before changing its text.

## Reorganize into nested sections

To turn a flat set of pages into a group:

::code-block
```text
Before:
  content/guides/installation.md
  content/guides/upgrades.md

After:
  content/guides/setup/index.md
  content/guides/setup/installation.md
  content/guides/setup/upgrades.md
```
::

1. Add the group's index page with an introduction and an `order`.
2. Move the child pages and update source-relative links to and from them.
3. Update root-relative links and any explicit `site.toml` menu destinations.
4. Check relative asset links; root-relative `/images/...` links need no change.
5. Run strict validation and inspect the nested sidebar.
6. Handle the old public URLs before deploying.

If existing URLs are important, keep the files where they are and use a custom
main-menu group instead. A header-menu hierarchy does not require the same
directory hierarchy.

## Preserve old destinations

There is no front-matter `slug`, `alias`, or redirect setting. Configure
permanent redirects at your host if it supports them. GitHub Pages does not
provide a general server-side redirect rules file.

An alternative is to leave a short page at the old source path with a normal
link to the new destination:

::code-block
```markdown
---
title: Installation has moved
description: Find the installation guide in the setup section.
navigation: false
---
# Installation has moved

Continue to [the installation guide](/guides/setup/installation/).
```
::

This is a regular published page, not an HTTP redirect, and it remains
searchable. Preserve important old heading IDs on that page if external links
use fragments; linking only to a new page cannot preserve every deep link.

Local validation cannot discover links from other websites, bookmarks, or
older releases. Keep a list of important old URLs and test them on the host.

## Move to another domain or deployment prefix

Set `url` to the final public address and rebuild the whole site. Authored
site-root links automatically receive the new prefix. Fully qualified links
to the old origin may become external links, so find and update them explicitly.

Deploy the complete output, update the host's domain/DNS settings as required,
and arrange old-origin redirects separately. Verify a nested page, image,
download, search, canonical URL, and sitemap at the new location.

For imported sites, changing `compatibility` from `true` to `false` also
changes underscore routes to hyphenated routes. Treat that switch as a URL
migration, not just a validation preference.
