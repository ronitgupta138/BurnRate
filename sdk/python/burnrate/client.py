"""
burnrate Python Client SDK: Hard Spend Caps & Cache Guard.
"""

import urllib.request
import json
from typing import Dict, Any

class BurnrateClient:
    """Client for checking live burnrate budget status and health."""

    def __init__(self, proxy_url: str = "http://localhost:7070"):
        self.proxy_url = proxy_url.rstrip("/")

    def get_status(self) -> Dict[str, Any]:
        """Fetch live session spend and budget metrics."""
        url = f"{self.proxy_url}/status"
        with urllib.request.urlopen(url) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def is_tripped(self) -> bool:
        """Check if session has reached hard spend limit."""
        status = self.get_status()
        return bool(status.get("circuit_breaker_tripped", False))
