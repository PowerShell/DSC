# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for ms_dsc.logging — DscLogHandler and configure_logging."""
from __future__ import annotations

import json
import logging
import os
from io import StringIO
from unittest.mock import patch

import pytest

from ms_dsc.logging import (
    DscLogHandler,
    _DSC_TO_PYTHON,
    _PYTHON_TO_DSC,
    configure_logging,
)


def _make_logger(name: str = "test") -> tuple[logging.Logger, StringIO]:
    """Return a logger wired to a DscLogHandler writing to a StringIO buffer."""
    buf = StringIO()
    handler = DscLogHandler()
    handler.setFormatter(logging.Formatter("%(name)s: %(message)s"))

    logger = logging.getLogger(name)
    logger.handlers.clear()
    logger.addHandler(handler)
    logger.propagate = False
    logger.setLevel(logging.DEBUG)

    with patch("sys.stderr", buf):
        yield logger, buf


class TestDscLogHandler:
    def _capture(self, level_fn_name: str, message: str) -> dict:
        buf = StringIO()
        handler = DscLogHandler()
        handler.setFormatter(logging.Formatter("%(name)s: %(message)s"))
        logger = logging.getLogger(f"test.{level_fn_name}")
        logger.handlers.clear()
        logger.addHandler(handler)
        logger.propagate = False
        logger.setLevel(logging.DEBUG)
        with patch("sys.stderr", buf):
            getattr(logger, level_fn_name)(message)
        return json.loads(buf.getvalue().strip())

    def test_info_emits_info_key(self):
        data = self._capture("info", "hello world")
        assert "info" in data
        assert "hello world" in data["info"]

    def test_error_emits_error_key(self):
        data = self._capture("error", "something broke")
        assert "error" in data

    def test_warning_emits_warn_key(self):
        data = self._capture("warning", "heads up")
        assert "warn" in data

    def test_debug_emits_debug_key(self):
        data = self._capture("debug", "verbose detail")
        assert "debug" in data

    def test_output_is_single_json_line(self):
        buf = StringIO()
        handler = DscLogHandler()
        logger = logging.getLogger("test.single")
        logger.handlers.clear()
        logger.addHandler(handler)
        logger.propagate = False
        logger.setLevel(logging.DEBUG)
        with patch("sys.stderr", buf):
            logger.info("one line")
        lines = [l for l in buf.getvalue().splitlines() if l.strip()]
        assert len(lines) == 1

    def test_logger_name_included_in_message(self):
        buf = StringIO()
        handler = DscLogHandler()
        handler.setFormatter(logging.Formatter("%(name)s: %(message)s"))
        logger = logging.getLogger("my.module")
        logger.handlers.clear()
        logger.addHandler(handler)
        logger.propagate = False
        logger.setLevel(logging.DEBUG)
        with patch("sys.stderr", buf):
            logger.info("test")
        data = json.loads(buf.getvalue().strip())
        assert "my.module" in data["info"]


class TestConfigureLogging:
    def teardown_method(self):
        # Reset root logger after each test.
        root = logging.getLogger()
        root.handlers.clear()

    def test_default_level_is_info(self):
        env = {k: v for k, v in os.environ.items() if k != "DSC_TRACE_LEVEL"}
        with patch.dict(os.environ, env, clear=True):
            configure_logging()
        assert logging.getLogger().level == logging.INFO

    def test_debug_level(self):
        with patch.dict(os.environ, {"DSC_TRACE_LEVEL": "debug"}):
            configure_logging()
        assert logging.getLogger().level == logging.DEBUG

    def test_trace_maps_to_debug(self):
        with patch.dict(os.environ, {"DSC_TRACE_LEVEL": "trace"}):
            configure_logging()
        assert logging.getLogger().level == logging.DEBUG

    def test_warning_level(self):
        with patch.dict(os.environ, {"DSC_TRACE_LEVEL": "warning"}):
            configure_logging()
        assert logging.getLogger().level == logging.WARNING

    def test_warn_alias(self):
        with patch.dict(os.environ, {"DSC_TRACE_LEVEL": "warn"}):
            configure_logging()
        assert logging.getLogger().level == logging.WARNING

    def test_error_level(self):
        with patch.dict(os.environ, {"DSC_TRACE_LEVEL": "error"}):
            configure_logging()
        assert logging.getLogger().level == logging.ERROR

    def test_unknown_level_defaults_to_info(self):
        with patch.dict(os.environ, {"DSC_TRACE_LEVEL": "bogus"}):
            configure_logging()
        assert logging.getLogger().level == logging.INFO

    def test_installs_dsc_handler(self):
        configure_logging()
        root = logging.getLogger()
        assert any(isinstance(h, DscLogHandler) for h in root.handlers)


class TestLevelMaps:
    def test_dsc_to_python_completeness(self):
        for key in ("trace", "debug", "info", "warning", "warn", "error", "critical"):
            assert key in _DSC_TO_PYTHON

    def test_python_to_dsc_completeness(self):
        for level in (logging.DEBUG, logging.INFO, logging.WARNING, logging.ERROR, logging.CRITICAL):
            assert level in _PYTHON_TO_DSC
