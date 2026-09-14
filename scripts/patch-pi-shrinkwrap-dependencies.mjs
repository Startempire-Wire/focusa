import { cpSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const safeBrace = join(root, "node_modules", "brace-expansion");
const piPackageRoot = process.env.FOCUSA_PI_PACKAGE_ROOT
  ? resolve(process.env.FOCUSA_PI_PACKAGE_ROOT)
  : join(root, "node_modules", "@earendil-works", "pi-coding-agent");
const bundledBrace = join(piPackageRoot, "node_modules", "brace-expansion");
const piAgentSession = join(piPackageRoot, "dist", "core", "agent-session.js");

const version = (dir) => JSON.parse(readFileSync(join(dir, "package.json"), "utf8")).version;

if (existsSync(bundledBrace)) {
  if (!existsSync(safeBrace) || version(safeBrace) !== "5.0.8") {
    throw new Error("safe brace-expansion@5.0.8 override is unavailable");
  }
  if (version(bundledBrace) !== "5.0.8") {
    // The upstream Pi package ships an npm-shrinkwrap with 5.0.7. Overlay the
    // root audited package without deleting the package tree, then verify it.
    cpSync(safeBrace, bundledBrace, { recursive: true, force: true });
  }
  if (version(bundledBrace) !== "5.0.8") {
    throw new Error("Pi shrinkwrap security overlay did not activate");
  }
  console.log("Pi shrinkwrap security overlay: brace-expansion@5.0.8");
}

const TOOL_BOUNDARY_MARKER = "FOCUSA_TOOL_BOUNDARY_COMPACTION_V2";

function replaceRequired(source, oldText, newText, label) {
  const matches = source.split(oldText).length - 1;
  if (matches !== 1) {
    throw new Error(`Pi tool-boundary overlay ${label} expected one anchor, found ${matches}`);
  }
  return source.replace(oldText, newText);
}

function replaceRequiredPattern(source, pattern, replacement, label) {
  const matches = source.match(pattern);
  if (!matches || matches.length !== 1) {
    throw new Error(`Pi tool-boundary overlay ${label} expected one anchor, found ${matches?.length ?? 0}`);
  }
  return source.replace(pattern[0], replacement);
}

function patchPiToolBoundaryCompaction(path) {
  if (!existsSync(path)) return;
  let source = readFileSync(path, "utf8");
  if (source.includes(TOOL_BOUNDARY_MARKER)) {
    console.log(`Pi compaction lifecycle overlay: ${TOOL_BOUNDARY_MARKER}`);
    return;
  }

  source = replaceRequired(
    source,
    `    // Track last assistant message for auto-compaction check\n    _lastAssistantMessage = undefined;\n    /** Internal handler for agent events - shared by subscribe and reconnect */`,
    `    // Track last assistant message for auto-compaction check\n    _lastAssistantMessage = undefined;\n    // ${TOOL_BOUNDARY_MARKER}: extension compaction requests made during an\n    // active tool loop are executed by Pi at turn_end without abort/resend.\n    _pendingExtensionToolBoundaryCompaction = undefined;\n    _focusaRefreshCompactedMessagesOnNextTurn = false;\n    _focusaToolBoundaryCapability = (() => {\n        globalThis[Symbol.for("focusa.pi.tool-boundary-compaction.v1")] = true;\n        return "${TOOL_BOUNDARY_MARKER}";\n    })();\n    _settleExtensionCompactionRequest = async (request, reason = "threshold") => {\n        const before = getLatestCompactionEntry(this.sessionManager.getBranch());\n        await this._runAutoCompaction(reason, false, request?.customInstructions);\n        const after = getLatestCompactionEntry(this.sessionManager.getBranch());\n        if (after && after.id !== before?.id) {\n            this._focusaRefreshCompactedMessagesOnNextTurn = this._isAgentRunActive;\n            request?.onComplete?.({\n                summary: after.summary,\n                firstKeptEntryId: after.firstKeptEntryId,\n                tokensBefore: after.tokensBefore,\n                estimatedTokensAfter: estimateMessagesTokens(this.agent.state.messages),\n                usage: after.usage,\n                details: after.details,\n            });\n            return;\n        }\n        request?.onError?.(new Error("Pi native compaction did not complete at the safe lifecycle boundary"));\n    };\n    /** Internal handler for agent events - shared by subscribe and reconnect */`,
    "coordinator fields"
  );

  source = replaceRequired(
    source,
    `            const previousContext = previousSnapshot?.context ?? turn.context;\n            return {`,
    `            const previousContext = previousSnapshot?.context ?? turn.context;\n            const refreshCompactedMessages = this._focusaRefreshCompactedMessagesOnNextTurn;\n            this._focusaRefreshCompactedMessagesOnNextTurn = false;\n            return {`,
    "next-turn refresh flag"
  );

  source = replaceRequired(
    source,
    `                    ...previousContext,\n                    systemPrompt: this._systemPromptOverride ?? this._baseSystemPrompt,`,
    `                    ...previousContext,\n                    messages: refreshCompactedMessages\n                        ? this.agent.state.messages.slice()\n                        : previousContext.messages,\n                    systemPrompt: this._systemPromptOverride ?? this._baseSystemPrompt,`,
    "compacted message refresh"
  );

  source = replaceRequired(
    source,
    `        // Emit to extensions first\n        await this._emitExtensionEvent(event);\n        // Notify all listeners`,
    `        // Emit to extensions first\n        await this._emitExtensionEvent(event);\n        // Focusa chooses whether compaction is useful. Pi owns the safe native\n        // lifecycle and executes the one queued request before the next model call.\n        if (event.type === "turn_end" && this._pendingExtensionToolBoundaryCompaction) {\n            const request = this._pendingExtensionToolBoundaryCompaction;\n            this._pendingExtensionToolBoundaryCompaction = undefined;\n            await this._settleExtensionCompactionRequest(request, "threshold");\n        }\n        // Notify all listeners`,
    "turn_end drain"
  );

  source = replaceRequired(
    source,
    `    async _runAutoCompaction(reason, willRetry) {`,
    `    async _runAutoCompaction(reason, willRetry, customInstructions) {`,
    "native compaction signature"
  );

  source = replaceRequired(
    source,
    `                const extensionResult = (await this._extensionRunner.emit({\n                    type: "session_before_compact",\n                    preparation,\n                    branchEntries: pathEntries,\n                    customInstructions: undefined,\n                    reason,\n                    willRetry,\n                    signal: this._autoCompactionAbortController.signal,\n                }));`,
    `                const compactionEvent = {\n                    type: "session_before_compact",\n                    preparation,\n                    branchEntries: pathEntries,\n                    customInstructions,\n                    reason,\n                    willRetry,\n                    signal: this._autoCompactionAbortController.signal,\n                };\n                const extensionResult = (await this._extensionRunner.emit(compactionEvent));\n                customInstructions = compactionEvent.customInstructions ?? customInstructions;`,
    "native instruction enrichment"
  );

  source = replaceRequiredPattern(
    source,
    /                const compactResult = await compact\(preparation, [^,\n]+, apiKey, headers, undefined, this\._autoCompactionAbortController\.signal, this\.thinkingLevel, this\.agent\.streamFunction, env, this\.settingsManager\.getRetrySettings\(\), this\._summarizationRetryCallbacks\(\{ source: "compaction", reason \}\)\);/g,
    (match) =>
      match.replace(
        ", undefined, this._autoCompactionAbortController",
        ", customInstructions, this._autoCompactionAbortController"
      ),
    "native instruction delivery"
  );

  source = replaceRequired(
    source,
    `            compact: (options) => {\n                void (async () => {\n                    try {\n                        const result = await this.compact(options?.customInstructions);\n                        options?.onComplete?.(result);\n                    }\n                    catch (error) {\n                        const err = error instanceof Error ? error : new Error(String(error));\n                        options?.onError?.(err);\n                    }\n                })();\n            },`,
    `            compact: (options) => {\n                if (this._isAgentRunActive) {\n                    if (this._pendingExtensionToolBoundaryCompaction) {\n                        options?.onError?.(new Error("A Focusa compaction request is already queued for this tool boundary"));\n                        return;\n                    }\n                    this._pendingExtensionToolBoundaryCompaction = options ?? {};\n                    return;\n                }\n                void this._settleExtensionCompactionRequest(options ?? {}, "threshold");\n            },`,
    "extension compact binding"
  );

  writeFileSync(path, source);
  const activated = readFileSync(path, "utf8");
  for (const required of [
    TOOL_BOUNDARY_MARKER,
    "_pendingExtensionToolBoundaryCompaction",
    "_settleExtensionCompactionRequest",
    "_focusaRefreshCompactedMessagesOnNextTurn",
    "customInstructions = compactionEvent.customInstructions ?? customInstructions",
  ]) {
    if (!activated.includes(required)) {
      throw new Error(`Pi compaction lifecycle overlay missing ${required}`);
    }
  }
  console.log(`Pi compaction lifecycle overlay: ${TOOL_BOUNDARY_MARKER}`);
}

patchPiToolBoundaryCompaction(piAgentSession);
