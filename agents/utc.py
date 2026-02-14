from datetime import datetime, timezone

def utcnow() -> datetime:
    """Return the current time in UTC with timezone information."""
    return datetime.now(timezone.utc)
