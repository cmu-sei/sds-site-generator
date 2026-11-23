import assert from "node:assert/strict";
import { test } from "node:test";
import { assertLighthouseBudget } from "./lighthouse-budget.mjs";

const scores = performance => ({ performance, accessibility: 100, "best-practices": 100, seo: 100 });

test("a median at or above the performance target emits no warning, regardless of order", t => {
  const warning = t.mock.method(console, "warn", () => {});
  for (const values of [[93, 99, 100], [100, 93, 99], [99, 100, 93], [94, 95, 96]]) {
    assert.equal(assertLighthouseBudget(values.map(scores)), [...values].sort((a, b) => a - b)[1]);
  }
  assert.equal(warning.mock.callCount(), 0);
});

test("sustained low performance warns visibly without blocking publication", t => {
  const warning = t.mock.method(console, "warn", () => {});
  assert.equal(assertLighthouseBudget([93, 93, 92].map(scores)), 93);
  assert.equal(warning.mock.callCount(), 1);
  assert.match(warning.mock.calls[0].arguments[0], /^::warning title=Lighthouse performance budget::Median performance score 93 below advisory target of 95 \(measurements: 93, 93, 92\)/);
});

test("every measurement must pass accessibility, best practices, and SEO", () => {
  for (const category of ["accessibility", "best-practices", "seo"]) {
    for (let index = 0; index < 3; index++) {
      const samples = [100, 100, 100].map(scores);
      samples[index][category] = 99;
      assert.throws(() => assertLighthouseBudget(samples), { message: new RegExp(`${category} score in measurement ${index + 1}`) });
    }
  }
});

test("incomplete or missing performance measurements cannot pass", () => {
  assert.throws(() => assertLighthouseBudget([100, 100].map(scores)), /three measurements/);
  assert.throws(() => assertLighthouseBudget([scores(100), scores(undefined), scores(100)]), /Missing performance score/);
});
