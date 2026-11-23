import assert from "node:assert/strict";

export function assertLighthouseBudget(samples) {
  assert.equal(samples.length, 3, "Lighthouse requires three measurements per route");
  for (const [index, scores] of samples.entries()) {
    for (const category of ["accessibility", "best-practices", "seo"]) {
      assert.equal(scores[category], 100, `${category} score in measurement ${index + 1}`);
    }
    assert.ok(Number.isFinite(scores.performance), `Missing performance score in measurement ${index + 1}`);
  }
  const median = samples.map(scores => scores.performance).sort((a, b) => a - b)[1];
  if (median < 95) {
    console.warn(`::warning title=Lighthouse performance budget::Median performance score ${median} below advisory target of 95 (measurements: ${samples.map(scores => scores.performance).join(", ")})`);
  }
  return median;
}
