#!/usr/bin/env python3
"""Compare complete static documentation outputs without fetching external URLs."""

import argparse
import collections
import difflib
import gzip
import hashlib
import json
import re
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit
import xml.etree.ElementTree as ET


def text(parts):
    return " ".join("".join(parts).split())


def literal(parts):
    return "\n".join(line.rstrip() for line in "".join(parts).replace("\r\n", "\n").strip().splitlines())


def comparable(value):
    return re.sub(r"\s+", "", value)


class Document(HTMLParser):
    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.ids = []
        self.all_ids = []
        self.headings = []
        self.codes = []
        self.inline = []
        self.tables = []
        self.links = []
        self.resources = []
        self.prose = []
        self.main = []
        self.canonical = []
        self.feed(source)
        self.not_found = "Page not found" in text(self.main) and not self.prose

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        active = tag == "article" or "prose" in attrs.get("class", "").split() or any(v for _, v in self.stack)
        identifier = attrs.get("id") or (attrs.get("name") if tag == "a" else None)
        if identifier:
            self.all_ids.append(identifier)
            if active:
                self.ids.append(identifier)
        if active and re.fullmatch("h[1-6]", tag):
            self.headings.append({"id": attrs.get("id"), "parts": [], "level": tag})
        if active and tag == "pre":
            self.codes.append([])
        if active and tag == "code" and not any(t == "pre" for t, _ in self.stack):
            self.inline.append([])
        if active and tag == "table":
            self.tables.append([])
        if tag == "a" and attrs.get("href"):
            self.links.append({"target": attrs["href"], "article": active})
        if tag in ("img", "script", "source", "iframe") and attrs.get("src"):
            self.resources.append({"target": attrs["src"], "type": tag})
        if tag == "link":
            if attrs.get("rel") == "canonical":
                self.canonical.append(attrs.get("href"))
            if attrs.get("rel") in ("stylesheet", "icon", "apple-touch-icon", "modulepreload", "preload"):
                self.resources.append({"target": attrs.get("href", ""), "type": "link"})
        if tag not in ("area", "base", "br", "col", "embed", "hr", "img", "input",
                       "link", "meta", "param", "source", "track", "wbr"):
            self.stack.append((tag, active))

    def handle_endtag(self, tag):
        for i in range(len(self.stack) - 1, -1, -1):
            if self.stack[i][0] == tag:
                del self.stack[i:]
                return

    def handle_data(self, data):
        tags = [t for t, _ in self.stack]
        if "main" in tags:
            self.main.append(data)
        if not any(v for _, v in self.stack):
            return
        if not any(t in tags for t in ("button", "script", "style", "nav", "output", "template")):
            self.prose.append(data)
        if "pre" in tags and self.codes:
            self.codes[-1].append(data)
        if "code" in tags and "pre" not in tags and self.inline:
            self.inline[-1].append(data)
        if "table" in tags and self.tables:
            self.tables[-1].append(data)
        if self.headings and any(re.fullmatch("h[1-6]", tag) for tag in tags):
            self.headings[-1]["parts"].append(data)


def read_site(root):
    pages = {}
    for path in sorted(root.rglob("index.html")):
        relative = path.relative_to(root)
        if relative.parts[0] in ("attachments", "_site", "_nuxt"):
            continue
        pages["/" + relative.as_posix().removesuffix("index.html")] = Document(path.read_text())
    if "/" not in pages:
        raise ValueError(f"{root} has no home page")
    return pages


def local_target(base, route, target):
    resolved = urlsplit(urljoin(base.rstrip("/") + route, target))
    origin = urlsplit(base)
    if resolved.scheme not in ("http", "https") or resolved.netloc != origin.netloc:
        return None
    prefix = origin.path.rstrip("/") + "/"
    path = unquote(resolved.path)
    if not path.startswith(prefix):
        return {"outside_base": True, "path": path, "fragment": unquote(resolved.fragment)}
    return {"outside_base": False, "path": "/" + path[len(prefix):], "fragment": unquote(resolved.fragment)}


def audit(root, pages, base):
    issues = []
    for route, doc in pages.items():
        for identifier, count in collections.Counter(doc.all_ids).items():
            if count > 1:
                issues.append({"source": route, "kind": "duplicate_id", "target": identifier})
        for item in doc.links + doc.resources:
            target = local_target(base, route, item["target"])
            if target is None:
                continue
            issue = {"source": route, "kind": "missing_resource" if "type" in item else "missing_page",
                     "target": target["path"], "fragment": target["fragment"]}
            if target["outside_base"]:
                issue["kind"] = "outside_base"
                issues.append(issue)
                continue
            path = target["path"]
            key = path if path in pages else path.rstrip("/") + "/"
            if "type" not in item and key in pages and not pages[key].not_found:
                if target["fragment"] and target["fragment"] not in pages[key].all_ids:
                    issue["kind"] = "missing_fragment"
                    issues.append(issue)
            elif key in pages and pages[key].not_found:
                issues.append(issue)
            elif not (root / path.lstrip("/")).is_file():
                issues.append(issue)
        if not doc.not_found and route != "/search/":
            expected = base.rstrip("/") + route
            if len(doc.canonical) != 1 or doc.canonical[0].rstrip("/") != expected.rstrip("/"):
                issues.append({"source": route, "kind": "canonical", "target": str(doc.canonical)})
    return issues


def issue_key(issue):
    kind = "missing_target" if issue["kind"] in ("missing_resource", "missing_page") else issue["kind"]
    return (issue["source"], kind, issue["target"].rstrip("/"), issue.get("fragment", ""))


def compare(reference, candidate, base):
    old, new = read_site(reference), read_site(candidate)
    old_content = {r: d for r, d in old.items() if not d.not_found and r != "/search/"}
    new_content = {r: d for r, d in new.items() if not d.not_found and r != "/search/"}
    report = {
        "audited_at": datetime.now(timezone.utc).isoformat(),
        "reference": str(reference), "candidate": str(candidate), "base_url": base,
        "content_pages": {"reference": len(old_content), "candidate": len(new_content)},
        "baseline_not_found_routes": [r for r, d in old.items() if d.not_found],
        "removed_content_routes": sorted(old_content.keys() - new_content.keys()),
        "added_content_routes": sorted(new_content.keys() - old_content.keys()),
        "lost_ids": [], "heading_changes": [], "silent_heading_retargets": [],
        "inherited_ambiguous_headings": [],
        "code_differences": [], "inline_code_differences": [], "table_differences": [],
        "prose_differences": [], "asset_differences": [], "invalid_gzip": [],
        "code_blocks": {"reference": 0, "candidate": 0},
    }
    for route in sorted(old_content.keys() & new_content.keys()):
        a, b = old_content[route], new_content[route]
        lost = sorted(set(a.ids) - set(b.ids))
        if lost:
            report["lost_ids"].append({"route": route, "ids": lost})
        aa, bb = [text(h["parts"]) for h in a.headings], [text(h["parts"]) for h in b.headings]
        for match in difflib.SequenceMatcher(None, aa, bb, autojunk=False).get_matching_blocks():
            for i in range(match.size):
                ah, bh = a.headings[match.a+i], b.headings[match.b+i]
                if ah["id"] != bh["id"]:
                    item = {"route": route, "heading": text(ah["parts"]), "old": ah["id"], "new": bh["id"]}
                    if a.ids.count(ah["id"]) > 1:
                        report["inherited_ambiguous_headings"].append(item)
                        continue
                    report["heading_changes"].append(item)
                    if ah["id"] in [h["id"] for h in b.headings]:
                        report["silent_heading_retargets"].append(item)
        report["code_blocks"]["reference"] += len(a.codes)
        report["code_blocks"]["candidate"] += len(b.codes)
        for field, key in (("codes", "code_differences"), ("inline", "inline_code_differences"),
                           ("tables", "table_differences")):
            before, after = [literal(x) for x in getattr(a, field)], [literal(x) for x in getattr(b, field)]
            if [comparable(x) for x in before] != [comparable(x) for x in after]:
                report[key].append({"route": route, "old_count": len(before), "new_count": len(after),
                                    "changes": [{"index": i, "old": x, "new": y}
                                                for i, (x, y) in enumerate(zip(before, after))
                                                if comparable(x) != comparable(y)]})
        if comparable(text(a.prose)) != comparable(text(b.prose)):
            report["prose_differences"].append({"route": route, "old_chars": len(text(a.prose)),
                                               "new_chars": len(text(b.prose))})
    for path in sorted((reference / "attachments").rglob("*")):
        if path.is_file():
            relative = path.relative_to(reference)
            target = candidate / relative
            if not target.is_file():
                report["asset_differences"].append({"path": relative.as_posix(), "status": "missing"})
            elif hashlib.sha256(path.read_bytes()).digest() != hashlib.sha256(target.read_bytes()).digest():
                report["asset_differences"].append({"path": relative.as_posix(), "status": "modified"})
    for path in candidate.rglob("*.gz"):
        original = path.with_suffix("")
        if original.is_file() and gzip.decompress(path.read_bytes()) != original.read_bytes():
            report["invalid_gzip"].append(str(path.relative_to(candidate)))
    old_issues, new_issues = audit(reference, old, base), audit(candidate, new, base)
    old_keys = {issue_key(i) for i in old_issues}
    report["issues"] = {"reference": old_issues, "candidate": new_issues,
                        "introduced": [i for i in new_issues if issue_key(i) not in old_keys]}
    report["sizes"] = {}
    for label, root in (("reference", reference), ("candidate", candidate)):
        files = [p for p in root.rglob("*") if p.is_file()]
        report["sizes"][label] = {"files": len(files), "bytes": sum(p.stat().st_size for p in files)}
    report["sitemap"] = {}
    for label, root in (("reference", reference), ("candidate", candidate)):
        urls = [e.text for e in ET.parse(root / "sitemap.xml").iter() if e.tag.endswith("}loc")]
        report["sitemap"][label] = {"count": len(urls), "unique": len(set(urls)),
                                   "wrong_base": [u for u in urls if not u.startswith(base)]}
    documents = [d for p in (candidate / "_site/search").glob("docs-*.json")
                 for d in json.loads(p.read_text())]
    search_errors = []
    search_pages = set()
    for document in documents:
        target = local_target(base, "/", document["url"])
        if target is None or target["outside_base"]:
            search_errors.append(document["url"])
            continue
        route = target["path"]
        search_pages.add(route)
        if route not in new_content or (target["fragment"] and target["fragment"] not in new_content[route].ids):
            search_errors.append(document["url"])
    report["search"] = {"sections": len(documents), "pages": len(search_pages),
                         "missing_pages": sorted(new_content.keys() - search_pages), "invalid_urls": search_errors}
    diagnostics = json.loads((candidate / "_site/diagnostics.json").read_text())
    report["diagnostics"] = {"count": len(diagnostics), "items": diagnostics}
    report["gates"] = {
        "content_routes_preserved": not report["removed_content_routes"],
        "content_ids_preserved": not report["lost_ids"],
        "heading_destinations_preserved": not report["heading_changes"],
        "no_silent_heading_retargets": not report["silent_heading_retargets"],
        "code_content_unchanged": not (report["code_differences"] or report["inline_code_differences"]),
        "no_new_broken_targets": not report["issues"]["introduced"],
        "attachments_retained": not any(i["status"] == "missing" for i in report["asset_differences"]),
        "gzip_integrity": not report["invalid_gzip"],
        "search_coverage": not search_errors and not report["search"]["missing_pages"],
        "sitemap_valid": not report["sitemap"]["candidate"]["wrong_base"]
                         and report["sitemap"]["candidate"]["count"] == len(new_content)
                         and report["sitemap"]["candidate"]["count"] == report["sitemap"]["candidate"]["unique"],
    }
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = compare(args.reference.resolve(), args.candidate.resolve(), args.base_url)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("audited_at", "content_pages", "code_blocks", "gates")}, indent=2))
    print(f"Full evidence: {args.output}")
    raise SystemExit(0 if all(result["gates"].values()) else 1)
