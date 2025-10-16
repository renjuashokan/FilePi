using FilePiWeb.Models;
using Refit;

namespace FilePiWeb.Interfaces;

public interface IFilePiApi
{
    [Get("/api/v1/files")]
    Task<FilePiResponse> GetFilesAsync(
        [Query] string path = "",
        [Query] int skip = 0,
        [Query] int limit = 25,
        [Query("sort_by")] string sortBy = "name",
        [Query] string order = "asc");

    [Get("/api/v1/videos")]
    Task<FilePiResponse> GetVideosAsync(
        [Query] string path = "",
        [Query] int skip = 0,
        [Query] int limit = 25,
        [Query] bool recursive = true,
        [Query("sort_by")] string sortBy = "name",
        [Query] string order = "asc");

    [Get("/api/v1/search")]
    Task<FilePiResponse> SearchFilesAsync(
        [Query] string query,
        [Query] string path = "",
        [Query] int skip = 0,
        [Query] int limit = 25,
        [Query("sort_by")] string sortBy = "name",
        [Query] string order = "asc");

    [Post("/api/v1/createfolder")]
    Task<ApiResponse<object>> CreateFolderAsync(
        [AliasAs("path")] string path,
        [AliasAs("foldername")] string folderName);

    [Multipart]
    [Post("/api/v1/uploadfile")]
    Task<ApiResponse<object>> UploadFileAsync(
        [AliasAs("file")] StreamPart file,
        [AliasAs("location")] string location,
        [AliasAs("user")] string user = "web-user");

    [Post("/api/v1/mv")]
    Task<ApiResponse<object>> MoveFileAsync([Body] MoveRequest request);

    [Get("/api/v1/file/{filePath}")]
    Task<Stream> DownloadFileAsync(string filePath);

    [Get("/api/v1/stream/{filePath}")]
    Task<Stream> StreamVideoAsync(string filePath);

    [Get("/api/v1/thumbnail/{filePath}")]
    Task<Stream> GetThumbnailAsync(string filePath);
}
