from typing import Optional

from fastapi import APIRouter, Form, HTTPException, Query, UploadFile

from ..file_operations import FileOperations
from ..models import FilePiResponse

router = APIRouter()
file_ops = FileOperations()


@router.get("/files", response_model=FilePiResponse)
async def list_files(
    path: str = "",
    skip: int = 0,
    limit: int = 25,
    sort_by: Optional[str] = Query(
        None,
        description="Field to sort by: name, modified_time, created_time, file_type, size",
    ),
    order: Optional[str] = Query("asc", description="Sort order: asc or desc"),
):
    try:
        return file_ops.get_directory_contents(path, skip, limit, sort_by, order)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/videos", response_model=FilePiResponse)
async def list_videos(
    path: str = "",
    skip: int = 0,
    limit: int = 25,
    recursive: Optional[bool] = Query(True, description="Recursively list videos in subdirectories"),
    sort_by: Optional[str] = Query(
        None,
        description="Field to sort by: name, modified_time, created_time, file_type, size",
    ),
    order: Optional[str] = Query("asc", description="Sort order: asc or desc"),
):
    try:
        return file_ops.get_videos(path, skip, limit, recursive, sort_by, order)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/search", response_model=FilePiResponse)
async def search_files(
    query: str = Query(..., description="Substring to search for in file names"),
    path: str = Query("", description="Directory path to start the search"),
    skip: int = Query(0, description="Number of files to skip for pagination"),
    limit: int = Query(25, description="Maximum number of files to return"),
    sort_by: Optional[str] = Query(None, description="Field to sort by"),
    order: Optional[str] = Query("asc", description="Sort order ('asc' or 'desc')"),
):
    try:
        return file_ops.search(query=query, path=path, skip=skip, limit=limit, sort_by=sort_by, order=order)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/file/{file_path:path}")
async def serve_file(file_path: str):
    try:
        return file_ops.serve_file(file_path)
    except FileNotFoundError:
        raise HTTPException(status_code=404, detail="File not found")
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/stream/{file_path:path}")
async def stream_file(file_path: str):
    try:
        return file_ops.stream_file(file_path)
    except FileNotFoundError:
        raise HTTPException(status_code=404, detail="File not found")
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/thumbnail/{file_path:path}")
async def get_thumbnail(file_path: str):
    try:
        return file_ops.get_thumbnail(file_path)
    except FileNotFoundError:
        raise HTTPException(status_code=404, detail="File not found")
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/createfolder")
async def create_folder(path: str, foldername: str):
    try:
        created_path = file_ops.create_folder(path, foldername)
        return {
            "message": f"Folder created successfully at {created_path}",
            "path": created_path,
        }
    except ValueError as e:
        # logger.warning(f"Bad request in folder creation: {str(e)}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        # logger.error(f"Error in folder creation: {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/uploadfile")
async def upload_file(
    file: UploadFile,
    location: str = Form(...),
    user: str = Form(...),
):
    try:
        # Validate incoming file
        if not file.filename:
            raise ValueError("No file selected")

        # Save the file
        saved_path = await file_ops.save_uploaded_file(file, location, user)

        return {
            "message": f"File uploaded successfully to {saved_path}",
            "filename": file.filename,
            "location": saved_path,
        }

    except ValueError as e:
        # logger.warning(f"Bad request in file upload: {str(e)}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        # logger.error(f"Error in file upload: {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))
