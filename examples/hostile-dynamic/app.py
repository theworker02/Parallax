"""Hostile dynamic Python — getattr, decorators, asyncio signals for Horizon demos."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from functools import wraps
from typing import Any, Callable


def audit(name: str) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    """Decorator that wraps call sites (metaprogram signal)."""

    def decorator(fn: Callable[..., Any]) -> Callable[..., Any]:
        @wraps(fn)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            return fn(*args, **kwargs)

        wrapper.__audit_name__ = name  # type: ignore[attr-defined]
        return wrapper

    return decorator


@dataclass
class UserRecord:
    email: str
    permissions: list[str]


class DynamicFacade:
    """Dynamic attribute access — Horizon should flag getattr/setattr barriers."""

    def __init__(self) -> None:
        self._bag: dict[str, Any] = {}

    def __getattr__(self, name: str) -> Any:
        return getattr(self._bag, name, None) if hasattr(self._bag, name) else self._bag.get(name)

    def __setattr__(self, name: str, value: Any) -> None:
        if name == "_bag":
            super().__setattr__(name, value)
        else:
            self._bag[name] = value


@audit("normalize")
def normalize_user(raw: dict[str, Any]) -> UserRecord:
    """Uses dynamic dict access patterns."""
    email = raw.get("email") or getattr(raw, "email", "")
    perms = raw.get("permissions") or []
    return UserRecord(email=str(email).strip().lower(), permissions=list(perms))


async def fetch_profile(client: Any, user_id: str) -> dict[str, Any]:
    """Async I/O stub — concurrency signal for CIR."""
    await asyncio.sleep(0)
    return {"id": user_id, "email": f"{user_id}@example.com", "permissions": ["read"]}


async def load_user(client: Any, user_id: str) -> UserRecord:
    payload = await fetch_profile(client, user_id)
    return normalize_user(payload)


def dynamic_lookup(obj: Any, field: str) -> Any:
    """Explicit getattr barrier."""
    return getattr(obj, field, None)


if __name__ == "__main__":
    facade = DynamicFacade()
    facade.role = "admin"
    print(dynamic_lookup(facade, "role"))
    print(normalize_user({"email": "  TEST@Example.COM ", "permissions": ["write"]}))
