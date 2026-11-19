const LABELS = {
  cursor: "Cursor",
  copilot: "GitHub Copilot",
  vibe: "Mistral Vibe",
  antigravity: "Google Antigravity",
};

function $(id) {
  return document.getElementById(id);
}

function flash(message) {
  const node = $("flash");
  node.textContent = message || "";
}

async function api(path, options = {}) {
  const headers = { ...(options.headers || {}) };
  if (options.method && options.method !== "GET") {
    headers["X-Duo-Request"] = "1";
    headers["Content-Type"] = headers["Content-Type"] || "application/json";
  }
  const response = await fetch(path, { ...options, headers });
  const text = await response.text();
  let body = null;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    body = { detail: text };
  }
  if (!response.ok) {
    throw new Error((body && body.detail) || response.statusText);
  }
  return body;
}

function authLabel(value) {
  if (value === true) return "authentifié";
  if (value === false) return "non authentifié";
  return "indetermine";
}

function quotaLine(item) {
  const li = document.createElement("li");
  if (item.status === "live" || item.status === "manual") {
    const parts = [];
    if (item.remaining != null && item.limit != null) {
      parts.push(`${item.remaining} / ${item.limit} restants`);
    } else if (item.remaining != null) {
      parts.push(`${item.remaining} restants`);
    } else if (item.used != null && item.limit != null) {
      parts.push(`${item.used} / ${item.limit}`);
    }
    if (item.detail) parts.push(item.detail);
    if (item.status === "manual") parts.push("(manuel)");
    li.textContent = `${item.label} : ${parts.join(" — ") || "ok"}`;
  } else {
    li.textContent = `${item.label} : indisponible — ${item.detail || "pas de chiffre inventé"}`;
  }
  return li;
}

function renderProvider(provider) {
  const article = document.createElement("article");
  article.className = `provider-card ${provider.id}`;
  const title = document.createElement("h2");
  title.textContent = LABELS[provider.id] || provider.label || provider.id;
  const status = document.createElement("p");
  status.className = "provider-status";
  if (!provider.available) {
    status.classList.add("bad");
    status.textContent = "CLI absente";
  } else {
    status.classList.add(provider.authenticated === true ? "ok" : "warn");
    status.textContent = authLabel(provider.authenticated);
  }
  const detail = document.createElement("p");
  detail.className = "muted";
  detail.textContent = provider.detail || "";
  const hint = document.createElement("p");
  hint.className = "muted";
  hint.textContent = provider.login_hint || "";

  const models = document.createElement("ul");
  models.className = "model-list";
  for (const model of provider.models || []) {
    const li = document.createElement("li");
    const extra = [model.tier, model.cost_hint].filter(Boolean).join(" · ");
    li.textContent = extra
      ? `Modèle : ${model.label || model.id} [${extra}]`
      : `Modèle : ${model.label || model.id}`;
    models.append(li);
  }

  const quotas = document.createElement("ul");
  quotas.className = "quota-list";
  for (const item of provider.quotas || []) {
    quotas.append(quotaLine(item));
  }

  const actions = document.createElement("div");
  actions.className = "actions";
  const login = document.createElement("button");
  login.type = "button";
  login.textContent = "Connexion OAuth";
  login.addEventListener("click", async () => {
    try {
      const result = await api(`/api/providers/${provider.id}/login`, { method: "POST", body: "{}" });
      flash(result.detail || "Login lancé.");
      await load();
    } catch (error) {
      flash(error.message);
    }
  });
  const logout = document.createElement("button");
  logout.type = "button";
  logout.textContent = "Déconnexion";
  logout.addEventListener("click", async () => {
    try {
      const result = await api(`/api/providers/${provider.id}/logout`, { method: "POST", body: "{}" });
      flash(result.detail || "Déconnexion.");
      await load();
    } catch (error) {
      flash(error.message);
    }
  });
  const refresh = document.createElement("button");
  refresh.type = "button";
  refresh.textContent = "Quotas";
  refresh.addEventListener("click", async () => {
    try {
      const data = await api(`/api/providers/${provider.id}/quotas`);
      quotas.replaceChildren();
      for (const item of data.quotas || []) {
        quotas.append(quotaLine(item));
      }
      flash(`Quotas ${provider.id} rafraîchis.`);
    } catch (error) {
      flash(error.message);
    }
  });
  actions.append(login, logout, refresh);

  const overrideBox = document.createElement("div");
  overrideBox.className = "inline-override";
  const pctInput = document.createElement("input");
  pctInput.type = "number";
  pctInput.min = "0";
  pctInput.max = "100";
  pctInput.placeholder = "Quota restant manuel (%) — ex. 75";
  const modelSelect = document.createElement("select");
  for (const model of provider.models || []) {
    const opt = document.createElement("option");
    opt.value = model.id;
    opt.textContent = `${model.label || model.id} (${model.cost_hint || model.tier || "standard"})`;
    modelSelect.append(opt);
  }
  if (provider.preferred_model) {
    modelSelect.value = provider.preferred_model;
  }
  const newModelInput = document.createElement("input");
  newModelInput.type = "text";
  newModelInput.placeholder = "Ajouter modèle dispo (ex. mistral-large-4-0-3)";
  const newTierSelect = document.createElement("select");
  for (const [val, text] of [
    ["light", "Tier : light (N1)"],
    ["standard", "Tier : standard (N2)"],
    ["deep", "Tier : deep (N3)"],
    ["critical", "Tier : critical (N4)"],
  ]) {
    const opt = document.createElement("option");
    opt.value = val;
    opt.textContent = text;
    newTierSelect.append(opt);
  }
  newTierSelect.value = "standard";
  const newCostInput = document.createElement("input");
  newCostInput.type = "text";
  newCostInput.placeholder = "Coût optionnel (ex. 0x, 1x, élevé)";

  const saveOverride = document.createElement("button");
  saveOverride.type = "button";
  saveOverride.className = "ghost";
  saveOverride.textContent = "Enregistrer quota / modèles";
  saveOverride.addEventListener("click", async () => {
    try {
      const payload = { preferred_model: modelSelect.value || undefined };
      const rawPct = pctInput.value.trim();
      if (rawPct !== "") {
        const val = Math.max(0, Math.min(100, Number(rawPct)));
        payload.quota_override = {
          remaining: val,
          limit: 100,
          detail: `${val} % restants (manuel)`,
        };
      }
      const addedId = newModelInput.value.trim();
      if (addedId) {
        payload.add_models = [
          {
            id: addedId,
            label: addedId,
            tier: newTierSelect.value || "standard",
            cost_hint: newCostInput.value.trim(),
          },
        ];
        payload.preferred_model = addedId;
      }
      await api(`/api/providers/${provider.id}/sync`, {
        method: "POST",
        body: JSON.stringify(payload),
      });
      flash(`Réglage ${provider.id} enregistré.`);
      await load();
    } catch (error) {
      flash(error.message);
    }
  });
  overrideBox.append(modelSelect, pctInput, newModelInput, newTierSelect, newCostInput, saveOverride);

  const meta = document.createElement("p");
  meta.className = "muted";
  meta.textContent = provider.last_check_at
    ? `Dernier contrôle : ${provider.last_check_at}`
    : "Pas encore contrôlé.";

  article.append(title, status, detail, hint, models, quotas, actions, overrideBox, meta);
  return article;
}

async function load() {
  const data = await api("/api/providers");
  const root = $("provider-list");
  root.replaceChildren();
  for (const provider of data.providers || []) {
    root.append(renderProvider(provider));
  }
}

$("refresh-all").addEventListener("click", () => {
  load().then(() => flash("Statuts rafraîchis.")).catch((error) => flash(error.message));
});

load().catch((error) => flash(error.message));
