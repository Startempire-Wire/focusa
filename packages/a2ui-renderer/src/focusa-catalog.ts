import { componentManifest } from "@focusa/elements";
import {
  ActionSchema,
  Catalog,
  DynamicNumberSchema,
  DynamicStringSchema,
} from "@a2ui/web_core/v0_9";
import {
  A2uiController,
  A2uiLitElement,
  basicCatalog,
  type LitComponentApi,
} from "@a2ui/lit/v0_9";
import { nothing } from "lit";
import { html, unsafeStatic } from "lit/static-html.js";
import { z } from "zod";

export const FOCUSA_CATALOG_ID = "https://focusa.dev/a2ui/v0_9/catalog.json";

export const FocusaComponentSchema = z.object({
  label: DynamicStringSchema.optional(),
  description: DynamicStringSchema.optional(),
  status: DynamicStringSchema.optional(),
  progress: DynamicNumberSchema.optional(),
  primaryActionLabel: DynamicStringSchema.optional(),
  action: ActionSchema.optional(),
  disabled: z.boolean().optional(),
  busy: z.boolean().optional(),
  details: DynamicStringSchema.optional(),
}).strict();

interface BoundProps {
  label?: string;
  description?: string;
  status?: string;
  progress?: number;
  primaryActionLabel?: string;
  action?: () => void;
  disabled?: boolean;
  busy?: boolean;
  details?: string;
}

export const focusaComponentNames = componentManifest.map(({ name }) => name);
const focusaA2uiComponents: LitComponentApi[] = [];

for (const definition of componentManifest) {
  const adapterTag = `${definition.tag}-a2ui`;
  const api = { name: definition.name, schema: FocusaComponentSchema };
  const trustedTag = unsafeStatic(definition.tag);

  if (!customElements.get(adapterTag)) {
    customElements.define(adapterTag, class extends A2uiLitElement<typeof api> {
      protected override createController(): A2uiController<typeof api> {
        return new A2uiController(this, api);
      }

      override render() {
        const props = this.controller.props as BoundProps | undefined;
        if (!props) return nothing;
        return html`<${trustedTag}
          .label=${props.label ?? definition.name}
          .description=${props.description ?? ""}
          .status=${props.status ?? "ready"}
          .progress=${props.progress ?? 0}
          .primaryActionLabel=${props.primaryActionLabel ?? "Continue"}
          .actionAvailable=${typeof props.action === "function"}
          .disabled=${props.disabled ?? false}
          .busy=${props.busy ?? false}
          .details=${props.details ?? ""}
          .invokeAction=${props.action}
        ></${trustedTag}>`;
      }
    });
  }

  focusaA2uiComponents.push({ ...api, tagName: adapterTag });
}

/** Trusted Focusa catalog extends maintained ordinary A2UI primitives. */
export const focusaCatalog = new Catalog<LitComponentApi>(FOCUSA_CATALOG_ID, [
  ...basicCatalog.components.values(),
  ...focusaA2uiComponents,
]);
