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

function renderSlots() {
  const count = Math.max(1, Math.min(5, Number($("agents-count").value) || 1));
  $("agents-count").value = String(count);
  const box = $("slots");
  const previous = [...box.querySelectorAll("select")].map((el) => el.value);
  box.replaceChildren();
  for (let index = 1; index <= count; index += 1) {
    const label = document.createElement("label");
    label.textContent = `Fenêtre Agent ${index}`;
    const select = document.createElement("select");
    select.dataset.index = String(index);
    select.innerHTML = `
      <option value="vscode">VS Code</option>
      <option value="cursor">Cursor</option>
    `;
    select.value = previous[index - 1] || "vscode";
    label.append(select);
    box.append(label);
  }
}

function collectSlots() {
  return [...$("slots").querySelectorAll("select")].map((select, index) => ({
    role: `agent${index + 1}`,
    window: select.value,
  }));
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
  const key = `${room.id}:${room.status}:${lastSeq}:${room.agents_count || 1}`;
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

$("agents-count").addEventListener("input", renderSlots);
$("agents-count").addEventListener("change", renderSlots);
renderSlots();

$("new-room").addEventListener("submit", async (event) => {
  event.preventDefault();
  flash("");
  try {
    const created = await api("/api/rooms", {
      method: "POST",
      body: JSON.stringify({
        task: $("task-input").value,
        agents_count: Number($("agents-count").value) || 1,
        slots: collectSlots(),
      }),
    });
    $("task-input").value = "";
    currentId = created.room.id;
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

refreshList().catch((error) => flash(error.message));
