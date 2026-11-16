"""Contrôles de sécurité : secrets, chemins, commandes, origine."""

from __future__ import annotations

import re
from pathlib import Path
from urllib.parse import urlparse

SECRET_PATTERNS = (
    re.compile(r"github_pat_[A-Za-z0-9_]+", re.IGNORECASE),
    re.compile(r"gh[pousr]_[A-Za-z0-9]+"),
    re.compile(r"cursor_[A-Za-z0-9]{8,}"),
    re.compile(r"sk-[A-Za-z0-9_\-]{8,}"),
    re.compile(r"(?i)bearer\s+[A-Za-z0-9._\-]{8,}"),
    re.compile(
        r"(?i)\b(api[_-]?key|token|password|secret|authorization)\b\s*[:=]\s*\S+"
    ),
)

DENIED_GIT_SUBCOMMANDS = frozenset(
    {
        "reset",
        "clean",
        "push",
        "rebase",
        "filter-branch",
        "gc",
        "prune",
        "submodule",
        "config",
        "update-ref",
        "reflog",
    }
)
DENIED_GIT_FLAGS = frozenset({"--hard", "--force", "-f", "--mirror", "--prune"})
ALLOWED_GIT_SUBCOMMANDS = frozenset(
    {
        "status",
        "diff",
        "log",
        "add",
        "commit",
        "rev-parse",
        "show",
        "init",
        "worktree",
        "branch",
    }
)


def redact(text: str) -> str:
    cleaned = text or ""
    for pattern in SECRET_PATTERNS:
        cleaned = pattern.sub("[secret masqué]", cleaned)
    return cleaned


def ensure_within(path: Path, roots: tuple[Path, ...] | list[Path]) -> Path:
    resolved = path.expanduser().resolve()
    for root in roots:
        base = root.expanduser().resolve()
        try:
            resolved.relative_to(base)
            return resolved
        except ValueError:
            continue
    raise PermissionError(f"Chemin hors des racines autorisées : {resolved}")


def safe_relative_file(root: Path, relative: str) -> Path:
    if not isinstance(relative, str) or not relative.strip():
        raise ValueError("Chemin de fichier vide.")
    candidate = Path(relative)
    if candidate.is_absolute() or candidate.drive or candidate.root:
        raise ValueError("Les chemins absolus sont refusés.")
    parts = candidate.parts
    if any(part in {"..", "", ".git"} or part.endswith(":") for part in parts):
        raise ValueError("Le chemin tente de sortir du bac à sable.")
    if ".." in relative.replace("\\", "/").split("/"):
        raise ValueError("Le chemin tente de sortir du bac à sable.")
    root_resolved = root.resolve()
    destination = (root_resolved / candidate).resolve()
    try:
        destination.relative_to(root_resolved)
    except ValueError as exc:
        raise ValueError("Le chemin tente de sortir du bac à sable.") from exc
    return destination


def git_args_allowed(args: list[str]) -> bool:
    if not args or not all(isinstance(item, str) for item in args):
        return False
    if args[0] in DENIED_GIT_SUBCOMMANDS:
        return False
    if args[0] not in ALLOWED_GIT_SUBCOMMANDS:
        return False
    if any(flag in DENIED_GIT_FLAGS for flag in args[1:]):
        return False
    if args[0] == "worktree":
        return len(args) >= 2 and args[1] == "add"
    if args[0] == "branch":
        return len(args) >= 2 and args[1] not in {"-D", "-d"}
    return True


def origin_allowed(origin: str | None, host: str, port: int) -> bool:
    if origin is None or origin == "" or origin == "null":
        return True
    parsed = urlparse(origin)
    if parsed.scheme not in {"http", "https"}:
        return False
    hostname = (parsed.hostname or "").lower()
    if hostname not in {"127.0.0.1", "localhost", "::1"}:
        return False
    if host in {"127.0.0.1", "localhost", "::1"} and hostname not in {
        "127.0.0.1",
        "localhost",
        "::1",
    }:
        return False
    expected_ports = {port}
    if parsed.port is None:
        inferred = 80 if parsed.scheme == "http" else 443
        return inferred == port
    return parsed.port in expected_ports


def mutation_allowed(
    method: str,
    origin: str | None,
    duo_header: str | None,
    host: str,
    port: int,
) -> bool:
    if method.upper() in {"GET", "HEAD", "OPTIONS"}:
        return origin_allowed(origin, host, port)
    if duo_header != "1":
        return False
    return origin_allowed(origin, host, port)
