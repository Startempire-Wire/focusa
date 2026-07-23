import { fireEvent, render } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import TrustedComponent from "../src/TrustedComponent.svelte";

const base = { componentName: "FocusaPrimaryAction", label: "Continue project" };

describe("trusted Focusa component semantics", () => {
  it("provides one keyboard-operable dominant action and visible save state", async () => {
    const view = render(TrustedComponent, {
      props: { ...base, kind: "action", actionAvailable: true, busy: false },
    });
    const button = view.getByRole("button", { name: "Continue" });
    await fireEvent.click(button);
    expect(button.hasAttribute("disabled")).toBe(false);
    expect(view.getByText("ready").getAttribute("aria-live")).toBe("polite");
  });

  it("exposes explicit recovery, progress, advanced, and terminal states", () => {
    const recovery = render(TrustedComponent, {
      props: { ...base, kind: "recovery", status: "retry", details: "operation unavailable" },
    });
    expect(recovery.getByRole("alert")).toBeTruthy();
    expect(recovery.getByText("Advanced details")).toBeTruthy();
    expect(recovery.container.querySelector("[data-terminal-fallback]")?.textContent).toContain("retry");

    const progress = render(TrustedComponent, {
      props: { ...base, kind: "progress", progress: 140 },
    });
    expect(progress.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("100");
  });
});
