from fastapi import FastAPI

from .api import router
from .logging_config import LOGGING_CONFIG

__version__ = "0.1.0"
__author__ = "Renju Ashokan"
__description__ = "Network File Browser with FastAPI"

__all__ = ["LOGGING_CONFIG", "app"]

app = FastAPI(
    title="Network File Browser",
    description="API for browsing and managing network files",
    version=__version__,
    root_path="/api/v1",
)

app.include_router(router)
