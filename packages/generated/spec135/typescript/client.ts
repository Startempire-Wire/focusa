import createClient, { type ClientOptions } from "openapi-fetch";
import type { paths } from "./schema.js";

export type FocusaSpec135Client = ReturnType<typeof createClient<paths>>;

/** Create a typed Focusa client from the generated OpenAPI 3.0.3 contract. */
export function createFocusaSpec135Client(options: ClientOptions): FocusaSpec135Client {
  return createClient<paths>(options);
}
