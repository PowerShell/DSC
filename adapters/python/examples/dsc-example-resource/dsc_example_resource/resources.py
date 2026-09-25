# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Example DSC resources: Greeting, Counter, EnvVar.

These three resources demonstrate progressively more complex DSC patterns:

  Example/Greeting  — read-only, get() only
  Example/Counter   — read/write with STATE_AND_DIFF, JSON file persistence
  Example/EnvVar    — presence pattern (_exist), get/set/test/delete/export
"""
from __future__ import annotations

import json
import logging
import os
from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path

from ms_dsc import DscResource, SetResult, TestResult, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.protocols import Deletable, Exportable, Gettable, Settable, Testable
from ms_dsc.schema import DataclassSchemaProvider

_logger = logging.getLogger(__name__)


# ===========================================================================
# Example/Greeting — read-only resource
# ===========================================================================

@dataclass
class GreetingSchema:
    """Schema for Example/Greeting."""

    name: str = field(
        metadata={"description": "Name to greet.", "title": "Name"}
    )
    message: str = field(
        default="",
        metadata={"description": "Generated greeting message (read-only).", "title": "Message"},
    )


@dsc_resource(
    type="Example/Greeting",
    version="1.0.0",
    description="Returns a greeting message for the given name. Read-only.",
    tags=["example", "read-only"],
)
class GreetingResource(DscResource[GreetingSchema], Gettable):
    """
    Read-only resource — implements only get().

    DSC calls get() to retrieve the current state.  Because this resource has
    no persistent state, get() simply computes the greeting and returns it.

    Illustrates: minimal resource, get() only, no side effects.
    """

    schema_provider = DataclassSchemaProvider(GreetingSchema)

    def get(self, instance: GreetingSchema) -> GreetingSchema:
        message = f"Hello, {instance.name}!"
        _logger.info("get(%s) → %s", instance.name, message)
        return GreetingSchema(name=instance.name, message=message)


# ===========================================================================
# Example/Counter — read/write resource with JSON file persistence
# ===========================================================================

_COUNTER_FILE = Path.home() / ".dsc" / "example_counters.json"


def _load_counters() -> dict[str, int]:
    try:
        return json.loads(_COUNTER_FILE.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError):
        return {}


def _save_counters(data: dict[str, int]) -> None:
    _COUNTER_FILE.parent.mkdir(parents=True, exist_ok=True)
    _COUNTER_FILE.write_text(json.dumps(data, indent=2), encoding="utf-8")


@dataclass
class CounterSchema:
    """Schema for Example/Counter."""

    id: str = field(
        metadata={"description": "Unique identifier for the counter.", "title": "ID"}
    )
    value: int = field(
        default=0,
        metadata={"description": "Current counter value.", "title": "Value"},
    )


@dsc_resource(
    type="Example/Counter",
    version="1.0.0",
    description="Manages a named integer counter persisted in ~/.dsc/example_counters.json.",
    tags=["example", "stateful"],
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class CounterResource(DscResource[CounterSchema], Gettable, Settable, Testable, Deletable, Exportable):
    """
    Read/write resource with STATE_AND_DIFF.

    Counter values are stored in ~/.dsc/example_counters.json across
    DSC invocations.

    Illustrates:
      - set() returning changed_properties for stateAndDiff mode
      - test() returning differing_properties
      - Persistent state via a JSON file
    """

    schema_provider = DataclassSchemaProvider(CounterSchema)

    def get(self, instance: CounterSchema) -> CounterSchema:
        value = _load_counters().get(instance.id, 0)
        _logger.info("get(counter=%s) -> value=%d", instance.id, value)
        return CounterSchema(id=instance.id, value=value)

    def set(self, instance: CounterSchema) -> SetResult[CounterSchema]:
        counters = _load_counters()
        before = counters.get(instance.id, 0)
        counters[instance.id] = instance.value
        _save_counters(counters)
        changed = ["value"] if before != instance.value else []
        _logger.info(
            "set(counter=%s, value=%d) changed=%s", instance.id, instance.value, changed
        )
        return SetResult(
            actual_state=CounterSchema(id=instance.id, value=instance.value),
            changed_properties=changed,
        )

    def test(self, instance: CounterSchema) -> TestResult[CounterSchema]:
        actual = self.get(instance)
        diffs = ["value"] if actual.value != instance.value else []
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: CounterSchema) -> None:
        counters = _load_counters()
        if instance.id in counters:
            del counters[instance.id]
            _save_counters(counters)
            _logger.info("delete(counter=%s)", instance.id)

    def export(self, instance: CounterSchema | None) -> Iterator[CounterSchema]:
        for cid, value in _load_counters().items():
            yield CounterSchema(id=cid, value=value)


# ===========================================================================
# Example/EnvVar — presence pattern (_exist), manages environment variables
# ===========================================================================

@dataclass
class EnvVarSchema:
    """Schema for Example/EnvVar."""

    name: str = field(
        metadata={"description": "Name of the environment variable.", "title": "Name"}
    )
    value: str = field(
        default="",
        metadata={"description": "Value of the environment variable.", "title": "Value"},
    )
    _exist: bool = field(
        default=True,
        metadata={"description": "Whether the environment variable should exist.", "title": "Exist"},
    )


@dsc_resource(
    type="Example/EnvVar",
    version="1.0.0",
    description=(
        "Manages a process-scope environment variable. "
        "Demonstrates the _exist presence pattern used across DSC resources."
    ),
    tags=["example", "environment", "cross-platform"],
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class EnvVarResource(DscResource[EnvVarSchema], Gettable, Settable, Testable, Deletable, Exportable):
    """
    Manages environment variables in the current process scope.

    Note: Environment variable changes made by this resource are scoped to the
    DSC adapter process and its children.  They do NOT persist to the user's
    shell session or system environment.  This resource is primarily educational.

    Illustrates:
      - _exist presence pattern (create/remove semantics)
      - get/set/test/delete/export — all five capabilities
      - Checking both presence and value in test()
    """

    schema_provider = DataclassSchemaProvider(EnvVarSchema)

    def get(self, instance: EnvVarSchema) -> EnvVarSchema:
        raw = os.environ.get(instance.name)
        exists = raw is not None
        _logger.info("get(EnvVar=%s) -> _exist=%s, value=%r", instance.name, exists, raw)
        return EnvVarSchema(name=instance.name, value=raw or "", _exist=exists)

    def set(self, instance: EnvVarSchema) -> SetResult[EnvVarSchema]:
        actual_before = self.get(instance)
        changed: list[str] = []

        if instance._exist:
            os.environ[instance.name] = instance.value
            if not actual_before._exist:
                changed.append("_exist")
            if actual_before.value != instance.value:
                changed.append("value")
        else:
            if actual_before._exist:
                del os.environ[instance.name]
                changed.append("_exist")

        actual_after = self.get(instance)
        _logger.info("set(EnvVar=%s) changed=%s", instance.name, changed)
        return SetResult(actual_state=actual_after, changed_properties=changed)

    def test(self, instance: EnvVarSchema) -> TestResult[EnvVarSchema]:
        actual = self.get(instance)
        diffs: list[str] = []
        if actual._exist != instance._exist:
            diffs.append("_exist")
        elif actual._exist and instance._exist and actual.value != instance.value:
            diffs.append("value")
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: EnvVarSchema) -> None:
        if instance.name in os.environ:
            del os.environ[instance.name]
            _logger.info("delete(EnvVar=%s)", instance.name)

    def export(self, instance: EnvVarSchema | None) -> Iterator[EnvVarSchema]:
        prefix = instance.name if instance is not None else ""
        for key, value in os.environ.items():
            if not prefix or key.startswith(prefix):
                yield EnvVarSchema(name=key, value=value, _exist=True)

