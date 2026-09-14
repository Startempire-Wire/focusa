const ROUTES = Object.freeze({
  activation_start: ["POST", "/v1/activation/start"],
  activation_verify: ["POST", "/v1/activation/verify"],
  activation_offers: ["GET", "/v1/activation/offers"],
  activation_select_offer: ["POST", "/v1/activation/select-offer"],
  activation_checkout: ["POST", "/v1/activation/checkout"],
  activation_existing_license: ["POST", "/v1/activation/existing-license"],
  activation_poll: ["POST", "/v1/activation/poll"],
  lease_refresh: ["POST", "/v1/lease/refresh"],
  nodes_list: ["GET", "/v1/nodes"],
  nodes_deactivate: ["POST", "/v1/nodes/deactivate"],
  account_manage_link: ["GET", "/v1/account/manage-link"],
});

const FACADE_BINDINGS = Object.freeze({
  focusa_install_v1: ["https://install.focusa.dev", [
    "focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
  ]],
  focusa_marketing_v1: ["https://focusa.dev", [
    "focusa_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
  ]],
  focusa_forge_v1: ["https://forge.focusa.dev", [
    "focusa_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
  ]],
  focusa_arena_v1: ["https://arena.focusa.dev", [
    "focusa_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
  ]],
  uiai_engine_v1: ["https://engine.focusa.dev", [
    "uiai_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
  ]],
  wpuiai_public_v1: ["https://wpuiai.com", [
    "focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1", "focusa_uiai_operator_bundle_lifetime_v1",
  ]],
});

const CALLBACKS = Object.freeze({
  cancel: "/activate/callback/cancel",
  recovery: "/activate/callback/recovery",
  success: "/activate/callback/success",
});

const FORBIDDEN_BODY_FIELDS = Object.freeze([
  "callback_url", "cancel_url", "credential", "edd_download_id", "edd_price_id",
  "email_exists", "features", "grants", "license_key", "limits", "price",
  "redirect_url", "secret", "sender_email", "success_url",
]);

function exactHttpsOrigin(value) {
  if (typeof value !== "string") return false;
  try {
    const url = new URL(value);
    return url.protocol === "https:" && url.origin === value &&
      !url.username && !url.password && url.pathname === "/" && !url.search && !url.hash;
  } catch { return false; }
}

function opaque(value, min = 24, max = 4096) {
  return typeof value === "string" && value.length >= min && value.length <= max &&
    /^[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*$/.test(value);
}

/** Browser adapter: the server-issued session stays in an HttpOnly, host-only cookie. */
export function createFacadeBrowserSecurity({ facadeId, origin, productCode, csrfToken, callbacks }) {
  const binding = Object.hasOwn(FACADE_BINDINGS, facadeId) ? FACADE_BINDINGS[facadeId] : undefined;
  if (!opaque(facadeId, 3, 128) || !opaque(productCode, 3, 128) || !exactHttpsOrigin(origin) ||
      !binding || binding[0] !== origin) {
    throw new Error("FACADE_ORIGIN_DENIED");
  }
  if (!binding[1].includes(productCode)) throw new Error("FACADE_PRODUCT_DENIED");
  if (!opaque(csrfToken)) throw new Error("FACADE_CSRF_DENIED");
  if (!callbacks || typeof callbacks !== "object" || Array.isArray(callbacks) ||
      Object.keys(callbacks).length !== Object.keys(CALLBACKS).length ||
      Object.entries(CALLBACKS).some(([handle, path]) => callbacks[handle] !== path)) {
    throw new Error("FACADE_REDIRECT_DENIED");
  }
  const safeCallbacks = CALLBACKS;

  return Object.freeze({
    resolveRedirect(handle) {
      if (!Object.hasOwn(safeCallbacks, handle)) throw new Error("FACADE_REDIRECT_DENIED");
      return `${origin}${safeCallbacks[handle]}`;
    },
    request(route, body = undefined) {
      if (!Object.hasOwn(ROUTES, route)) throw new Error("FACADE_METHOD_DENIED");
      const [method, path] = ROUTES[route];
      if (body !== undefined && (body === null || Array.isArray(body) || typeof body !== "object")) {
        throw new Error("FACADE_REQUEST_DENIED");
      }
      const payload = body === undefined ? {} : { ...body };
      if (FORBIDDEN_BODY_FIELDS.some((field) => Object.hasOwn(payload, field)) ||
          (Object.hasOwn(payload, "product_code") && payload.product_code !== productCode)) {
        throw new Error("FACADE_REQUEST_DENIED");
      }
      const headers = { "Accept": "application/json" };
      const init = { method, credentials: "same-origin", redirect: "error", cache: "no-store", headers };
      if (method === "POST") {
        headers["Content-Type"] = "application/json";
        headers["X-CSRF-Token"] = csrfToken;
        init.body = JSON.stringify({ ...payload, facade_id: facadeId, product_code: productCode });
      } else if (Object.keys(payload).length !== 0) {
        throw new Error("FACADE_METHOD_DENIED");
      }
      return Object.freeze({ url: `${origin}${path}`, init: Object.freeze(init) });
    },
  });
}

export const facadeBrowserSecurityContract = Object.freeze({
  schema: "focusa.spec152e.facade_browser_security.v1",
  cookieAuthority: "server_http_only_host_only_same_site_strict",
  storageAuthority: "forbidden",
  routes: ROUTES,
});
