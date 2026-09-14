import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const root = new URL("../", import.meta.url);
const componentUrl = new URL("public/activation/focusa-facade-security.mjs", root);
const registry = JSON.parse(await readFile(new URL("docs/contracts/spec152e-facade-registry.v1.json", root), "utf8"));
const source = await readFile(componentUrl, "utf8");
const { createFacadeBrowserSecurity, facadeBrowserSecurityContract } = await import(pathToFileURL(componentUrl.pathname));

assert.equal(facadeBrowserSecurityContract.schema, "focusa.spec152e.facade_browser_security.v1");
assert.equal(facadeBrowserSecurityContract.cookieAuthority, "server_http_only_host_only_same_site_strict");
assert.equal(facadeBrowserSecurityContract.storageAuthority, "forbidden");

const facade = registry.facades.find((entry) => entry.facade_id === "focusa_install_v1");
assert.ok(facade, "synthetic test facade is registered");
const origin = facade.exact_origins[0];
const productCode = facade.products[0];
const csrfToken = "csrfSyntheticBrowserToken_00000001";
const browser = createFacadeBrowserSecurity({
  facadeId: facade.facade_id,
  origin,
  productCode,
  csrfToken,
  callbacks: facade.callbacks,
});

assert.equal(browser.resolveRedirect("success"), `${origin}${facade.callbacks.success}`);
const post = browser.request("activation_start", {
  request_id: "req_synthetic_browser_01",
  idempotency_key: "idem_synthetic_browser_01",
});
assert.equal(post.url, `${origin}/v1/activation/start`);
assert.equal(post.init.method, "POST");
assert.equal(post.init.credentials, "same-origin");
assert.equal(post.init.redirect, "error");
assert.equal(post.init.cache, "no-store");
assert.equal(post.init.headers["X-CSRF-Token"], csrfToken);
assert.deepEqual(JSON.parse(post.init.body), {
  request_id: "req_synthetic_browser_01",
  idempotency_key: "idem_synthetic_browser_01",
  facade_id: facade.facade_id,
  product_code: productCode,
});

const get = browser.request("activation_offers");
assert.equal(get.url, `${origin}/v1/activation/offers`);
assert.equal(get.init.method, "GET");
assert.equal(get.init.body, undefined);
assert.equal(get.init.headers["X-CSRF-Token"], undefined);

for (const spoofedOrigin of [
  "http://install.focusa.dev",
  "https://install.focusa.dev.evil.invalid",
  "https://install.focusa.dev/extra",
  "https://user@install.focusa.dev",
  "https://*.focusa.dev",
]) {
  assert.throws(() => createFacadeBrowserSecurity({
    facadeId: facade.facade_id, origin: spoofedOrigin, productCode, csrfToken, callbacks: facade.callbacks,
  }), /FACADE_ORIGIN_DENIED/, `spoofed origin denied: ${spoofedOrigin}`);
}

for (const invalidCallbacks of [
  { success: "https://evil.invalid/callback" },
  { success: "//evil.invalid/callback" },
  { success: "/callback?redirect=https://evil.invalid" },
  { success: "/callback#fragment" },
  { "https://evil.invalid": "/callback" },
  { success: "../authority" },
]) {
  assert.throws(() => createFacadeBrowserSecurity({
    facadeId: facade.facade_id, origin, productCode, csrfToken, callbacks: invalidCallbacks,
  }), /FACADE_REDIRECT_DENIED/, "unsafe callback registry denied");
}
assert.throws(() => browser.resolveRedirect("https://evil.invalid/callback"), /FACADE_REDIRECT_DENIED/);
assert.throws(() => browser.resolveRedirect("unknown"), /FACADE_REDIRECT_DENIED/);

for (const forbiddenBody of [
  { callback_url: "https://evil.invalid" },
  { redirect_url: "https://evil.invalid" },
  { success_url: "https://evil.invalid" },
  { cancel_url: "https://evil.invalid" },
  { edd_download_id: 453 },
  { edd_price_id: 0 },
  { price: "0" },
  { features: ["all"] },
  { grants: ["all"] },
  { limits: { nodes: 999 } },
  { license_key: "SYNTHETIC-MUST-NOT-PASS" },
  { credential: "synthetic-must-not-pass" },
  { secret: "synthetic-must-not-pass" },
  { sender_email: "synthetic@invalid.example" },
  { product_code: "attacker_product_v1" },
]) {
  assert.throws(() => browser.request("activation_start", forbiddenBody), /FACADE_REQUEST_DENIED/, "client authority field denied");
}

assert.throws(() => browser.request("authority_issue"), /FACADE_METHOD_DENIED/);
assert.throws(() => browser.request("activation_start", "invalid"), /FACADE_REQUEST_DENIED/);
assert.throws(() => browser.request("activation_start", []), /FACADE_REQUEST_DENIED/);
assert.throws(() => browser.request("activation_offers", { product_code: productCode }), /FACADE_METHOD_DENIED/);
assert.throws(() => createFacadeBrowserSecurity({
  facadeId: facade.facade_id, origin, productCode, csrfToken: "short", callbacks: facade.callbacks,
}), /FACADE_CSRF_DENIED/);
assert.throws(() => createFacadeBrowserSecurity({
  facadeId: "attacker facade", origin, productCode, csrfToken, callbacks: facade.callbacks,
}), /FACADE_ORIGIN_DENIED/);

assert.ok(!source.includes("localStorage") && !source.includes("sessionStorage"), "session and CSRF authority are never persisted by browser code");
assert.ok(!source.includes("document.cookie"), "browser code cannot read or mint the presenter session");
assert.ok(!source.includes("redirect: \"follow\""), "browser requests never follow redirects");
assert.ok(!/@(?:gmail|outlook|yahoo|icloud)\./i.test(source), "source contains no real-email evidence");

console.log(JSON.stringify({
  schema: "focusa.spec152e.facade_browser_security_adversarial_matrix.v1",
  positive_checks: 17,
  negative_checks: 5 + 6 + 2 + 15 + 6,
  result: "passed_fail_closed",
}));
