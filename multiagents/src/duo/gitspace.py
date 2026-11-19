"""Bac à sable Git et écriture contrôlée des fichiers."""

from __future__ import annotations

import subprocess
from pathlib import Path

from duo.security import ensure_within, git_args_allowed, safe_relative_file

GIT_IDENTITY = ["-c", "user.name=Duo", "-c", "user.email=duo@localhost"]


class GitError(RuntimeError):
    pass


def run_git(repo: Path, args: list[str], check: bool = False) -> subprocess.CompletedProcess[str]:
    if not git_args_allowed(args):
        raise PermissionError(f"Commande Git refusée : {' '.join(args)}")
    command = ["git", "-C", str(repo), *GIT_IDENTITY, *args]
    completed = subprocess.run(
        command,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if check and completed.returncode != 0:
        raise GitError(completed.stderr.strip() or completed.stdout.strip() or "git a échoué")
    return completed


def init_repository(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    completed = subprocess.run(
        ["git", "init", str(path)],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if completed.returncode != 0:
        raise GitError(completed.stderr.strip() or "git init a échoué")
    commit = run_git(path, ["commit", "--allow-empty", "-m", "duo: état initial du bac à sable"])
    if commit.returncode != 0:
        raise GitError(commit.stderr.strip() or "le commit initial a échoué")


class WorkspaceManager:
    def __init__(self, sandbox_root: Path, worktree_root: Path, allowed_roots: tuple[Path, ...]):
        self.sandbox_root = sandbox_root
        self.worktree_root = worktree_root
        self.allowed_roots = allowed_roots
        self.sandbox_root.mkdir(parents=True, exist_ok=True)
        self.worktree_root.mkdir(parents=True, exist_ok=True)

    def create(self, session_id: str, external_repo: str | None, confirm_external: bool) -> Path:
        if external_repo:
            return self._create_worktree(session_id, external_repo, confirm_external)
        destination = ensure_within(self.sandbox_root / session_id, (self.sandbox_root,))
        if destination.exists() and (destination / ".git").exists():
            return destination
        init_repository(destination)
        return destination

    def _create_worktree(self, session_id: str, external_repo: str, confirm_external: bool) -> Path:
        if not confirm_external:
            raise PermissionError(
                "La création d'un worktree sur un dépôt existant exige une confirmation explicite."
            )
        repo = ensure_within(Path(external_repo), self.allowed_roots)
        probe = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--is-inside-work-tree"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        if probe.returncode != 0 or probe.stdout.strip() != "true":
            raise GitError("Le chemin indiqué n'est pas un dépôt Git.")
        destination = ensure_within(self.worktree_root / session_id, (self.worktree_root,))
        destination.parent.mkdir(parents=True, exist_ok=True)
        branch = f"duo/{session_id[:8]}"
        added = subprocess.run(
            ["git", "-C", str(repo), "worktree", "add", "-b", branch, str(destination), "HEAD"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        if added.returncode != 0:
            raise GitError(added.stderr.strip() or "création du worktree refusée")
        return destination

    def status(self, repo: Path) -> str:
        completed = run_git(repo, ["status", "--porcelain=v1", "--branch"])
        return completed.stdout

    def diff(self, repo: Path) -> str:
        parent = run_git(repo, ["rev-parse", "--verify", "HEAD~1"])
        if parent.returncode == 0:
            completed = run_git(repo, ["diff", "HEAD~1"])
            if completed.stdout.strip():
                return completed.stdout
        completed = run_git(repo, ["diff"])
        return completed.stdout

    def changed_files(self, repo: Path) -> list[str]:
        completed = run_git(repo, ["log", "--name-only", "--pretty=format:"])
        seen: list[str] = []
        for line in completed.stdout.splitlines():
            name = line.strip()
            if name and name not in seen:
                seen.append(name)
        return seen

    def commit_all(self, repo: Path, message: str) -> str:
        if any(char in message for char in "\n\r"):
            raise ValueError("Le message de commit doit tenir sur une ligne.")
        run_git(repo, ["add", "--", "."], check=False)
        completed = run_git(repo, ["commit", "-m", message])
        if completed.returncode != 0 and "nothing to commit" not in (completed.stdout + completed.stderr):
            raise GitError(completed.stderr.strip() or completed.stdout.strip())
        return (completed.stdout + completed.stderr).strip()


def write_declared_files(root: Path, files: list) -> list[str]:
    written: list[str] = []
    if not isinstance(files, list):
        return written
    for item in files:
        if not isinstance(item, dict):
            continue
        relative = item.get("path")
        content = item.get("content")
        if not isinstance(relative, str) or not isinstance(content, str):
            continue
        destination = safe_relative_file(root, relative)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content, encoding="utf-8")
        written.append(relative.replace("\\", "/"))
    return written
