"""Lancement détaché des CLI OAuth (navigateur côté utilisateur)."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def spawn_detached(command: list[str], *, cwd: Path | None = None) -> None:
    """Démarre un process détaché pour un login interactif (navigateur)."""
    if not command or any(not isinstance(part, str) or not part for part in command):
        raise ValueError("Commande de login invalide.")
    kwargs: dict = {
        "stdin": subprocess.DEVNULL,
        "stdout": subprocess.DEVNULL,
        "stderr": subprocess.DEVNULL,
        "close_fds": True,
        "env": os.environ.copy(),
    }
    if cwd is not None:
        kwargs["cwd"] = str(cwd)
    if sys.platform == "win32":
        flags = subprocess.CREATE_NEW_PROCESS_GROUP | getattr(subprocess, "DETACHED_PROCESS", 0)
        flags |= getattr(subprocess, "CREATE_NO_WINDOW", 0)
        kwargs["creationflags"] = flags
    else:
        kwargs["start_new_session"] = True
    subprocess.Popen(command, **kwargs)
