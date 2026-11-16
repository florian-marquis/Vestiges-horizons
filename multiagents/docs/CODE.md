# Carte du code Duo

Vue d’ensemble pour un agent qui doit modifier ou dépanner Duo.

## Entrée

- `start.bat` → venv + `python -m duo`
- `src/duo/__main__.py` → Uvicorn sur `Settings.host:port` (défaut `127.0.0.1:8765`)
- `src/duo/api.py` → `create_app()` : middleware localhost, routes salon + orchestrateur, static

## Salon (chemin principal)

```
UI salon.js ──POST /api/rooms──► Bus.create_room
Agents ──wait/messages──► Bus (SQLite bus.sqlite)
Bus.consignes ──► textes collés dans chaque fenêtre
```

- `Bus` (`bus.py`) : tables `rooms`, `room_messages`, `room_tokens`, `presence`
- Rôles : `pilote` + `agent1`…`agent5`
- Visibilité : `message_visible_to()` (broadcast vs `@role`)
- Migration : anciens salons duo → `room_tokens` + `agents_count`

## Orchestrateur (chemin secondaire)

```
UI app.js ──POST /api/sessions──► Orchestrator ──► adapters CLI
```

- `orchestrator.py` : tours, phases, bilan
- `adapters/cursor_cli.py`, `copilot_cli.py`, `simulated.py`
- Persistance : `db.py` → `data/duo.sqlite`

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
