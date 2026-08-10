import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
const LEGACY_RESTART_MARKER = "pi-extension-restart-required.json";
const RESTART_MARKER = "pi-extension-silent-restart-required.json";
const ACTIVATING_MARKER = "pi-extension-activating.json";
const ACTIVATION_RECEIPT = "pi-extension-activation-receipt.json";
const POLL_MS = 2_500;
const TIMER_KEY = Symbol.for("focusa.pi_extension.ota_activation_timer.v1");
export function otaActivationStateRoot(env = process.env) {
    const stateHome = String(env.XDG_STATE_HOME || "").trim() ||
        join(String(env.HOME || "").trim() || homedir(), ".local", "state");
    return join(stateHome, "focusa", "update");
}
export function otaActivationPaths(root = otaActivationStateRoot()) {
    return {
        legacy: join(root, LEGACY_RESTART_MARKER),
        restart: join(root, RESTART_MARKER),
        activating: join(root, ACTIVATING_MARKER),
        receipt: join(root, ACTIVATION_RECEIPT),
    };
}
function readMarker(path) {
    try {
        return JSON.parse(readFileSync(path, "utf8"));
    }
    catch {
        return {};
    }
}
function loadedExtensionVersion() {
    const here = dirname(fileURLToPath(import.meta.url));
    for (const path of [join(here, "package.json"), join(here, "..", "package.json")]) {
        const version = String(readMarker(path).version || "").replace(/^v/, "");
        if (version)
            return version;
    }
    return "";
}
function writeReceipt(source, receipt, activation) {
    if (!existsSync(source))
        return;
    const marker = readMarker(source);
    const payload = {
        schema: "focusa.pi_extension_activation_receipt.v1",
        status: "activated",
        version: marker.version || null,
        installed_at: marker.installed_at || null,
        activated_at: new Date().toISOString(),
        activation,
    };
    mkdirSync(join(receipt, ".."), { recursive: true, mode: 0o700 });
    const temporary = `${receipt}.tmp-${process.pid}-${Date.now()}`;
    writeFileSync(temporary, `${JSON.stringify(payload, null, 2)}\n`, { mode: 0o600 });
    renameSync(temporary, receipt);
    rmSync(source, { force: true });
}
/**
 * Activate an atomically installed Pi extension without synthetic conversation.
 * A supported runtime may expose reloadWhenIdle; otherwise the marker remains
 * pending and the next natural Pi process start writes the activation receipt.
 */
export function registerAutomaticOtaActivation(pi) {
    const paths = otaActivationPaths();
    const timerGlobal = globalThis;
    let reloading = false;
    try {
        if (existsSync(paths.legacy) && !existsSync(paths.restart)) {
            renameSync(paths.legacy, paths.restart);
        }
        const loaded = loadedExtensionVersion();
        const activatingVersion = String(readMarker(paths.activating).version || "").replace(/^v/, "");
        const restartVersion = String(readMarker(paths.restart).version || "").replace(/^v/, "");
        if (loaded && activatingVersion === loaded) {
            writeReceipt(paths.activating, paths.receipt, "safe_idle_reload");
        }
        else if (loaded && restartVersion === loaded) {
            writeReceipt(paths.restart, paths.receipt, "process_start");
        }
    }
    catch {
        // Keep retry markers; startup must never crash Pi or claim false activation.
    }
    const requestActivation = async () => {
        if (reloading || !existsSync(paths.restart))
            return;
        const reloadWhenIdle = pi.reloadWhenIdle;
        if (typeof reloadWhenIdle !== "function")
            return;
        reloading = true;
        mkdirSync(otaActivationStateRoot(), { recursive: true, mode: 0o700 });
        rmSync(paths.activating, { force: true });
        renameSync(paths.restart, paths.activating);
        try {
            await reloadWhenIdle.call(pi);
        }
        catch {
            if (existsSync(paths.activating))
                renameSync(paths.activating, paths.restart);
        }
        finally {
            reloading = false;
        }
    };
    pi.on("session_start", async () => void requestActivation());
    pi.on("agent_end", async () => void requestActivation());
    pi.on("session_shutdown", async () => {
        const timer = timerGlobal[TIMER_KEY];
        if (timer)
            clearInterval(timer);
        delete timerGlobal[TIMER_KEY];
    });
    const previous = timerGlobal[TIMER_KEY];
    if (previous)
        clearInterval(previous);
    const timer = setInterval(() => void requestActivation(), POLL_MS);
    timer.unref?.();
    timerGlobal[TIMER_KEY] = timer;
    void requestActivation();
    return () => {
        clearInterval(timer);
        if (timerGlobal[TIMER_KEY] === timer)
            delete timerGlobal[TIMER_KEY];
    };
}
