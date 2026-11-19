# Carte du code Duo

Vue d’ensemble pour un agent qui doit modifier ou dépanner Duo.

## Entrée

- `start.bat` → venv + `python -m duo`
- `src/duo/__main__.py` → Uvicorn sur `Settings.host:port` (défaut `127.0.0.1:8765`)
- `src/duo/api.py` → `create_app()` : middleware localhost, routes salon + orchestrateur + comptes, static

## Salon (chemin principal)

```
UI salon.js ──POST /api/rooms──► Bus.create_room
Agents ──wait/messages──► Bus (SQLite bus.sqlite)
Bus.consignes ──► textes collés dans chaque fenêtre
```

- `Bus` (`bus.py`) : tables `rooms` (dont `pilote_slot_json` / `slots_json`), `room_messages`, `room_tokens`, `presence` ; switch à chaud `update_slot()`
- Rôles : `pilote` + `agent1`…`agent5` (chacun avec `provider`, `window`, `model`)
- Visibilité : `message_visible_to()` (broadcast vs `@role`)
- Migration : anciens salons duo → `room_tokens` + `agents_count` + `pilote_slot_json`
- Attente opaque : `wait_room.py` (CLI + MCP) enchaîne les polls `/wait` jusqu’à message ou clôture ; `mcp_bus.py` expose aussi `etat_quotas_modeles`, `changer_slot` et `apprendre_modeles`

## Orchestrateur (chemin secondaire)

```
UI app.js ──POST /api/sessions──► Orchestrator ──► adapters CLI
```

- `orchestrator.py` : tours, phases, bilan (agents `cursor` / `copilot` / `vibe` / `antigravity`)
- `adapters/cursor_cli.py`, `copilot_cli.py`, `vibe_cli.py`, `antigravity_cli.py`, `simulated.py`
- `providers/*` : statut auth, login OAuth CLI, modèles avec tiers/coûts, quotas best-effort + sync Antigravity (`parse_antigravity_swarm_payload`) + routage (`compute_routing`) ; UI `/comptes`, routes `/api/routing`, `/api/providers/{id}/sync`, `PATCH /api/rooms/{id}/slots`
- Persistance : `db.py` → `data/duo.sqlite` ; métadonnées providers non secrètes → `providers_meta.json`

## Sécurité

- Écoute loopback uniquement (`config.py`)
- Mutations API : header `X-Duo-Request: 1` + origine locale (`security.py`)
- Jetons par rôle : un agent ne publie pas sous un autre nom

## Tests

```bat
set PYTHONPATH=src
.venv\Scripts\python -m pytest
```

Priorité salon : `tests/test_bus.py`.
