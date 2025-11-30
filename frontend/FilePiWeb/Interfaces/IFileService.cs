using FilePiWeb.Models;
using Microsoft.AspNetCore.Components.Forms;

namespace FilePiWeb.Interfaces;

public interface IFileService
{
    Task<FilePiResponse?> GetFilesAsync(string path = "", int skip = 0, int limit = 25,
        string sortBy = "name", string order = "asc");
    Task<FilePiResponse?> GetVideosAsync(string path = "", int skip = 0, int limit = 25,
        bool recursive = true, string sortBy = "name", string order = "asc");
    Task<FilePiResponse?> SearchFilesAsync(string query, string path = "", int skip = 0,
        int limit = 25, string sortBy = "name", string order = "asc");
    Task<bool> CreateFolderAsync(string path, string folderName);
    Task<bool> UploadFileAsync(IBrowserFile file, string location, string user = "web-user");
    Task<bool> MoveFileAsync(string oldPath, string newPath);
    Task<Stream?> DownloadFileAsync(string filePath);
    Task<Stream?> StreamVideoAsync(string filePath);
    Task<Stream?> GetThumbnailAsync(string filePath);
}
