from __future__ import annotations

import asyncio
import sys

from duo.models import CancelToken
from duo.process_util import ProcessRegistry, run_command, windows_flags


def test_process_stop_kills_child():
    async def scenario():
        registry = ProcessRegistry()
        token = CancelToken()

        async def launch():
            return await run_command(
                [sys.executable, "-c", "import time; time.sleep(30)"],
                cwd=".",
                timeout=20,
                token=token,
                registry=registry,
                session_id="s",
            )

        task = asyncio.create_task(launch())
        await asyncio.sleep(0.3)
        token.trigger("stop")
        code, _out, _err, reason = await task
        assert reason == "cancelled"
        assert code is not None

    asyncio.run(scenario())
    assert windows_flags() != 0
