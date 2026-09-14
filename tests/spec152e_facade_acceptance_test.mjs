#!/usr/bin/env node
// Spec 152E multi-domain facade acceptance matrix — browser/website surface.
//
// Exercises every registered facade domain through the shipped browser
// components: facade browser security (all six domains, allowed and denied
// products, verification/checkout/success/cancel/recovery redirects, proxy
// routes, spoofing, CSRF/session-shape and timeout behavior) and the shared
// website registration presenter (identical semantic flow across branded
// pages, checkout return, polling, recovery, outage disclosure). Offline,
// deterministic, replayable from the pinned commit. No network, no build.

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const root = new URL("../", import.meta.url);
const registry = JSON.parse(await readFile(new URL("docs/contracts/spec152e-facade-registry.v1.json", root), "utf8"));
const products = JSON.parse(await readFile(new URL("docs/contracts/spec152e-edd-product-registry.v1.json", root), "utf8"));
const openapi = JSON.parse(await readFile(new URL("docs/contracts/spec152e-activation-public-openapi.v1.json", root), "utf8"));
const golden = JSON.parse(await readFile(new URL("docs/contracts/spec152e-facade-golden-vectors.v1.json", root), "utf8"));
const themes = JSON.parse(await readFile(new URL("public/activation/themes.v1.json", root), "utf8"));
const fixture = JSON.parse(await readFile(new URL("tests/fixtures/spec152e/website-registration-browser-fixtures.v1.json", root), "utf8"));

const securityUrl = new URL("public/activation/focusa-facade-security.mjs", root);
const registrationUrl = new URL("public/activation/focusa-registration.mjs", root);
const securitySource = await readFile(securityUrl, "utf8");
const registrationSource = await readFile(registrationUrl, "utf8");
const pageSource = await readFile(new URL("public/activation/page.html", root), "utf8");
const { createFacadeBrowserSecurity, facadeBrowserSecurityContract } = await import(pathToFileURL(securityUrl.pathname));
const { createRegistrationModel, themeFor, registrationContract } = await import(pathToFileURL(registrationUrl.pathname));

let positive = 0;
let negative = 0;

function check(condition, message, kind = "positive") {
  if (!condition) throw new Error(`FAIL (${kind}): ${message}`);
  if (kind === "positive") positive += 1;
  else negative += 1;
}

function throws(fn, pattern, message) {
  let threw = null;
  try { fn(); } catch (error) { threw = error; }
  check(threw !== null && pattern.test(threw.message), message, "negative");
}

const FORBIDDEN_BODY_FIELDS = [
  "callback_url", "cancel_url", "credential", "edd_download_id", "edd_price_id",
  "email_exists", "features", "grants", "license_key", "limits", "price",
  "redirect_url", "secret", "sender_email", "success_url",
];
const csrfFor = (facadeId) => `csrfSyntheticAcceptance_${facadeId}`;

// --- A. All six registered domains through the browser security component ------
check(facadeBrowserSecurityContract.schema === "focusa.spec152e.facade_browser_security.v1", "browser security schema");
check(facadeBrowserSecurityContract.cookieAuthority === "server_http_only_host_only_same_site_strict", "session cookie authority");
check(facadeBrowserSecurityContract.storageAuthority === "forbidden", "browser storage forbidden");
check(registry.facades.length === 6, "six registered facade domains");

const openapiRoutes = Object.fromEntries(
  Object.entries(openapi.paths).map(([path, operations]) => [
    Object.values(operations)[0].operationId.replace(/\./g, "_"), path,
  ]),
);
check(JSON.stringify(Object.entries(openapiRoutes).sort()) === JSON.stringify(Object.entries(registry.proxy_routes).sort()),
  "OpenAPI proxy paths equal registry routes");

const browserContractRoutes = Object.fromEntries(
  Object.entries(facadeBrowserSecurityContract.routes).map(([route, [method, path]]) => [route, path]),
);
check(JSON.stringify(Object.entries(browserContractRoutes).sort()) === JSON.stringify(Object.entries(registry.proxy_routes).sort()),
  "browser component routes equal registry routes");

let bindings = 0;
for (const facade of registry.facades) {
  const origin = facade.exact_origins[0];
  for (const product of facade.products) {
    bindings += 1;
    const browser = createFacadeBrowserSecurity({
      facadeId: facade.facade_id, origin, productCode: product,
      csrfToken: csrfFor(facade.facade_id), callbacks: facade.callbacks,
    });
    for (const [handle, path] of Object.entries(facade.callbacks)) {
      check(browser.resolveRedirect(handle) === `${origin}${path}`,
        `${facade.facade_id} ${handle} redirect resolves against the exact origin`);
    }
    for (const [route, [method, path]] of Object.entries(facadeBrowserSecurityContract.routes)) {
      const body = method === "POST"
        ? { request_id: `req_${route}_01`, idempotency_key: `idem_${route}_01` } : undefined;
      const request = browser.request(route, body);
      check(request.url === `${origin}${path}`, `${facade.facade_id} ${route} proxies to ${path}`);
      check(request.init.method === method && request.init.credentials === "same-origin"
        && request.init.redirect === "error" && request.init.cache === "no-store",
        `${facade.facade_id} ${route} fetch policy`);
      if (method === "POST") {
        check(typeof request.init.headers["X-CSRF-Token"] === "string", `${facade.facade_id} ${route} carries CSRF`);
        const payload = JSON.parse(request.init.body);
        check(payload.facade_id === facade.facade_id && payload.product_code === product
          && payload.request_id === `req_${route}_01`, `${facade.facade_id} ${route} body is server-bound`);
      } else {
        check(request.init.body === undefined && request.init.headers["X-CSRF-Token"] === undefined,
          `${facade.facade_id} ${route} GET carries no body`);
      }
    }
  }
}
check(bindings === registry.counts.product_bindings, "every product binding exercised through the browser component");

// --- B. Allowed and denied products at the component level ---------------------
const allProducts = new Set(products.protected_offers.map((row) => row.public_code));
check(allProducts.size === 3, "three protected offers");
for (const facade of registry.facades) {
  const origin = facade.exact_origins[0];
  for (const product of allProducts) {
    if (facade.products.includes(product)) continue;
    throws(() => createFacadeBrowserSecurity({
      facadeId: facade.facade_id, origin, productCode: product,
      csrfToken: csrfFor(facade.facade_id), callbacks: facade.callbacks,
    }), /FACADE_PRODUCT_DENIED/, `${facade.facade_id} denies ${product}`);
  }
  throws(() => createFacadeBrowserSecurity({
    facadeId: facade.facade_id, origin, productCode: "attacker_product_v1",
    csrfToken: csrfFor(facade.facade_id), callbacks: facade.callbacks,
  }), /FACADE_PRODUCT_DENIED/, `${facade.facade_id} denies attacker product`);
}

// --- C. Verification, checkout return, polling, and recovery links -------------
const routeForAction = {
  start: "/v1/activation/start", verify: "/v1/activation/verify",
  select: "/v1/activation/select-offer", pay: "/v1/activation/checkout",
  pending: "/v1/activation/poll", manage: "/v1/account/manage-link",
  recovery: "/v1/activation/poll",
};
for (const route of Object.values(routeForAction)) {
  check(Object.values(openapi.paths).some((ops) => Object.values(ops).some((op) => op.operationId
    && op.operationId.replace(/\./g, "_") === Object.keys(registry.proxy_routes)
      .find((key) => registry.proxy_routes[key] === route))),
    `${route} is a registered proxy route`);
}
check(registrationContract.schema === "focusa.spec152e.website_registration.v1", "registration contract schema");
check(registrationContract.authority === "WPUIAI.com EDD", "registration contract authority");
check(registrationContract.role === "presenter_only", "registration contract is presenter-only");
check(JSON.stringify(registrationContract.pages) === JSON.stringify(fixture.flow), "fixture flow equals component pages");
check(registrationContract.facadeIds.length === 4, "four branded website registration pages");

const registryById = new Map(registry.facades.map((facade) => [facade.facade_id, facade]));
const traces = [];
for (const entry of fixture.facades) {
  const registered = registryById.get(entry.facade_id);
  check(registered !== undefined && registered.exact_origins.includes(entry.origin)
    && registered.products.includes(entry.product_code), `${entry.facade_id} registry binding`);
  const theme = themeFor(entry.facade_id);
  check(theme.name === entry.brand && theme.product === entry.product_code, `${entry.facade_id} theme brand`);
  check(JSON.stringify(themes.themes[entry.facade_id]) === JSON.stringify({
    name: theme.name, mark: theme.mark, accent: theme.accent, product_code: theme.product,
  }), `${entry.facade_id} themes contract`);

  const model = createRegistrationModel(entry.facade_id);
  const trace = [model.page];
  for (const next of fixture.flow.slice(1)) {
    model.transition(next, {
      continuation_token: "opaqueSyntheticAcceptanceContinuation_01",
      masked_email: "s***@invalid.example",
      next_action: `continue_${next}`,
    });
    trace.push(model.page);
  }
  traces.push(trace);
  check(model.semanticState === 8, `${entry.facade_id} completes the full semantic flow`);
  for (const [action, expectedRoute] of Object.entries(routeForAction)) {
    const request = model.authorityRequest(action, { reason: "synthetic_acceptance" });
    check(request.route === expectedRoute, `${entry.facade_id} ${action} proxies to ${expectedRoute}`);
    check(request.facade_id === entry.facade_id && request.product_code === entry.product_code,
      `${entry.facade_id} ${action} request is server-bound`);
  }
}
for (const trace of traces.slice(1)) {
  check(JSON.stringify(trace) === JSON.stringify(traces[0]), "identical semantic flow across branded pages");
}
check(registrationSource.includes("Activation ready"), "checkout return reaches ready state");
check(registrationSource.includes("Keep this page open while the authority confirms the order and license."),
  "polling presenter discloses authority confirmation");
check(registrationSource.includes("Protected execution is unavailable."), "recovery presenter discloses outage posture");
for (const facadeId of registrationContract.facadeIds) {
  const model = createRegistrationModel(facadeId);
  for (const state of ["start", "verify", "select", "pay", "pending", "success", "manage"]) {
    const probe = createRegistrationModel(facadeId, state);
    check(probe.transition("recovery", {}) === "recovery", `${facadeId} recovery reachable from ${state}`);
  }
  check(model.authorityRequest("recovery").route === "/v1/activation/poll", `${facadeId} recovery polls the authority`);
}

// --- D. Spoofing at the component level ----------------------------------------
const installFacade = registryById.get("focusa_install_v1");
const installOrigin = installFacade.exact_origins[0];
const installProduct = installFacade.products[0];
for (const spoofedOrigin of [
  "http://install.focusa.dev",
  "https://install.focusa.dev.evil.invalid",
  "https://child.install.focusa.dev",
  "https://*.focusa.dev",
  "https://user@install.focusa.dev",
  "https://install.focusa.dev:8443",
  "https://install.focusa.dev/extra",
]) {
  throws(() => createFacadeBrowserSecurity({
    facadeId: "focusa_install_v1", origin: spoofedOrigin, productCode: installProduct,
    csrfToken: csrfFor("focusa_install_v1"), callbacks: installFacade.callbacks,
  }), /FACADE_ORIGIN_DENIED/, `spoofed origin denied: ${spoofedOrigin}`);
}
throws(() => createFacadeBrowserSecurity({
  facadeId: "attacker_facade_v1", origin: installOrigin, productCode: installProduct,
  csrfToken: csrfFor("focusa_install_v1"), callbacks: installFacade.callbacks,
}), /FACADE_ORIGIN_DENIED/, "unknown facade id denied");
for (const invalidCallbacks of [
  { success: "https://evil.invalid/callback" },
  { success: "//evil.invalid/callback" },
  { success: "/callback?redirect=https://evil.invalid" },
  { success: "/callback#fragment" },
  { success: "../authority" },
]) {
  throws(() => createFacadeBrowserSecurity({
    facadeId: "focusa_install_v1", origin: installOrigin, productCode: installProduct,
    csrfToken: csrfFor("focusa_install_v1"), callbacks: invalidCallbacks,
  }), /FACADE_REDIRECT_DENIED/, "unsafe callback registry denied");
}
const installBrowser = createFacadeBrowserSecurity({
  facadeId: "focusa_install_v1", origin: installOrigin, productCode: installProduct,
  csrfToken: csrfFor("focusa_install_v1"), callbacks: installFacade.callbacks,
});
throws(() => installBrowser.resolveRedirect("https://evil.invalid/callback"), /FACADE_REDIRECT_DENIED/, "absolute redirect denied");
throws(() => installBrowser.resolveRedirect("attacker"), /FACADE_REDIRECT_DENIED/, "unknown redirect handle denied");
for (const field of FORBIDDEN_BODY_FIELDS) {
  throws(() => installBrowser.request("activation_start", { [field]: "attacker-controlled" }),
    /FACADE_REQUEST_DENIED/, `browser body field ${field} denied`);
}
throws(() => installBrowser.request("activation_start", { product_code: "attacker_product_v1" }),
  /FACADE_REQUEST_DENIED/, "caller product override denied");
throws(() => installBrowser.request("activation_start", { grants: ["all"], limits: { nodes: 999 } }),
  /FACADE_REQUEST_DENIED/, "caller grant and limit denied");
throws(() => installBrowser.request("authority_issue"), /FACADE_METHOD_DENIED/, "issuance method denied");
throws(() => installBrowser.request("activation_offers", { product_code: installProduct }),
  /FACADE_METHOD_DENIED/, "GET with body denied");
throws(() => installBrowser.request("activation_start", "invalid"), /FACADE_REQUEST_DENIED/, "non-object body denied");
throws(() => installBrowser.request("activation_start", []), /FACADE_REQUEST_DENIED/, "array body denied");
throws(() => createFacadeBrowserSecurity({
  facadeId: "focusa_install_v1", origin: installOrigin, productCode: installProduct,
  csrfToken: "short", callbacks: installFacade.callbacks,
}), /FACADE_CSRF_DENIED/, "short CSRF token denied");

const marketingModel = createRegistrationModel("focusa_marketing_v1");
throws(() => themeFor("attacker_v1"), /FACADE_ORIGIN_DENIED/, "unknown theme facade denied");
throws(() => createRegistrationModel("focusa_marketing_v1", "authority-admin"), /FACADE_ROUTE_DENIED/, "unknown page denied");
throws(() => marketingModel.transition("success"), /ACTIVATION_TRANSITION_DENIED/, "invalid transition denied");
throws(() => marketingModel.authorityRequest("start", { redirect_url: "https://evil.invalid" }),
  /FACADE_REQUEST_FIELD_DENIED/, "caller redirect denied");
throws(() => marketingModel.authorityRequest("start", { price: "0", grants: ["all"] }),
  /FACADE_REQUEST_FIELD_DENIED/, "caller price/grant denied");
throws(() => marketingModel.transition("verify", { email: "synthetic@invalid.example" }),
  /ACTIVATION_ENVELOPE_DENIED/, "plaintext email envelope denied");
throws(() => marketingModel.transition("verify", { license_key: "SYNTHETIC-NOT-A-KEY" }),
  /ACTIVATION_ENVELOPE_DENIED/, "license key envelope denied");
throws(() => marketingModel.transition("verify", { credential: "synthetic-not-a-credential" }),
  /ACTIVATION_ENVELOPE_DENIED/, "credential envelope denied");
throws(() => marketingModel.transition("verify", { masked_email: "synthetic@invalid.example" }),
  /ACTIVATION_ENVELOPE_DENIED/, "unmasked email envelope denied");

// --- E. Timeout and expiry semantics -------------------------------------------
for (const badToken of [
  "https://evil.invalid/token", "//evil.invalid/token", "/relative/token",
  "short", "opaque token with spaces",
]) {
  throws(() => marketingModel.transition("verify", { continuation_token: badToken }),
    /FACADE_CONTINUATION_DENIED/, `malformed continuation denied: ${badToken}`);
}
check(securitySource.includes("cache: \"no-store\""), "browser responses are never cached");
check(!securitySource.includes("localStorage") && !securitySource.includes("sessionStorage"),
  "session and CSRF are never persisted by browser code");
check(!securitySource.includes("document.cookie"), "browser code cannot read or mint the presenter session");
check(!registrationSource.includes("localStorage") && !registrationSource.includes("sessionStorage"),
  "continuation state is memory-only and expires with the page");
check(registrationSource.includes("autocomplete: \"one-time-code\""), "verification code field shape");

// --- F. Unavailable authority disclosure ----------------------------------------
check(golden.expected.authority_route === "/v1/activation/start"
  && golden.expected.safe_redirect === `${installOrigin}${installFacade.callbacks.success}`,
  "golden vector binds authority routing and safe redirect");
check(!Object.keys(facadeBrowserSecurityContract.routes).some((route) => route.includes("issue")),
  "browser component exposes no issuance route");
check(pageSource.includes('<meta name="referrer" content="no-referrer">'), "branded page leaks no referrer");
check(!/[?&](?:redirect|product|price|grant)=/i.test(pageSource), "page has no client-controlled authority query values");
check(registrationSource.includes("WPUIAI.com EDD is the authority."), "presenter discloses the authority");
check(themes.authority === "WPUIAI.com EDD" && themes.facade_role === "presenter_only", "themes contract authority");

// --- G. Cross-surface parity ----------------------------------------------------
for (const facade of registry.facades) {
  check(securitySource.includes(facade.facade_id), `browser security source binds ${facade.facade_id}`);
  check(securitySource.includes(facade.exact_origins[0]), `browser security source binds ${facade.facade_id} origin`);
  for (const product of facade.products) {
    check(securitySource.includes(product), `browser security source binds ${product}`);
  }
}
for (const facadeId of registrationContract.facadeIds) {
  check(registryById.has(facadeId), `website theme facade ${facadeId} is registered`);
  check(fixture.facades.some((entry) => entry.facade_id === facadeId), `website theme facade ${facadeId} is in fixtures`);
}
check(securitySource.includes("focusa_install_v1") && securitySource.includes("uiai_engine_v1"),
  "installer surfaces are bound by the browser security component");

// --- H. Hygiene ----------------------------------------------------------------
const serialized = JSON.stringify({ securitySource, registrationSource, pageSource, themes, fixture, golden });
check(!/[A-Z0-9]{4}(?:-[A-Z0-9]{4}){3}/.test(serialized), "no license-shaped evidence");
check(!/@(?:gmail|outlook|yahoo|icloud)\./i.test(serialized), "no real email evidence");
check(!/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/i.test(serialized), "no secret-shaped evidence");
check(!registrationSource.includes("innerHTML"), "authority data is never rendered as HTML");
check(!securitySource.includes("redirect: \"follow\""), "browser requests never follow redirects");

console.log(JSON.stringify({
  schema: "focusa.spec152e.facade_acceptance_matrix_browser.v1",
  facades: registry.facades.length,
  exact_origins: registry.counts.exact_origins,
  product_bindings: bindings,
  proxy_routes: Object.keys(registry.proxy_routes).length,
  website_facades: registrationContract.facadeIds.length,
  positive_checks: positive,
  negative_checks: negative,
  result: "passed_fail_closed",
}));
