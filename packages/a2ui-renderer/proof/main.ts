import {
  FOCUSA_A2UI_CATALOG_ID,
  FocusaA2uiRenderer,
  type A2uiClientAction,
  type A2uiMessage,
} from "../src/index.js";

const projectScope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-alpha0-eval",
  attachment_id: "attachment:context-alpha0",
};
const result = document.querySelector<HTMLElement>("#action-result")!;
const surface = document.querySelector<HTMLElement>("#surface")!;
const observedActions: A2uiClientAction[] = [];

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set(["context.review"]),
  onAction(action) {
    observedActions.push(action);
    result.textContent = `Executed ${action.name} for ${String(action.context.project_root)}`;
    document.body.dataset.lastAction = action.name;
  },
});

const snapshot: A2uiMessage[] = [
  {
    version: "v0.9",
    createSurface: {
      surfaceId: "alpha0-generated-ui",
      catalogId: FOCUSA_A2UI_CATALOG_ID,
    },
  },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "alpha0-generated-ui",
      components: [
        { id: "root", component: "Column", children: ["stage", "allowed", "blocked", "unknown"] },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "What Focusa knows",
          description: "Three grounded sources are ready for your decision.",
          status: "saved",
          details: "surface=alpha0-generated-ui; cursor=42",
        },
        {
          id: "allowed",
          component: "FocusaPrimaryAction",
          label: "Review grounded sources",
          description: "This action is allowlisted by the generated Operation Registry binding.",
          primaryActionLabel: "Review sources",
          action: { event: { name: "context.review", context: projectScope } },
        },
        {
          id: "blocked",
          component: "FocusaPrimaryAction",
          label: "Unsafe unbound action",
          description: "This intentionally exercises fail-closed recovery.",
          primaryActionLabel: "Test recovery",
          action: { event: { name: "unknown.mutate", context: projectScope } },
        },
        { id: "unknown", component: "UntrustedGeneratedWidget" },
      ],
    },
  },
];

renderer.processSnapshot(snapshot);
renderer.mount(surface, "alpha0-generated-ui");

Object.assign(window, {
  focusaEval: {
    renderer,
    observedActions,
    projectScope,
    schema: "uiai.focusa_ui_eval_runtime.v1",
  },
});
