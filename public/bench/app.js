const text = (value, fallback = "Not measured") => value ?? fallback;
const metric = (label, value, note = "") => `<div class="metric"><span>${label}</span><strong>${text(value)}</strong><span>${note}</span></div>`;

async function loadSnapshot() {
  const response = await fetch("./latest.json", { cache: "no-store" });
  if (!response.ok) throw new Error(`snapshot HTTP ${response.status}`);
  return response.json();
}

function render(snapshot) {
  const published = snapshot.publish_allowed === true && snapshot.redaction_status === "passed";
  const badge = document.querySelector("#snapshot-badge");
  badge.textContent = published ? `Published · ${snapshot.snapshot_id}` : `Preview · ${snapshot.snapshot_state}`;
  badge.classList.add(published ? "published" : "draft");

  const comparison = snapshot.comparison;
  document.querySelector("#headline-metrics").innerHTML = [
    metric("Resolved", comparison?.resolved ?? null, "Focusa / baseline"),
    metric("Focusa uplift", comparison?.uplift_score ?? null, "Measured ratio"),
    metric("Cost per resolved", comparison?.cost_per_resolved_delta ?? null, "Measured delta"),
    metric("Grounded claims", comparison?.grounded_claim_delta ?? null, "Measured delta"),
    metric("Operator burden", comparison?.operator_burden_delta ?? null, "Measured delta"),
  ].join("");

  const empty = document.querySelector("#empty-state");
  if (!published) {
    empty.hidden = false;
    empty.textContent = snapshot.empty_state_message;
  }

  const trend = snapshot.trend ?? [];
  const chart = document.querySelector("#trend-chart");
  if (!trend.length) {
    chart.innerHTML = '<p class="trend-empty">No public measured trend yet.</p>';
  } else {
    chart.innerHTML = trend.map((point) => `<div class="bar" style="height:${Math.max(2, point.value * 100)}%" title="${point.label}: ${point.value}"></div>`).join("");
    chart.setAttribute("aria-label", `Measured trend with ${trend.length} points`);
  }
  document.querySelector("#trend-label").textContent = `${trend.length} public points`;

  const failures = snapshot.failure_to_fix ?? [];
  document.querySelector("#failure-board").innerHTML = failures.length
    ? failures.map((item) => `<article class="failure-card"><p class="status">${item.status}</p><h3>${item.failure_class}</h3><p>${item.observed_failure}</p><p><strong>Candidate:</strong> ${item.improvement_candidate}</p><p><strong>Rerun:</strong> ${text(item.rerun_result, "Pending")}</p></article>`).join("")
    : '<p class="empty">No public-safe failure cards have passed the publication gate.</p>';

  const replays = snapshot.replays ?? [];
  document.querySelector("#replay-list").innerHTML = replays.length
    ? replays.map((run) => `<article class="replay"><strong>${run.task_prompt_hash}</strong><p>${run.arm} · ${run.judge_result}</p><p>Redaction: ${run.redaction_status}</p></article>`).join("")
    : '<p class="empty">No redacted public replay is available.</p>';

  const evidence = snapshot.evidence ?? {};
  document.querySelector("#evidence-bundle").innerHTML = Object.entries(evidence).map(([key, value]) => `<div><dt>${key.replaceAll("_", " ")}</dt><dd>${text(value)}</dd></div>`).join("");

  const honesty = snapshot.honesty;
  document.querySelector("#honesty-rail").innerHTML = [
    ["Claim state", honesty.claim_state], ["Split", honesty.data_split], ["Model", honesty.model_version],
    ["Scoring", honesty.scoring_commit], ["Environment", honesty.environment_digest], ["Redaction", snapshot.redaction_status],
  ].map(([key, value]) => `<div><dt>${key}</dt><dd>${text(value)}</dd></div>`).join("");
}

loadSnapshot().then(render).catch((error) => {
  document.querySelector("#snapshot-badge").textContent = "Snapshot unavailable";
  const empty = document.querySelector("#empty-state");
  empty.hidden = false;
  empty.textContent = `No benchmark claim rendered: ${error.message}`;
});
