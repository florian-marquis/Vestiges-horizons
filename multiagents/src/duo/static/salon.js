const headers = { "Content-Type": "application/json", "X-Duo-Request": "1" };
const fallbackLabels = {
  pilote: "Pilote",
  humain: "Vous",
};

let currentId = null;
let poll = null;
let renderedKey = "";
let currentLabels = { ...fallbackLabels };
let currentPrompts = {};

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

function labelOf(sender, room) {
  if (room && room.labels && room.labels[sender]) return room.labels[sender];
  if (sender && sender.startsWith("agent")) {
    return `Agent ${sender.slice(5)}`;
  }
  return currentLabels[sender] || fallbackLabels[sender] || sender;
}

function roleClass(sender) {
  if (sender === "pilote") return "pilote";
  if (sender && sender.startsWith("agent")) {
    const n = Number(sender.slice(5)) || 1;
    return `agent1 agent-n-${((n - 1) % 5) + 1}`;
  }
  return sender;
}

const PROVIDER_OPTIONS = [
  { provider: "antigravity", window: "antigravity", label: "Google Antigravity" },
  { provider: "copilot", window: "vscode", label: "GitHub Copilot (VS Code)" },
  { provider: "vibe", window: "vscode", label: "Mistral Vibe (VS Code)" },
  { provider: "cursor", window: "cursor", label: "Cursor" },
];

const FALLBACK_MODELS = {
  antigravity: [
    { id: "gemini-pro-orchestrator-g1-pro", label: "Orchestrator (Argon)", tier: "deep", cost_hint: "quota hebdo" },
    { id: "gemini-4-hydrogen-medium", label: "Gemini 4 Hydrogen (Medium)", tier: "standard", cost_hint: "quota 5h" },
    { id: "gemini-3.8-flash-high", label: "Gemini 3.8 Flash (High)", tier: "light", cost_hint: "quota 5h (éco)" },
    { id: "claude-opus-5-5-high", label: "Claude Opus 5.5 (High)", tier: "critical", cost_hint: "quota 5h (coûteux)" },
  ],
  copilot: [
    { id: "gpt-4.1", label: "GPT-4.1", tier: "standard", cost_hint: "0x (inclus)" },
    { id: "gemini-3-flash", label: "Gemini 3 Flash", tier: "standard", cost_hint: "0.33x" },
    { id: "claude-sonnet-4.6", label: "Claude Sonnet 4.6", tier: "deep", cost_hint: "1x" },
    { id: "claude-opus-4.6", label: "Claude Opus 4.6", tier: "critical", cost_hint: "3x" },
  ],
  vibe: [
    { id: "devstral-small-latest", label: "Devstral Small", tier: "light", cost_hint: "éco" },
    { id: "devstral-medium-latest", label: "Devstral Medium", tier: "standard", cost_hint: "standard" },
    { id: "codestral-latest", label: "Codestral", tier: "standard", cost_hint: "standard" },
    { id: "mistral-large-latest", label: "Mistral Large", tier: "deep", cost_hint: "élevé" },
    { id: "mistral-large-4-0-3", label: "Mistral Large 4", tier: "deep", cost_hint: "élevé" },
  ],
  cursor: [
    { id: "auto", label: "Auto / Cursor 2.5", tier: "standard", cost_hint: "quota maison" },
    { id: "grok-4.7", label: "Grok 4.7", tier: "standard", cost_hint: "quota maison" },
    { id: "claude-4.6-sonnet", label: "Claude Sonnet (Cursor)", tier: "deep", cost_hint: "quota extérieur" },
    { id: "claude-4.6-opus", label: "Claude Opus (Cursor)", tier: "critical", cost_hint: "quota extérieur" },
  ],
};

let providersCache = {};

function windowForProvider(provider) {
  const found = PROVIDER_OPTIONS.find((item) => item.provider === provider);
  return found ? found.window : "vscode";
}

function modelsForProvider(provider) {
  const snap = providersCache[provider];
  if (snap && Array.isArray(snap.models) && snap.models.length) {
    return snap.models;
  }
  return FALLBACK_MODELS[provider] || [{ id: "default", label: "Défaut", tier: "standard", cost_hint: "" }];
}

function fillModelOptions(select, provider, selectedModel) {
  const models = modelsForProvider(provider);
  select.replaceChildren();
  let matched = false;
  for (const model of models) {
    const option = document.createElement("option");
    option.value = model.id;
    const extra = [model.tier, model.cost_hint].filter(Boolean).join(" · ");
    option.textContent = extra ? `${model.label || model.id} [${extra}]` : (model.label || model.id);
    if (selectedModel && model.id === selectedModel) {
      matched = true;
    }
    select.append(option);
  }
  if (selectedModel && !matched) {
    const custom = document.createElement("option");
    custom.value = selectedModel;
    custom.textContent = selectedModel;
    select.append(custom);
  }
  if (selectedModel) {
    select.value = selectedModel;
  } else if (models.length) {
    select.value = models[0].id;
  }
}

function buildSlotCard(role, titleText, initialProvider, initialModel) {
  const card = document.createElement("div");
  card.className = "slot-card";
  card.dataset.role = role;

  const header = document.createElement("div");
  header.className = "slot-card-header";
  header.textContent = titleText;

  const providerLabel = document.createElement("label");
  providerLabel.textContent = "Outil / fenêtre";
  const providerSelect = document.createElement("select");
  providerSelect.className = "slot-provider";
  for (const opt of PROVIDER_OPTIONS) {
    const option = document.createElement("option");
    option.value = opt.provider;
    option.textContent = opt.label;
    providerSelect.append(option);
  }
  providerSelect.value = initialProvider || (role === "pilote" ? "antigravity" : "copilot");
  providerLabel.append(providerSelect);

  const modelLabel = document.createElement("label");
  modelLabel.textContent = "Modèle";
  const modelSelect = document.createElement("select");
  modelSelect.className = "slot-model";
  fillModelOptions(modelSelect, providerSelect.value, initialModel);
  modelLabel.append(modelSelect);

  providerSelect.addEventListener("change", () => {
    fillModelOptions(modelSelect, providerSelect.value, null);
  });

  card.append(header, providerLabel, modelLabel);
  return { card, providerSelect, modelSelect };
}

function defaultProviderForAgent(index, count) {
  if (count >= 3) {
    if (index === 1) return "cursor";
    if (index === 2) return "copilot";
    if (index === 3) return "vibe";
  }
  if (index === 1) return "copilot";
  if (index === 2) return "vibe";
  return "copilot";
}

function renderPiloteSlot(forcedProvider, forcedModel) {
  const box = $("pilote-slot");
  const prevProv = forcedProvider || box.querySelector(".slot-provider")?.value || "antigravity";
  const prevMod = forcedModel || (forcedProvider ? null : box.querySelector(".slot-model")?.value) || null;
  box.replaceChildren();
  const { card } = buildSlotCard("pilote", "Pilote (@pilote)", prevProv, prevMod);
  box.append(card);
}

function renderSlots(forcedByRole) {
  const count = Math.max(1, Math.min(5, Number($("agents-count").value) || 1));
  $("agents-count").value = String(count);
  const box = $("slots");
  const previous = [...box.querySelectorAll(".slot-card")].map((el) => ({
    provider: el.querySelector(".slot-provider")?.value,
    model: el.querySelector(".slot-model")?.value,
  }));
  box.replaceChildren();
  for (let index = 1; index <= count; index += 1) {
    const role = `agent${index}`;
    const forced = forcedByRole && forcedByRole[role];
    const prev = previous[index - 1] || {};
    const provider = (forced && forced.provider) || prev.provider || defaultProviderForAgent(index, count);
    const model = (forced && forced.model) || (forced ? null : prev.model) || null;
    const { card } = buildSlotCard(role, `Agent ${index} (@${role})`, provider, model);
    box.append(card);
  }
}

function collectPiloteSlot() {
  const card = $("pilote-slot").querySelector(".slot-card");
  const provider = card?.querySelector(".slot-provider")?.value || "antigravity";
  const model = card?.querySelector(".slot-model")?.value || "";
  return {
    role: "pilote",
    provider,
    window: windowForProvider(provider),
    model: model || undefined,
  };
}

function collectSlots() {
  return [...$("slots").querySelectorAll(".slot-card")].map((card, index) => {
    const provider = card.querySelector(".slot-provider")?.value || "copilot";
    const model = card.querySelector(".slot-model")?.value || "";
    return {
      role: `agent${index + 1}`,
      provider,
      window: windowForProvider(provider),
      model: model || undefined,
    };
  });
}

function renderList(rooms) {
  const list = $("room-list");
  list.replaceChildren();
  if (!rooms.length) {
    const item = document.createElement("li");
    item.textContent = "Aucun salon pour l'instant.";
    list.append(item);
    return;
  }
  for (const room of rooms) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.className = `session-item${room.id === currentId ? " active" : ""}`;
    const n = room.agents_count || 1;
    button.textContent = `${room.status === "open" ? "ouvert" : "clos"} · Pilote + ${n} agent${n > 1 ? "s" : ""} — ${room.title}`;
    button.addEventListener("click", () => selectRoom(room.id));
    item.append(button);
    list.append(item);
  }
}

function renderActiveSlots(room) {
  const container = $("active-slots");
  if (!container) return;
  container.replaceChildren();
  const allSlots = room.all_slots || (room.pilote_slot ? [room.pilote_slot, ...(room.slots || [])] : (room.slots || []));
  for (const slot of allSlots) {
    const role = slot.role || "agent1";
    const { card, providerSelect, modelSelect } = buildSlotCard(
      role,
      `${labelOf(role, room)} (@${role})`,
      slot.provider || "copilot",
      slot.model || null
    );
    const applyBtn = document.createElement("button");
    applyBtn.type = "button";
    applyBtn.className = "ghost";
    applyBtn.textContent = `Switcher ${labelOf(role, room)}`;
    applyBtn.disabled = room.status !== "open";
    applyBtn.addEventListener("click", async () => {
      try {
        const provider = providerSelect.value;
        const model = modelSelect.value;
        const result = await api(`/api/rooms/${room.id}/slots`, {
          method: "PATCH",
          body: JSON.stringify({
            role,
            provider,
            window: windowForProvider(provider),
            model,
            reason: "Switch manuel depuis l'UI",
          }),
        });
        if (result.handover) {
          await api(`/api/rooms/${room.id}/messages`, {
            method: "POST",
            body: JSON.stringify({
              sender: "humain",
              text: `${result.handover} (Si besoin, basculer le sélecteur IDE sur ${model}.)`,
            }),
          });
        }
        flash(result.handover || `Slot ${role} mis à jour.`);
        renderedKey = "";
        await selectRoom(room.id);
      } catch (error) {
        flash(error.message);
      }
    });
    card.append(applyBtn);
    container.append(card);
  }
}

function renderPrompts(room, prompts) {
  currentPrompts = prompts || currentPrompts || {};
  const box = $("prompts");
  box.replaceChildren();
  const roles = room.roles || ["pilote", "agent1"];
  for (const role of roles) {
    const section = document.createElement("section");
    section.className = "prompt-block";
    const title = document.createElement("h3");
    title.textContent = `Consigne ${labelOf(role, room)}`;
    const button = document.createElement("button");
    button.type = "button";
    button.className = "ghost";
    button.textContent = `Copier pour ${labelOf(role, room)}`;
    button.addEventListener("click", () => copyPrompt(role));
    const pre = document.createElement("pre");
    pre.className = "consigne";
    pre.id = `prompt-${role}`;
    pre.textContent = currentPrompts[role] || "Ouvrez un salon pour obtenir la consigne.";
    section.append(title, button, pre);
    box.append(section);
  }
}

function renderRoom(room, prompts) {
  currentLabels = { ...fallbackLabels, ...(room.labels || {}) };
  $("room-kicker").textContent = room.status === "open" ? "Salon ouvert" : "Salon clos";
  $("room-title").textContent = room.title;
  $("stop-button").disabled = room.status !== "open";
  $("close-button").disabled = room.status !== "open";
  $("send-button").disabled = room.status !== "open";
  const lastSeq = room.last_seq || 0;
  const slotsSig = JSON.stringify(room.all_slots || room.slots || []);
  const key = `${room.id}:${room.status}:${lastSeq}:${room.agents_count || 1}:${slotsSig}`;
  const chat = $("chat");
  if (key === renderedKey) {
    renderPresence(room);
    if (prompts) renderPrompts(room, prompts);
    return;
  }
  renderedKey = key;
  chat.replaceChildren();
  for (const message of room.messages || []) {
    const article = document.createElement("article");
    article.className = `message ${roleClass(message.sender)}`;
    const header = document.createElement("header");
    header.append(badge(labelOf(message.sender, room), message.sender));
    header.append(badge(`#${message.seq}`));
    const body = document.createElement("pre");
    body.textContent = message.text;
    article.append(header, body);
    chat.append(article);
  }
  chat.scrollTop = chat.scrollHeight;
  renderPresence(room);
  renderActiveSlots(room);
  renderPrompts(room, prompts || currentPrompts);
}

function renderPresence(room) {
  const presence = $("presence");
  presence.replaceChildren();
  const roles = room.roles || ["pilote", "agent1"];
  for (const agent of roles) {
    const state = (room.presence && room.presence[agent]) || {};
    const item = document.createElement("li");
    item.append(badge(state.label || labelOf(agent, room)));
    item.append(document.createTextNode(state.connected ? " en attente ou vient d'écrire" : " pas encore connecté"));
    presence.append(item);
  }
}

async function refreshProviders() {
  try {
    const data = await api("/api/providers");
    const next = {};
    for (const provider of data.providers || []) {
      next[provider.id] = provider;
    }
    providersCache = next;
    renderPiloteSlot();
    renderSlots();
  } catch {
    // Repli sur FALLBACK_MODELS si indisponible.
  }
}

async function refreshList() {
  renderList(await api("/api/rooms"));
}

async function selectRoom(id) {
  currentId = id;
  const [room, prompts] = await Promise.all([
    api(`/api/rooms/${id}`),
    api(`/api/rooms/${id}/consignes`),
  ]);
  renderRoom(room, prompts);
  await refreshList();
  if (poll) clearInterval(poll);
  poll = setInterval(() => reloadRoom(), 1500);
}

async function reloadRoom() {
  if (!currentId) return;
  try {
    renderRoom(await api(`/api/rooms/${currentId}`));
  } catch (error) {
    flash(error.message);
  }
}

async function copyPrompt(role) {
  const node = $(`prompt-${role}`);
  const text = node ? node.textContent : "";
  try {
    await navigator.clipboard.writeText(text);
    flash(`Consigne ${labelOf(role)} copiée.`);
  } catch (error) {
    flash("Copie impossible depuis ce navigateur. Sélectionnez le texte à la main.");
  }
}

$("agents-count").addEventListener("input", () => renderSlots());
$("agents-count").addEventListener("change", () => renderSlots());
renderPiloteSlot();
renderSlots();

$("apply-routing-button")?.addEventListener("click", async () => {
  try {
    const difficulty = $("difficulty-select")?.value || "N2";
    const curPilote = $("pilote-slot").querySelector(".slot-provider")?.value || "antigravity";
    const data = await api(`/api/routing?difficulty=${encodeURIComponent(difficulty)}&pilote_provider=${encodeURIComponent(curPilote)}`);
    if (data.pilote) {
      renderPiloteSlot(data.pilote.provider, data.pilote.model);
    }
    const forced = {};
    if (data.session_profile) {
      for (const key of ["escalation_agent", "simple_agent", "complex_agent"]) {
        const entry = data.session_profile[key];
        if (entry && entry.role) {
          forced[entry.role] = entry;
        }
      }
      renderSlots(forced);
      const sp = data.session_profile;
      flash(
        `Profil stable (${difficulty}, cible @${data.assigned_role || "agent2"}) : ` +
        `Pilote=${sp.pilote?.provider} (${sp.pilote?.model}) | ` +
        `Simples=${sp.simple_agent?.role}:${sp.simple_agent?.model} | ` +
        `Complexes=${sp.complex_agent?.role}:${sp.complex_agent?.model}.`
      );
    } else if (Array.isArray(data.agents) && data.agents.length) {
      for (const item of data.agents) {
        forced[item.role] = item;
      }
      renderSlots(forced);
      flash(`Recommandation ${difficulty} appliquée : Pilote=${data.pilote?.provider} (${data.pilote?.model}).`);
    }
  } catch (error) {
    flash(error.message);
  }
});

$("new-room").addEventListener("submit", async (event) => {
  event.preventDefault();
  flash("");
  try {
    const created = await api("/api/rooms", {
      method: "POST",
      body: JSON.stringify({
        task: $("task-input").value,
        agents_count: Number($("agents-count").value) || 1,
        pilote: collectPiloteSlot(),
        slots: collectSlots(),
      }),
    });
    $("task-input").value = "";
    currentId = created.room.id;
    renderedKey = "";
    renderRoom(created.room, created.consignes);
    await refreshList();
    if (poll) clearInterval(poll);
    poll = setInterval(() => reloadRoom(), 1500);
  } catch (error) {
    flash(error.message);
  }
});

$("human-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!currentId) return;
  const text = $("human-text").value.trim();
  if (!text) return;
  try {
    await api(`/api/rooms/${currentId}/messages`, {
      method: "POST",
      body: JSON.stringify({ sender: "humain", text }),
    });
    $("human-text").value = "";
    await reloadRoom();
  } catch (error) {
    flash(error.message);
  }
});

$("stop-button").addEventListener("click", async () => {
  if (!currentId) return;
  try {
    await api(`/api/rooms/${currentId}/messages`, {
      method: "POST",
      body: JSON.stringify({ sender: "humain", text: "STOP" }),
    });
    await reloadRoom();
  } catch (error) {
    flash(error.message);
  }
});

$("close-button").addEventListener("click", async () => {
  if (!currentId) return;
  try {
    await api(`/api/rooms/${currentId}/close`, { method: "POST" });
    await selectRoom(currentId);
  } catch (error) {
    flash(error.message);
  }
});

refreshProviders();
refreshList().catch((error) => flash(error.message));
