const headers = { "Content-Type": "application/json", "X-Duo-Request": "1" };
const phaseNames = {
  received: "Réception",
  analyze: "Analyse",
  critique: "Critique",
  respond: "Réponse",
  revise: "Révision",
  implement: "Implémentation",
  review: "Relecture",
  test: "Tests",
  correct: "Correction",
  verify: "Vérification",
  done: "Terminé",
};
const statusNames = {
  created: "créée",
  running: "en cours",
  paused: "en pause",
  succeeded: "réussie",
  failed: "échouée",
  interrupted: "interrompue",
  needs_arbitration: "à arbitrer",
  error: "en erreur",
  available: "disponible",
  waiting: "en attente",
  finished: "terminé",
};

let currentId = null;
let source = null;
let poll = null;

const $ = (id) => document.getElementById(id);

function flash(text) {
  $("flash").textContent = text || "";
}

async function api(path, options = {}) {
  const response = await fetch(path, {
    ...options,
    headers: { ...headers, ...(options.headers || {}) },
  });
  const isJson = (response.headers.get("content-type") || "").includes("json");
  const body = isJson ? await response.json() : await response.text();
  if (!response.ok) {
    const detail = body && body.detail ? body.detail : "La requête a échoué.";
    throw new Error(typeof detail === "string" ? detail : "La requête a échoué.");
  }
  return body;
}

function badge(text, className) {
  const span = document.createElement("span");
  span.className = `badge ${className || ""}`;
  span.textContent = text;
  return span;
}

const PROVIDER_NAMES = ["cursor", "copilot", "vibe", "antigravity"];

function renderHealth(data) {
  for (const name of PROVIDER_NAMES) {
    const item = data.adapters[name];
    const node = $(`health-${name}`);
    if (!node || !item) continue;
    node.className = "pill";
    if (!item.available) {
      node.classList.add("bad");
      node.textContent = `${name} absent`;
    } else if (item.authenticated === false) {
      node.classList.add("warn");
      node.textContent = `${name} non authentifié`;
    } else if (item.authenticated === null) {
      node.classList.add("warn");
      node.textContent = `${name} non vérifié`;
    } else {
      node.classList.add("ok");
      node.textContent = `${name} disponible`;
    }
    node.title = item.detail;
  }
}

function renderQuotas(providers) {
  const panel = $("quota-panel");
  if (!panel) return;
  panel.replaceChildren();
  for (const provider of providers || []) {
    const title = document.createElement("li");
    title.className = "quota-provider";
    title.textContent = provider.label || provider.id;
    panel.append(title);
    for (const item of provider.quotas || []) {
      const li = document.createElement("li");
      if (item.status === "live") {
        const bits = [];
        if (item.remaining != null) bits.push(`${item.remaining} restants`);
        if (item.limit != null) bits.push(`limite ${item.limit}`);
        if (item.detail) bits.push(item.detail);
        li.textContent = `${item.label} : ${bits.join(" · ") || "ok"}`;
      } else {
        li.textContent = `${item.label} : indisponible`;
        li.title = item.detail || "";
      }
      panel.append(li);
    }
  }
}

async function refreshHealthAndQuotas() {
  const [health, providers] = await Promise.all([
    api("/api/health", { method: "GET", headers: {} }),
    api("/api/providers", { method: "GET", headers: {} }),
  ]);
  renderHealth(health);
  renderQuotas(providers.providers);
}

function renderSessions(items) {
  const list = $("session-list");
  list.replaceChildren();
  for (const session of items) {
    const li = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.className = `session-item${session.id === currentId ? " active" : ""}`;
    button.textContent = `${session.title} — ${statusNames[session.status] || session.status}`;
    button.addEventListener("click", () => selectSession(session.id));
    li.append(button);
    list.append(li);
  }
}

function renderMessage(message) {
  const article = document.createElement("article");
  article.className = `message ${message.sender}`;
  const header = document.createElement("header");
  header.append(
    badge(message.sender),
    badge(message.type),
    badge(`tour ${message.turn}`),
  );
  const body = document.createElement("pre");
  body.textContent = message.content;
  article.append(header, body);
  return article;
}

function definition(target, pairs) {
  target.replaceChildren();
  for (const [key, value] of pairs) {
    const dt = document.createElement("dt");
    const dd = document.createElement("dd");
    dt.textContent = key;
    dd.textContent = value;
    target.append(dt, dd);
  }
}

function renderSnapshot(data) {
  const session = data.session;
  $("session-kicker").textContent = `${statusNames[session.status] || session.status} · ${phaseNames[session.phase] || session.phase}`;
  $("session-title").textContent = session.title;
  const chat = $("chat");
  const stick = chat.scrollHeight - chat.scrollTop - chat.clientHeight < 40;
  chat.replaceChildren(...data.messages.map(renderMessage));
  if (stick) chat.scrollTop = chat.scrollHeight;
  definition($("session-state"), [
    ["Tour", `${session.turn} / ${session.max_turns}`],
    ["Restants", String(session.turns_remaining)],
    ["Agent actif", session.active_agent || "—"],
    ["Écoulé", `${session.elapsed_seconds} s`],
    ["Reste", `${session.seconds_remaining} s`],
    ["Mode", session.mode],
  ]);
  const agents = $("agent-state");
  agents.replaceChildren();
  for (const agent of data.agents) {
    const li = document.createElement("li");
    li.textContent = `${agent.agent} : ${statusNames[agent.status] || agent.status}${agent.detail ? " — " + agent.detail : ""}`;
    agents.append(li);
  }
  const tasks = $("task-list");
  tasks.replaceChildren();
  for (const task of data.tasks) {
    const li = document.createElement("li");
    li.textContent = `${task.title} — ${task.status}`;
    tasks.append(li);
  }
  const changes = $("changes");
  const files = (session.final_report && session.final_report.files_modified) || [];
  const tests = data.tests || [];
  changes.textContent = [
    files.length ? `Fichiers : ${files.join(", ")}` : "Aucun fichier modifié pour l'instant.",
    ...tests.map((item) => `Test ${item.command} → code ${item.exit_code}\n${item.stdout || ""}`),
  ].join("\n\n");
  const report = $("report");
  const finalReport = session.final_report;
  if (!finalReport) {
    report.textContent = session.last_error || "Le bilan sera publié à la fin de la session.";
  } else {
    const open = (finalReport.disagreements_open || []).join(" | ") || "aucun";
    const resolved = (finalReport.disagreements_resolved || []).join(" | ") || "aucun";
    report.textContent = [
      `Résultat : ${finalReport.outcome}`,
      `Solution : ${finalReport.solution || "—"}`,
      `Accords : ${resolved}`,
      `Désaccords ouverts : ${open}`,
      `Limites : ${(finalReport.limitations || []).join(" ") || "aucune"}`,
      `À faire : ${(finalReport.remaining_actions || []).join(" ") || "rien"}`,
    ].join("\n\n");
  }
  const running = session.status === "running" || session.status === "created";
  $("pause-button").disabled = !running;
  $("resume-button").disabled = !["paused", "interrupted"].includes(session.status);
  $("stop-button").disabled = ["succeeded", "failed", "needs_arbitration", "interrupted"].includes(session.status);
  $("retry-button").disabled = session.status !== "error";
  $("export-json").disabled = false;
  $("export-md").disabled = false;
}

async function refreshList() {
  renderSessions(await api("/api/sessions", { method: "GET", headers: {} }));
}

async function selectSession(id) {
  currentId = id;
  if (source) source.close();
  if (poll) clearInterval(poll);
  const data = await api(`/api/sessions/${id}`, { method: "GET", headers: {} });
  renderSnapshot(data);
  await refreshList();
  source = new EventSource(`/api/sessions/${id}/events`);
  source.addEventListener("update", async () => {
    if (currentId !== id) return;
    renderSnapshot(await api(`/api/sessions/${id}`, { method: "GET", headers: {} }));
    await refreshList();
  });
  poll = setInterval(async () => {
    if (!currentId) return;
    const latest = await api(`/api/sessions/${currentId}`, { method: "GET", headers: {} });
    renderSnapshot(latest);
    if (!["running", "created"].includes(latest.session.status)) {
      clearInterval(poll);
      poll = null;
    }
  }, 1500);
}

async function command(name) {
  if (!currentId) return;
  try {
    renderSnapshot(await api(`/api/sessions/${currentId}/${name}`, { method: "POST", body: "{}" }));
    flash("");
    await refreshList();
  } catch (error) {
    flash(error.message);
  }
}

$("new-session").addEventListener("submit", async (event) => {
  event.preventDefault();
  flash("");
  const mode = $("mode").value;
  const payload = {
    task: $("task-input").value,
    mode,
    scenario: mode === "simulated" ? $("scenario").value : "collaboration",
    proposer: $("proposer").value,
    critic: $("critic").value,
    implementer: $("implementer").value,
    max_turns: Number($("max-turns").value),
    max_duration_seconds: Number($("max-duration").value),
    agent_timeout_seconds: Number($("timeout").value),
    allow_implementation: $("allow-impl").checked,
    wait: false,
  };
  try {
    const data = await api("/api/sessions", { method: "POST", body: JSON.stringify(payload) });
    $("task-input").value = "";
    await selectSession(data.session.id);
  } catch (error) {
    flash(error.message);
  }
});

$("pause-button").addEventListener("click", () => command("pause"));
$("resume-button").addEventListener("click", () => command("resume"));
$("stop-button").addEventListener("click", () => command("stop"));
$("retry-button").addEventListener("click", () => command("retry"));
$("export-json").addEventListener("click", () => {
  if (currentId) window.open(`/api/sessions/${currentId}/export?format=json`, "_blank");
});
$("export-md").addEventListener("click", () => {
  if (currentId) window.open(`/api/sessions/${currentId}/export?format=md`, "_blank");
});

const refreshQuotasBtn = $("refresh-quotas");
if (refreshQuotasBtn) {
  refreshQuotasBtn.addEventListener("click", () => {
    refreshHealthAndQuotas().then(() => flash("Santé et quotas rafraîchis.")).catch((error) => flash(error.message));
  });
}

refreshHealthAndQuotas().catch((error) => flash(error.message));
refreshList().catch((error) => flash(error.message));
