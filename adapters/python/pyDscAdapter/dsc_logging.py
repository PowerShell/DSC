import json
import logging
import sys
from contextlib import contextmanager
import contextvars

# ---------------------------------
# Logger implementation
# ---------------------------------

# Context visible to all logs emitted during an operation
cv_operation = contextvars.ContextVar("dsc_operation", default="")
cv_resource_type = contextvars.ContextVar("dsc_resource_type", default="")

# --------------------------------------------------------------
# DSC JSON Formatter - converts log records to DSC JSON format
# ---------------------------------------------------------------
class DSCJsonFormatter(logging.Formatter):
    """Formats log records as DSC-compliant JSON."""
    
    def format(self, record):
        payload = {
            "message": record.getMessage(),
            "target": record.name,
            "level": record.levelname.lower(),
        }
        
        # Add context if available
        if hasattr(record, "operation") and record.operation:
            payload["operation"] = record.operation
        if hasattr(record, "resourceType") and record.resourceType:
            payload["resourceType"] = record.resourceType
        
        if record.exc_info:
            payload["exception"] = self.formatException(record.exc_info)
        
        return json.dumps(payload, ensure_ascii=False)


# -------------------------------------------------------
# Context Filter - injects contextvars into log records
# -------------------------------------------------------
class DSCContextFilter(logging.Filter):
    """Injects DSC context variables into every log record."""
    
    def filter(self, record):
        # Inject context into record
        record.operation = cv_operation.get("")
        record.resourceType = cv_resource_type.get("")
        return True  # pass record through


# -------------------------------------------------------------------------------------
# LoggingSetup - configure dedicated dsc_adapter logger with DSCJsonFormatter/filter
# -------------------------------------------------------------------------------------
def setup_dsc_logging(level="info"):
    """
    Configure DSC logging in one call.
    
    Args:
        level: DSC trace level (trace/debug/info/warning/error/critical)
    
    Returns:
        Logger instance ready to use
    """
    # Map DSC levels to Python levels
    level_map = {
        "trace": logging.DEBUG,
        "debug": logging.DEBUG,
        "info": logging.INFO,
        "warning": logging.WARNING,
        "error": logging.ERROR,
        "critical": logging.CRITICAL,
    }
    
    logger = logging.getLogger("dsc_adapter")
    logger.setLevel(level_map.get(level.lower(), logging.INFO))
    logger.propagate = False

    # Reset only this logger's handlers to avoid duplicate emissions across repeated setup calls.
    logger.handlers.clear()
    
    # Add handler with JSON formatter and context filter
    handler = logging.StreamHandler(sys.stderr)
    handler.setFormatter(DSCJsonFormatter())
    handler.addFilter(DSCContextFilter())
    logger.addHandler(handler)

    return logger


# -------------------------------------------------------
# Context managers
# -------------------------------------------------------
@contextmanager
def operation_context(operation, resource_type=""):
    """Set operation and resource type for all logs in scope."""
    tokens = [cv_operation.set(operation)]
    if resource_type:
        tokens.append(cv_resource_type.set(resource_type))
    
    try:
        yield
    finally:
        for token in reversed(tokens):
            token.var.reset(token)
