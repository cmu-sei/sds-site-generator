import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("audit_dist", Path(__file__).with_name("audit-dist.py"))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)
BASE = "https://example.org/docs/"


class OutputAuditTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.old = Path(self.temp.name) / "old"
        self.new = Path(self.temp.name) / "new"
        for root in (self.old, self.new):
            root.mkdir()
            self.page(root, "/", '<h1 id="home">Home</h1><p>A page.</p>')
            (root / "sitemap.xml").write_text(
                f'<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>{BASE}</loc></url></urlset>'
            )
            search = root / "_site/search"
            search.mkdir(parents=True)
            (search / "docs-0.json").write_text(json.dumps([{"url": BASE + "#home"}]))
            (root / "_site/diagnostics.json").write_text("[]")

    def page(self, root, route, body):
        target = root / route.strip("/")
        target.mkdir(parents=True, exist_ok=True)
        (target / "index.html").write_text(
            f'<link rel="canonical" href="{BASE.rstrip("/") + route}"><main><article class="prose">{body}</article></main>'
        )

    def compare(self):
        return audit.compare(self.old, self.new, BASE)

    def test_detects_silent_duplicate_heading_retarget(self):
        self.page(self.old, "/", '<h1 id="home">Home</h1><h2 id="example">Example</h2><h2 id="example-1">Example</h2><h2 id="example-2">Example</h2>')
        self.page(self.new, "/", '<h1 id="home">Home</h1><h2 id="example">Example</h2><h2 id="example-2">Example</h2><h2 id="example-3">Example</h2>')
        result = self.compare()
        self.assertFalse(result["gates"]["content_ids_preserved"])
        self.assertFalse(result["gates"]["no_silent_heading_retargets"])
        self.assertEqual(result["silent_heading_retargets"][0]["old"], "example-2")

    def test_inherited_missing_image_does_not_hide_a_new_destination(self):
        body = '<h1 id="home">Home</h1><img src="missing.png">'
        self.page(self.old, "/", body)
        self.page(self.new, "/", body + '<a href="missing.png">Image</a>')
        self.assertTrue(self.compare()["gates"]["no_new_broken_targets"])
        self.page(self.new, "/", body + '<a href="different.png">Different</a>')
        self.assertFalse(self.compare()["gates"]["no_new_broken_targets"])

    def test_preexisting_error_pages_are_not_lost_documentation(self):
        (self.old / "missing").mkdir()
        (self.old / "missing/index.html").write_text("<main><h1>Page not found</h1></main>")
        self.assertTrue(self.compare()["gates"]["content_routes_preserved"])

    def test_detects_changed_literal_code(self):
        self.page(self.old, "/", '<h1 id="home">Home</h1><pre><code>if (a &lt; b) ...</code></pre>')
        self.page(self.new, "/", '<h1 id="home">Home</h1><pre><code>if (a &lt; b) &#8230;</code></pre>')
        result = self.compare()
        self.assertFalse(result["gates"]["code_content_unchanged"])
        self.assertEqual(result["code_blocks"]["reference"], result["code_blocks"]["candidate"])

    def test_detects_changed_inline_code(self):
        self.page(self.old, "/", '<h1 id="home">Home</h1><p>Call <code>pthread_mutex_trylock()</code>.</p>')
        self.page(self.new, "/", '<h1 id="home">Home</h1><p>Call <code>pthread_mutext_trylock()</code>.</p>')
        result = self.compare()
        self.assertFalse(result["gates"]["code_content_unchanged"])
        self.assertEqual(len(result["inline_code_differences"]), 1)


if __name__ == "__main__":
    unittest.main()
