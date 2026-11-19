"""Attente salon avec retry opaque : ne revient que sur message ou clôture."""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from urllib.parse import quote

DEFAULT_BASE = "http://127.0.0.1:8765"
POLL_TIMEOUT = 90


def wait_until_message(
    base: str,
    room_id: str,
    token: str,
    after: int,
    poll_timeout: int = POLL_TIMEOUT,
) -> dict:
    """Enchaîne les long-polls jusqu'à des messages visibles ou un salon clos."""
    base = base.rstrip("/")
    poll = max(1, min(int(poll_timeout), POLL_TIMEOUT))
    quoted = quote(token, safe="")
    while True:
        payload = http_get(
            f"{base}/api/rooms/{room_id}/wait?token={quoted}&after={after}&timeout={poll}",
            timeout=poll + 15,
        )
        if payload.get("messages") or payload.get("closed"):
            result = dict(payload)
            result["timed_out"] = False
            return result
        if not payload.get("timed_out"):
            result = dict(payload)
            result["timed_out"] = False
            return result


def http_get(url: str, timeout: int) -> dict:
    request = urllib.request.Request(url, method="GET")
    return _read_response(request, timeout)


def _http_json(url: str, payload: dict, timeout: int, method: str) -> dict:
    body = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=body,
        headers={
            "Content-Type": "application/json",
            "X-Duo-Request": "1",
        },
        method=method,
    )
    return _read_response(request, timeout)


def http_post(url: str, payload: dict, timeout: int) -> dict:
    return _http_json(url, payload, timeout, "POST")


def http_patch(url: str, payload: dict, timeout: int) -> dict:
    return _http_json(url, payload, timeout, "PATCH")


def _read_response(request: urllib.request.Request, timeout: int) -> dict:
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(detail or f"Le salon a répondu {exc.code}.") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(
            "Le salon Duo ne répond pas. Lancez start.bat, puis réessayez."
        ) from exc


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Attend un message du salon Duo (retry jusqu'à message ou clôture)."
    )
    parser.add_argument("room_id")
    parser.add_argument("token")
    parser.add_argument("after", type=int, help="Dernier seq déjà traité.")
    parser.add_argument(
        "--base",
        default=os.environ.get("DUO_BUS_URL", DEFAULT_BASE),
        help="URL du serveur Duo (défaut DUO_BUS_URL ou http://127.0.0.1:8765).",
    )
    parser.add_argument(
        "--poll",
        type=int,
        default=POLL_TIMEOUT,
        help=f"Durée d'un poll HTTP, max {POLL_TIMEOUT}s.",
    )
    args = parser.parse_args(argv)
    try:
        payload = wait_until_message(
            args.base,
            args.room_id,
            args.token,
            args.after,
            poll_timeout=args.poll,
        )
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
