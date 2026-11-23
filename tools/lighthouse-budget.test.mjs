import assert from "node:assert/strict";
import { test } from "node:test";
import { assertLighthouseBudget } from "./lighthouse-budget.mjs";

const scores = performance => ({ performance, accessibility: 100, "best-practices": 100, seo: 100 });

test("one performance outlier does not fail the median budget, regardless of order", () => {
  for (const values of [[93, 99, 100], [100, 93, 99], [99, 100, 93], [94, 95, 96]]) {
    assert.equal(assertLighthouseBudget(values.map(scores)), [...values].sort((a, b) => a - b)[1]);
  }
});

test("two low performance measurements still fail the unchanged budget", () => {
  assert.throws(() => assertLighthouseBudget([93, 94, 100].map(scores)), /Median performance score 94 below regression budget of 95/);
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
