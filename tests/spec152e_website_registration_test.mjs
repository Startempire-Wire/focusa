import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const root = new URL("../", import.meta.url);
const componentUrl = new URL("public/activation/focusa-registration.mjs", root);
const fixture = JSON.parse(await readFile(new URL("tests/fixtures/spec152e/website-registration-browser-fixtures.v1.json", root), "utf8"));
const themes = JSON.parse(await readFile(new URL("public/activation/themes.v1.json", root), "utf8"));
const registry = JSON.parse(await readFile(new URL("docs/contracts/spec152e-facade-registry.v1.json", root), "utf8"));
const source = await readFile(componentUrl, "utf8");
const page = await readFile(new URL("public/activation/page.html", root), "utf8");
const { createRegistrationModel, registrationContract, themeFor } = await import(pathToFileURL(componentUrl.pathname));

assert.equal(registrationContract.schema, "focusa.spec152e.website_registration.v1");
assert.equal(registrationContract.authority, "WPUIAI.com EDD");
assert.equal(registrationContract.role, "presenter_only");
assert.deepEqual(registrationContract.pages, fixture.flow);
assert.equal(fixture.synthetic_only, true);
assert.equal(fixture.facades.length, 4);

const registryById = new Map(registry.facades.map((facade) => [facade.facade_id, facade]));
const semanticTraces = [];
for (const facade of fixture.facades) {
  const registered = registryById.get(facade.facade_id);
  assert.ok(registered, `${facade.facade_id} is registered`);
  assert.ok(registered.exact_origins.includes(facade.origin), `${facade.facade_id} exact origin`);
  assert.ok(registered.products.includes(facade.product_code), `${facade.facade_id} fixed product`);

  const theme = themeFor(facade.facade_id);
  assert.equal(theme.name, facade.brand);
  assert.equal(theme.product, facade.product_code);
  assert.deepEqual(themes.themes[facade.facade_id], {
    name: theme.name, mark: theme.mark, accent: theme.accent, product_code: theme.product,
  });

  const model = createRegistrationModel(facade.facade_id);
  const trace = [model.page];
  for (const next of fixture.flow.slice(1)) {
    model.transition(next, {
      continuation_token: "opaqueSyntheticContinuation_01",
      masked_email: "s***@invalid.example",
      next_action: `continue_${next}`,
    });
    trace.push(model.page);
  }
  semanticTraces.push(trace);
  assert.equal(model.semanticState, 8);
  const request = model.authorityRequest("recovery", { reason: "synthetic_fixture" });
  assert.equal(request.product_code, facade.product_code);
  assert.equal(request.route, "/v1/activation/poll");
  assert.equal(request.continuation_token, "opaqueSyntheticContinuation_01");
  assert.ok(!("redirect_url" in request));
}
for (const trace of semanticTraces.slice(1)) assert.deepEqual(trace, semanticTraces[0]);

assert.throws(() => themeFor("attacker_v1"), /FACADE_ORIGIN_DENIED/);
assert.throws(() => createRegistrationModel("focusa_marketing_v1", "authority-admin"), /FACADE_ROUTE_DENIED/);
assert.throws(() => createRegistrationModel("focusa_marketing_v1").transition("success"), /ACTIVATION_TRANSITION_DENIED/);

const guarded = createRegistrationModel("focusa_marketing_v1");
for (const forbidden of [
  { product_code: "attacker_product" }, { redirect_url: "https://evil.invalid" },
  { success_url: "https://evil.invalid" }, { price: "0" }, { grants: ["all"] },
]) assert.throws(() => guarded.authorityRequest("start", forbidden), /FACADE_REQUEST_FIELD_DENIED/);
for (const forbiddenEnvelope of [
  { email: "synthetic@invalid.example" }, { license_key: "SYNTHETIC-NOT-A-KEY" },
  { credential: "synthetic-not-a-credential" }, { secret: "synthetic-not-a-secret" },
  { product_code: "attacker_product" }, { redirect_url: "https://evil.invalid" },
]) assert.throws(() => createRegistrationModel("focusa_marketing_v1").transition("verify", forbiddenEnvelope), /ACTIVATION_ENVELOPE_DENIED/);
assert.throws(() => createRegistrationModel("focusa_marketing_v1").transition("verify", {
  continuation_token: "https://evil.invalid/token",
}), /FACADE_CONTINUATION_DENIED/);
assert.throws(() => createRegistrationModel("focusa_marketing_v1").transition("verify", {
  masked_email: "synthetic@invalid.example",
}), /ACTIVATION_ENVELOPE_DENIED/);

for (const required of [
  'aria-labelledby', 'aria-describedby', 'aria-live', 'role = "status"',
  'autocomplete: "email"', 'autocomplete: "one-time-code"', 'labelNode.htmlFor',
  'textContent', 'attachShadow({ mode: "closed" })',
]) assert.ok(source.includes(required), `accessibility/security marker: ${required}`);
assert.ok(!source.includes("innerHTML"), "authority data is never rendered as HTML");
assert.ok(!source.includes("localStorage") && !source.includes("sessionStorage"), "continuation is memory-only");
assert.match(page, /<meta name="referrer" content="no-referrer">/);
assert.ok(!/[?&](?:redirect|product|price|grant)=/i.test(page), "page has no client-controlled authority query values");

const serializedFixtures = JSON.stringify(fixture) + JSON.stringify(themes);
assert.ok(!/[A-Z0-9]{4}(?:-[A-Z0-9]{4}){3}/.test(serializedFixtures), "no license-shaped evidence");
assert.ok(!/@(?:gmail|outlook|yahoo|icloud)\./i.test(serializedFixtures), "no real email evidence");

console.log(JSON.stringify({
  schema: "focusa.spec152e.website_registration_test.v1",
  facades: fixture.facades.length,
  pages_per_facade: fixture.flow.length,
  positive_checks: fixture.positive_checks.length,
  negative_checks: fixture.negative_checks.length,
  result: "passed_identical_semantics_brand_only",
}));
