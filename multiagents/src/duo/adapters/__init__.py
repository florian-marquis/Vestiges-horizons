"""Adaptateurs d'agents."""

from duo.adapters.base import AgentAdapter
from duo.adapters.copilot_cli import CopilotCliAdapter
from duo.adapters.cursor_cli import CursorCliAdapter
from duo.adapters.cursor_sdk import CursorSdkAdapter
from duo.adapters.simulated import SimulatedAdapter

__all__ = [
    "AgentAdapter",
    "CopilotCliAdapter",
    "CursorCliAdapter",
    "CursorSdkAdapter",
    "SimulatedAdapter",
]
