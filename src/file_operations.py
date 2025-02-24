import hashlib
import logging
import mimetypes
import os
import shutil
import subprocess
from pathlib import Path

from fastapi import UploadFile
from fastapi.responses import FileResponse

from .models import FileInfo, FilePiResponse

logger = logging.getLogger("file_operations")

valid_sort_fields = ["name", "size", "modified_time", "created_time", "file_type"]


class FileOperations:
    def __init__(self):
        """
        Initialize the FileOperations class
        """
        current_dir = os.path.dirname(os.path.realpath(__file__))
        self.root_dir = os.getenv("FILE_PI_ROOT_DIR", current_dir)
        if not os.path.exists(self.root_dir):
            raise ValueError(f"Root directory {self.root_dir} does not exist")

        self.temp_dir = os.path.join(self.root_dir, ".cache")
        os.makedirs(self.temp_dir, exist_ok=True)
        logger.info(f"Root directory: {self.root_dir}")

    def get_absolute_path(self, relative_path: str) -> str:
        """
        Convert relative path to absolute path within root directory
        """
        # Clean the path to prevent directory traversal
        clean_path = os.path.normpath(relative_path).lstrip("/")
        return str(Path(self.root_dir) / clean_path)

    def get_directory_contents(
        self,
        path: str = "",
        skip: int = 0,
        limit: int = 25,
        sort_by: str = "",
        order: str = "",
    ) -> FilePiResponse:
        """
        Get the contents of a directory
        """
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        contents = []
        try:
            for entry in os.scandir(abs_path):
                stats = entry.stat()
                file_type = "inode/directory" if entry.is_dir() else mimetypes.guess_type(entry.name)[0]

                file_info = FileInfo(
                    name=entry.name,
                    size=stats.st_size,
                    is_directory=entry.is_dir(),
                    created_time=int(stats.st_ctime * 1000),
                    modified_time=int(stats.st_mtime * 1000),
                    file_type=file_type,
                    owner=self._get_file_owner(entry.path),
                    full_name=entry.name,
                )
                contents.append(file_info)
        except Exception as e:
            raise Exception(f"Error reading directory: {str(e)}")

        if sort_by and sort_by not in valid_sort_fields:
            raise ValueError(f"Invalid sort field: {sort_by}")

        if sort_by:
            reverse = True if order == "desc" else False
            contents.sort(key=lambda x: getattr(x, sort_by), reverse=reverse)

        total_files = len(contents)
        paginated_files = contents[skip : skip + limit]

        return FilePiResponse(
            total_files=total_files,
            files=paginated_files,
            skip=skip,
            limit=limit,
        )

    def _get_file_owner(self, path: str) -> str:
        # Implementation of getting file owner
        return "renju"  # Placeholder

    def serve_file(self, path: str) -> FileResponse:
        """
        Serve a file from the given path
        Returns: FastAPI FileResponse
        """
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        if not os.path.isfile(abs_path):
            raise FileNotFoundError(f"File {abs_path} does not exist")

        return FileResponse(
            abs_path,
            filename=os.path.basename(abs_path),
            media_type=mimetypes.guess_type(abs_path)[0] or "application/octet-stream",
        )

    def stream_file(self, path: str) -> FileResponse:
        """
        Stream a file from the given path
        """
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        if not os.path.isfile(abs_path):
            raise FileNotFoundError(f"Video file {abs_path} does not exist")

        mime_type, _ = mimetypes.guess_type(abs_path)
        if not mime_type or not mime_type.startswith("video/"):
            raise ValueError("Not a video file")

        return FileResponse(path=abs_path, media_type=mime_type, filename=os.path.basename(abs_path))

    def get_thumbnail(self, path: str) -> FileResponse:
        """
        Get a thumbnail for the given file
        """
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        if not os.path.isfile(abs_path):
            raise FileNotFoundError(f"File {abs_path} does not exist")

        thumbnail_path = self._generate_thumbnail(abs_path)
        return FileResponse(
            thumbnail_path,
            filename=os.path.basename(thumbnail_path),
            media_type="image/jpeg",
        )

    def _get_md5_hash(self, path: str) -> str:
        """
        Get the MD5 hash of a string
        """
        return hashlib.md5(path.encode("utf-8")).hexdigest()

    def _generate_thumbnail(self, input_path: str) -> str:
        """
        Generate the thumbnail of video file with ffmpeg
        """
        if not os.path.isfile(input_path):
            raise FileNotFoundError(f"Video file {input_path} does not exist")

        output_dir = os.path.join(self.temp_dir, self._get_md5_hash(input_path))

        output_path = os.path.join(output_dir, "thumbnail.jpg")

        # if the file already exists, return
        if os.path.exists(output_path):
            return output_path

        logger.info(f"Generating thumbnail for video: {input_path}")

        os.makedirs(output_dir, exist_ok=True)
        timestamp = "00:00:01"
        cmd = [
            "ffmpeg",
            "-i",
            input_path,
            "-ss",
            timestamp,
            "-vframes",
            "1",
            "-vf",
            "scale=320:-1",  # Width 320px, height auto
            "-y",  # Overwrite output file if exists
            output_path,
        ]

        try:
            subprocess.run(cmd, check=True, capture_output=True, text=True)
        except subprocess.CalledProcessError as e:
            raise Exception(f"Error generating thumbnail: {e.stderr}")

        return output_path

    def create_folder(self, path: str, folder_name: str) -> str:
        """
        Create a new folder at the specified path
        Returns: Created folder path
        """
        logger.debug(f"Creating folder '{folder_name}' at path: {path}")

        # Get absolute path and validate
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        # Create full folder path
        new_folder_path = os.path.join(abs_path, folder_name)

        try:
            os.makedirs(new_folder_path, exist_ok=True)
            logger.info(f"Created folder: {new_folder_path}")
            return os.path.join(path, folder_name)
        except Exception as e:
            logger.error(f"Error creating folder: {str(e)}")
            raise Exception(f"Unable to create folder: {str(e)}")

    async def save_uploaded_file(self, file: UploadFile, location: str, user: str) -> str:
        """
        Save an uploaded file to the specified location
        """
        logger.debug(f"Uploading file from user {user} to location: {location}")

        # Get the absolute path and validate it
        abs_path = self.get_absolute_path(location)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        # Create directory if it doesn't exist
        os.makedirs(abs_path, exist_ok=True)

        # Create full file path
        file_path = os.path.join(abs_path, file.filename)
        logger.debug(f"Saving file to: {file_path}")

        try:
            # Save the file
            with open(file_path, "wb") as buffer:
                shutil.copyfileobj(file.file, buffer)
        except Exception as e:
            logger.error(f"Error saving file: {str(e)}")
            raise Exception(f"Unable to save file: {str(e)}")

        return os.path.join(location, file.filename)

    def get_videos(
        self,
        path: str = "",
        skip: int = 0,
        limit: int = 25,
        recursive: bool = True,
        sort_by: str = "",
        order: str = "",
    ) -> FilePiResponse:
        """
        Get the video contents of a directory. Filters only video files (e.g., mp4, avi).
        If recursive is True, it will search for video files in subdirectories as well.
        Supports pagination and sorting.
        """
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        video_contents = []

        def _collect_videos(directory_path: str):
            """Helper function to collect video files."""
            try:
                for entry in os.scandir(directory_path):
                    if entry.is_dir(follow_symlinks=False) and recursive:
                        # Recursively process subdirectories if recursive is True
                        _collect_videos(entry.path)
                    elif entry.is_file():
                        # Check if the file is a video by guessing its MIME type
                        mime_type, _ = mimetypes.guess_type(entry.name)
                        if mime_type and mime_type.startswith("video/"):
                            stats = entry.stat()
                            relative_path = os.path.relpath(entry.path, abs_path)
                            file_info = FileInfo(
                                name=entry.name,
                                size=stats.st_size,
                                is_directory=False,
                                created_time=int(stats.st_ctime * 1000),
                                modified_time=int(stats.st_mtime * 1000),
                                file_type=mime_type,
                                owner=self._get_file_owner(entry.path),
                                full_name=relative_path,
                            )
                            video_contents.append(file_info)
            except Exception as e:
                raise Exception(f"Error reading directory: {str(e)}")

        # Start collecting video files from the given path
        _collect_videos(abs_path)

        # Sorting logic
        if sort_by and sort_by not in valid_sort_fields:
            raise ValueError(f"Invalid sort field: {sort_by}")
        if sort_by:
            reverse = True if order == "desc" else False
            video_contents.sort(key=lambda x: getattr(x, sort_by), reverse=reverse)

        # Pagination
        total_files = len(video_contents)
        paginated_files = video_contents[skip : skip + limit]

        return FilePiResponse(
            total_files=total_files,
            files=paginated_files,
            skip=skip,
            limit=limit,
        )

    def search(
        self,
        query: str,
        path: str = "",
        skip: int = 0,
        limit: int = 25,
        sort_by: str = "",
        order: str = "",
    ) -> FilePiResponse:
        """
        Recursively search for files whose names contain the given query (case-insensitive).
        Supports pagination and sorting.
        """
        abs_path = self.get_absolute_path(path)
        if not abs_path.startswith(self.root_dir):
            raise ValueError("Path is outside of root directory")

        matching_files = []

        def _recursive_search(directory_path: str):
            """Helper function to recursively search for files."""
            try:
                for entry in os.scandir(directory_path):
                    if entry.is_dir(follow_symlinks=False):
                        # Recursively process subdirectories
                        _recursive_search(entry.path)
                    elif entry.is_file():
                        # Check if the file name contains the query (case-insensitive)
                        if query.lower() in entry.name.lower():
                            stats = entry.stat()
                            file_type = mimetypes.guess_type(entry.name)[0]
                            relative_path = os.path.relpath(entry.path, abs_path)
                            file_info = FileInfo(
                                name=entry.name,
                                size=stats.st_size,
                                is_directory=False,
                                created_time=int(stats.st_ctime * 1000),
                                modified_time=int(stats.st_mtime * 1000),
                                file_type=file_type,
                                owner=self._get_file_owner(entry.path),
                                full_name=relative_path,  # Populate the full_name field
                            )
                            matching_files.append(file_info)
            except Exception as e:
                raise Exception(f"Error reading directory: {str(e)}")

        # Start searching from the given path
        _recursive_search(abs_path)

        # Sorting logic
        if sort_by and sort_by not in valid_sort_fields:
            raise ValueError(f"Invalid sort field: {sort_by}")
        if sort_by:
            reverse = True if order == "desc" else False
            matching_files.sort(key=lambda x: getattr(x, sort_by), reverse=reverse)

        # Pagination
        total_files = len(matching_files)
        paginated_files = matching_files[skip : skip + limit]

        return FilePiResponse(
            total_files=total_files,
            files=paginated_files,
            skip=skip,
            limit=limit,
        )
