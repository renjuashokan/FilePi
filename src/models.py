"""Pydantic models for FilePi."""

from typing import List, Optional

from pydantic import BaseModel


class FileInfo(BaseModel):
    """Information about a file or directory."""

    name: str
    size: int
    is_directory: bool
    created_time: int
    modified_time: int
    file_type: Optional[str] = None  # This matches your Go optional field
    owner: str
    full_name: str


class FilePiResponse(BaseModel):
    """Response model for serving files."""

    total_files: int
    files: list[FileInfo]
    skip: int
    limit: int


class User:
    def __init__(self, username: str, roles: List[str]):
        self.username = username
        self.roles = roles
