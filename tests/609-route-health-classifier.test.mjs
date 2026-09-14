import assert from "node:assert/strict";
import test from "node:test";
import { classifyRouteResponse } from "../scripts/audit-route-health.mjs";

const unsupported = {
  method: "GET", path: "/credentials/providers", status: 501,
  json: { status: "unsupported", code: "credential_provider_registry_not_implemented", providers: [] },
};

test("exact unwired-provider response is unavailable, never healthy", () => {
  assert.equal(classifyRouteResponse(unsupported), "unavailable");
});

test("provider exceptions cannot mask a false success or malformed failure", () => {
  for (const change of [
    { status: 200 }, { status: 500 }, { status: 503 }, { status: 404 },
    { json: null }, { json: {} },
    { json: { ...unsupported.json, status: "ok" } },
    { json: { ...unsupported.json, code: "different_failure" } },
    { json: { ...unsupported.json, providers: [{}] } },
    { json: { ...unsupported.json, providers: null } },
  ]) assert.equal(classifyRouteResponse({ ...unsupported, ...change }), "broken", JSON.stringify(change));
});

test("no broad server-error exemption exists", () => {
  for (const status of [404, 405, 500, 501, 502, 503]) {
    assert.equal(classifyRouteResponse({ ...unsupported, path: "/health", status }), "broken");
    assert.equal(classifyRouteResponse({ ...unsupported, method: "POST", status }), "broken");
  }
  for (const status of [200, 400, 403, 409, 422]) {
    assert.equal(classifyRouteResponse({ method: "GET", path: "/health", status }), "responsive");
  }
});
