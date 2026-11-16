from __future__ import annotations

from pathlib import Path

import pytest

from duo.config import load_settings
from duo.gitspace import WorkspaceManager, init_repository, run_git, write_declared_files
from duo.security import ensure_within, git_args_allowed, mutation_allowed, safe_relative_file

from tests.conftest import make_stack, run, session_fields


def test_command_policy_blocks_destructive_git():
    assert git_args_allowed(["status", "--porcelain=v1"])
    assert not git_args_allowed(["reset", "--hard"])
    assert not git_args_allowed(["clean", "-fd"])
    assert not git_args_allowed(["push", "--force"])
    assert not git_args_allowed(["commit", "--hard"])


def test_path_escape(tmp_path: Path):
    root = tmp_path / "sandbox"
    root.mkdir()
    with pytest.raises(ValueError):
        write_declared_files(root, [{"path": "../outside.py", "content": "print(1)\n"}])
    with pytest.raises(ValueError):
        write_declared_files(root, [{"path": "C:/Windows/evil.py", "content": "x"}])
    assert not (tmp_path / "outside.py").exists()
    with pytest.raises(PermissionError):
        ensure_within(tmp_path.parent / "nope", (root,))
    safe = safe_relative_file(root, "pkg/mod.py")
    assert safe.parent == root / "pkg"


def test_origin_and_header():
    assert mutation_allowed("POST", None, "1", "127.0.0.1", 8765)
    assert not mutation_allowed("POST", None, None, "127.0.0.1", 8765)
    assert not mutation_allowed("POST", "https://evil.example", "1", "127.0.0.1", 8765)
    assert not mutation_allowed("GET", "https://evil.example", None, "127.0.0.1", 8765)
    assert mutation_allowed("POST", "http://127.0.0.1:8765", "1", "127.0.0.1", 8765)
    assert not mutation_allowed("POST", "http://127.0.0.1:9", "1", "127.0.0.1", 8765)


def test_bind_is_loopback(tmp_path: Path):
    config = tmp_path / "duo.toml"
    config.write_text('host = "0.0.0.0"\n', encoding="utf-8")
    with pytest.raises(ValueError):
        load_settings(config)


def test_git_rejects_reset_and_keeps_user_file(tmp_path: Path):
    repo = tmp_path / "repo"
    init_repository(repo)
    (repo / "user.txt").write_text("ne pas effacer\n", encoding="utf-8")
    run_git(repo, ["add", "--", "user.txt"])
    run_git(repo, ["commit", "-m", "travail utilisateur"])
    with pytest.raises(PermissionError):
        run_git(repo, ["reset", "--hard"])
    assert (repo / "user.txt").read_text(encoding="utf-8") == "ne pas effacer\n"


def test_external_repo_requires_confirmation(tmp_path: Path):
    _, db, orch = make_stack(tmp_path)
    repo = tmp_path / "repo"
    init_repository(repo)
    (repo / "user.txt").write_text("conservé\n", encoding="utf-8")
    run_git(repo, ["add", "--", "user.txt"])
    run_git(repo, ["commit", "-m", "utilisateur"])
    session = orch.create_session(
        session_fields(external_repo=str(repo), confirm_external=False, allow_implementation=False)
    )
    result = run(orch.run_session(session["id"]))
    assert result["status"] == "failed"
    assert (repo / "user.txt").read_text(encoding="utf-8") == "conservé\n"
    db.close()


def test_confirmed_worktree_does_not_touch_user_file(tmp_path: Path):
    _, db, orch = make_stack(tmp_path)
    repo = tmp_path / "repo"
    init_repository(repo)
    (repo / "user.txt").write_text("conservé\n", encoding="utf-8")
    run_git(repo, ["add", "--", "user.txt"])
    run_git(repo, ["commit", "-m", "utilisateur"])
    session = orch.create_session(
        session_fields(
            external_repo=str(repo),
            confirm_external=True,
            allow_implementation=False,
            max_turns=4,
        )
    )
    result = run(orch.run_session(session["id"]))
    assert result["status"] in {"succeeded", "needs_arbitration", "failed"}
    assert (repo / "user.txt").read_text(encoding="utf-8") == "conservé\n"
    worktree = Path(result["workspace_path"])
    assert worktree.exists()
    assert worktree.resolve() != repo.resolve()
    db.close()
