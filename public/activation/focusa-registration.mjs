const PAGE_STATES = Object.freeze([
  "start", "verify", "select", "pay", "pending", "success", "manage", "recovery",
]);

const STATE_TRANSITIONS = Object.freeze({
  start: ["verify", "recovery"],
  verify: ["select", "recovery"],
  select: ["pay", "pending", "recovery"],
  pay: ["pending", "recovery"],
  pending: ["pending", "success", "recovery"],
  success: ["manage", "recovery"],
  manage: ["recovery"],
  recovery: ["start", "verify", "manage"],
});

const THEMES = Object.freeze({
  focusa_marketing_v1: Object.freeze({
    name: "Focusa", mark: "F", accent: "#6d5efc", product: "focusa_operator_lifetime_v1",
  }),
  focusa_forge_v1: Object.freeze({
    name: "Focusa Forge", mark: "F", accent: "#d4552d", product: "focusa_operator_lifetime_v1",
  }),
  focusa_arena_v1: Object.freeze({
    name: "Focusa Arena", mark: "A", accent: "#087e8b", product: "focusa_operator_lifetime_v1",
  }),
  wpuiai_public_v1: Object.freeze({
    name: "WPUIAI", mark: "W", accent: "#3159a6", product: "uiai_operator_lifetime_v1",
  }),
});

const COPY = Object.freeze({
  start: ["Activate your product", "Enter your email to receive a single-use verification challenge."],
  verify: ["Verify your email", "Use the short-lived code sent to the masked address."],
  select: ["Choose an offer", "Offers and grants are supplied by the authority."],
  pay: ["Continue to secure checkout", "Payment details are collected only by EDD and its configured Stripe gateway."],
  pending: ["Activation pending", "Keep this page open while the authority confirms the order and license."],
  success: ["Activation ready", "Your verified entitlement is ready for device registration."],
  manage: ["Manage account", "Review licenses and nodes through the authenticated authority workflow."],
  recovery: ["Activation recovery", "Protected execution is unavailable. Account, repair, export, and uninstall remain available."],
});

function opaqueToken(value) {
  return typeof value === "string" && value.length >= 24 && value.length <= 4096 &&
    /^[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*$/.test(value) &&
    !/^(?:https?:|\/)/i.test(value);
}

function maskedEmail(value) {
  return typeof value === "string" && /^[^@]*\*[^@]*@[^@]+$/.test(value);
}

export function themeFor(facadeId) {
  if (!Object.hasOwn(THEMES, facadeId)) throw new Error("FACADE_ORIGIN_DENIED");
  return THEMES[facadeId];
}

export function createRegistrationModel(facadeId, initialPage = "start") {
  const theme = themeFor(facadeId);
  if (!PAGE_STATES.includes(initialPage)) throw new Error("FACADE_ROUTE_DENIED");
  let page = initialPage;
  let continuation = "";
  let publicStatus = "";

  return Object.freeze({
    get page() { return page; },
    get theme() { return theme; },
    get semanticState() { return PAGE_STATES.indexOf(page) + 1; },
    get status() { return publicStatus; },
    transition(next, envelope = {}) {
      if (!STATE_TRANSITIONS[page].includes(next)) throw new Error("ACTIVATION_TRANSITION_DENIED");
      const forbidden = ["email", "license_key", "credential", "secret", "redirect_url", "product_code"];
      if (forbidden.some((key) => Object.hasOwn(envelope, key))) throw new Error("ACTIVATION_ENVELOPE_DENIED");
      if (Object.hasOwn(envelope, "masked_email") && !maskedEmail(envelope.masked_email)) {
        throw new Error("ACTIVATION_ENVELOPE_DENIED");
      }
      if (Object.hasOwn(envelope, "continuation_token")) {
        if (!opaqueToken(envelope.continuation_token)) throw new Error("FACADE_CONTINUATION_DENIED");
        continuation = envelope.continuation_token;
      }
      publicStatus = typeof envelope.next_action === "string" && envelope.next_action.length <= 240
        ? envelope.next_action : "";
      page = next;
      return page;
    },
    authorityRequest(action, fields = {}) {
      const routes = {
        start: "/v1/activation/start", verify: "/v1/activation/verify",
        select: "/v1/activation/select-offer", pay: "/v1/activation/checkout",
        pending: "/v1/activation/poll", manage: "/v1/account/manage-link",
        recovery: "/v1/activation/poll",
      };
      if (!Object.hasOwn(routes, action)) throw new Error("FACADE_ACTION_DENIED");
      if (Object.hasOwn(fields, "product_code") || Object.hasOwn(fields, "redirect_url") ||
          Object.hasOwn(fields, "success_url") || Object.hasOwn(fields, "price") ||
          Object.hasOwn(fields, "grants")) throw new Error("FACADE_REQUEST_FIELD_DENIED");
      return Object.freeze({
        route: routes[action],
        facade_id: facadeId,
        product_code: theme.product,
        continuation_token: continuation,
        fields: Object.freeze({ ...fields }),
      });
    },
  });
}

function field(document, { id, label, type, autocomplete, inputmode }) {
  const wrap = document.createElement("div");
  wrap.className = "field";
  const labelNode = document.createElement("label");
  labelNode.htmlFor = id;
  labelNode.textContent = label;
  const input = document.createElement("input");
  input.id = id;
  input.name = id;
  input.type = type;
  input.required = true;
  input.autocomplete = autocomplete;
  if (inputmode) input.inputMode = inputmode;
  wrap.append(labelNode, input);
  return wrap;
}

export class FocusaRegistration extends (globalThis.HTMLElement || class {}) {
  connectedCallback() {
    const facadeId = this.getAttribute("facade-id") || "";
    const requestedPage = this.getAttribute("page") || "start";
    let model;
    try { model = createRegistrationModel(facadeId, requestedPage); }
    catch { this.replaceChildren(this.ownerDocument.createTextNode("Registration unavailable.")); return; }

    const root = this.attachShadow({ mode: "closed" });
    const document = this.ownerDocument;
    const main = document.createElement("main");
    main.setAttribute("aria-labelledby", "activation-title");
    main.style.setProperty("--activation-accent", model.theme.accent);

    const brand = document.createElement("p");
    brand.className = "brand";
    brand.setAttribute("aria-label", model.theme.name);
    brand.textContent = `${model.theme.mark}  ${model.theme.name}`;
    const title = document.createElement("h1");
    title.id = "activation-title";
    title.textContent = COPY[model.page][0];
    const help = document.createElement("p");
    help.id = "activation-help";
    help.textContent = COPY[model.page][1];
    const form = document.createElement("form");
    form.method = "post";
    form.action = model.page === "manage" ? "/v1/account/manage-link" :
      (model.authorityRequest(model.page === "success" ? "manage" : model.page).route);
    form.setAttribute("aria-describedby", "activation-help activation-status");

    if (model.page === "start") form.append(field(document, {
      id: "email", label: "Email address", type: "email", autocomplete: "email", inputmode: "email",
    }));
    if (model.page === "verify") form.append(field(document, {
      id: "verification_code", label: "Verification code", type: "text", autocomplete: "one-time-code", inputmode: "numeric",
    }));

    const button = document.createElement("button");
    button.type = "submit";
    button.textContent = model.page === "recovery" ? "Continue recovery" : "Continue";
    form.append(button);
    const status = document.createElement("p");
    status.id = "activation-status";
    status.role = "status";
    status.setAttribute("aria-live", "polite");
    const note = document.createElement("p");
    note.className = "privacy";
    note.textContent = "WPUIAI.com EDD is the authority. This branded page cannot issue licenses or entitlements.";
    const style = document.createElement("style");
    style.textContent = `:host{display:block;color:#172033;font:16px/1.5 system-ui,sans-serif}main{box-sizing:border-box;max-width:36rem;margin:auto;padding:2rem;border:1px solid #d9deea;border-top:4px solid var(--activation-accent);border-radius:.75rem;background:#fff}.brand{color:var(--activation-accent);font-weight:750}h1{font-size:1.75rem;line-height:1.2}.field{display:grid;gap:.4rem;margin:1.5rem 0}input,button{box-sizing:border-box;min-height:44px;font:inherit;border-radius:.4rem}input{padding:.65rem;border:1px solid #687386}button{padding:.65rem 1rem;border:0;color:#fff;background:var(--activation-accent);font-weight:700}.privacy{font-size:.85rem;color:#4e596d}`;
    main.append(brand, title, help, form, status, note);
    root.append(style, main);
  }
}

export const registrationContract = Object.freeze({
  schema: "focusa.spec152e.website_registration.v1",
  pages: PAGE_STATES,
  transitions: STATE_TRANSITIONS,
  facadeIds: Object.freeze(Object.keys(THEMES)),
  authority: "WPUIAI.com EDD",
  role: "presenter_only",
});

if (globalThis.customElements && !globalThis.customElements.get("focusa-registration")) {
  globalThis.customElements.define("focusa-registration", FocusaRegistration);
}
