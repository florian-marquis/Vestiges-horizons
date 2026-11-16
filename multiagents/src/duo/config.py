"""Chargement de la configuration locale."""

from __future__ import annotations

import os
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

LOOPBACK_HOSTS = frozenset({"127.0.0.1", "localhost", "::1"})


@dataclass(frozen=True)
class Settings:
    host: str = "127.0.0.1"
    port: int = 8765
    data_dir: Path = Path("data")
    max_turns: int = 8
    max_duration_seconds: int = 600
    agent_timeout_seconds: int = 180
    similarity_threshold: float = 0.92
    max_corrections: int = 1
    allowed_roots: tuple[Path, ...] = field(default_factory=tuple)
    cursor_model: str = "auto"
    copilot_model: str = ""

    @property
    def db_path(self) -> Path:
        return self.data_dir / "duo.sqlite"

    @property
    def sandbox_root(self) -> Path:
        return self.data_dir / "sandboxes"

    @property
    def worktree_root(self) -> Path:
        return self.data_dir / "worktrees"

    def roots(self) -> tuple[Path, ...]:
        base = (self.data_dir.resolve(), Path.cwd().resolve())
        extra = tuple(path.resolve() for path in self.allowed_roots)
        seen: list[Path] = []
        for path in (*base, *extra):
            if path not in seen:
                seen.append(path)
        return tuple(seen)


def load_settings(path: Path | None = None) -> Settings:
    candidate = path
    if candidate is None and os.environ.get("DUO_CONFIG"):
        candidate = Path(os.environ["DUO_CONFIG"])
    if candidate is None:
        candidate = Path("duo.toml")
    raw: dict = {}
    if candidate and Path(candidate).exists():
        raw = tomllib.loads(Path(candidate).read_text(encoding="utf-8"))
    host = str(raw.get("host", "127.0.0.1")).strip()
    if host not in LOOPBACK_HOSTS:
        raise ValueError(
            "L'écoute réseau est limitée à la machine locale "
            "(127.0.0.1, localhost ou ::1)."
        )
    port = int(raw.get("port", 8765))
    if not 1 <= port <= 65535:
        raise ValueError("Le port doit être compris entre 1 et 65535.")
    data_dir = Path(str(raw.get("data_dir", "data"))).expanduser()
    if not data_dir.is_absolute():
        data_dir = (Path.cwd() / data_dir).resolve()
    roots_raw = raw.get("allowed_roots", [])
    if not isinstance(roots_raw, list):
        raise ValueError("allowed_roots doit être une liste de chemins.")
    roots = tuple(Path(str(item)).expanduser() for item in roots_raw)
    settings = Settings(
        host=host,
        port=port,
        data_dir=data_dir,
        max_turns=int(raw.get("max_turns", 8)),
        max_duration_seconds=int(raw.get("max_duration_seconds", 600)),
        agent_timeout_seconds=int(raw.get("agent_timeout_seconds", 180)),
        similarity_threshold=float(raw.get("similarity_threshold", 0.92)),
        max_corrections=int(raw.get("max_corrections", 1)),
        allowed_roots=roots,
        cursor_model=str(raw.get("cursor_model", "auto")),
        copilot_model=str(raw.get("copilot_model", "")),
    )
    settings.data_dir.mkdir(parents=True, exist_ok=True)
    settings.sandbox_root.mkdir(parents=True, exist_ok=True)
    settings.worktree_root.mkdir(parents=True, exist_ok=True)
    return settings
