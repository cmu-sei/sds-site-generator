---
order: 2
---
# Stable URLs and migrations

A page's route comes from its source path, not its title, navigation label, or `order`.

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

Keep filenames stable when editing wording. Use an
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

To keep existing URLs, leave files in place and use
[custom main navigation](../start/configuration.md#main-navigation) instead.

## Preserve old destinations

Configure permanent redirects at your host; there is no `slug`, `alias`, or
redirect setting. GitHub Pages has no general server-side redirect rules file.

Alternatively, leave a page at the old source path linking to the new one:

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

This remains a searchable page, not an HTTP redirect. Preserve old heading IDs
for inbound fragment links. Local validation cannot check other sites'
links or bookmarks; test important old URLs on the host.

## Move to another domain or deployment prefix

Set `url` to the new address and rebuild. Site-root links receive the new prefix;
update fully qualified old-origin links explicitly.

Deploy the complete output, update domain/DNS settings, and arrange old-origin
redirects. Test a nested page, image, download, search, canonical URL, and sitemap.

For imported sites, also review
[Compatibility mode: Disable it deliberately](../start/configuration.md#disable-it-deliberately)
before changing that setting.
