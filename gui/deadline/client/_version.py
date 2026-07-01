# Version info — delegates to the Rust native extension.
# This module exists for compatibility with code that imports
# deadline.client._version directly (e.g. DCC submitter addons).
from __future__ import annotations

from deadline._native import __version__

__all__ = ["__version__", "version"]

version: str = __version__
