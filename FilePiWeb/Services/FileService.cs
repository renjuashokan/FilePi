using FilePiWeb.Interfaces;
using FilePiWeb.Models;
using Refit;
using Microsoft.AspNetCore.Components.Forms;

namespace FilePiWeb.Services;

public class FileService : IFileService
{
    private readonly IFilePiApi _api;
    private readonly ILogger<FileService> _logger;

    public FileService(IFilePiApi api, ILogger<FileService> logger)
    {
        _api = api;
        _logger = logger;
    }

    public async Task<FilePiResponse?> GetFilesAsync(string path = "", int skip = 0, int limit = 25, string sortBy = "name", string order = "asc")
    {
        try
        {
            _logger.LogInformation("Loading files from path: {Path} with pagination Skip={Skip}, Take={Take}",
                path, skip, limit);

            var response = await _api.GetFilesAsync(path, skip, limit, sortBy, order);

            _logger.LogInformation("Successfully loaded {FileCount} files from {Path}",
                response.Files?.Count ?? 0, path);

            return response;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to load files from path: {Path}", path);
            return null;
        }
    }

    public async Task<FilePiResponse?> GetVideosAsync(string path = "", int skip = 0, int limit = 25, bool recursive = true, string sortBy = "name", string order = "asc")
    {
        try
        {
            _logger.LogInformation("Loading videos from path: {Path}", path);
            var response = await _api.GetVideosAsync(path, skip, limit, recursive, sortBy, order);
            _logger.LogInformation("Successfully loaded {VideoCount} videos from {Path}",
                response.Files?.Count ?? 0, path);
            return response;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to load videos from path: {Path}", path);
            return null;
        }
    }

    public async Task<FilePiResponse?> SearchFilesAsync(string query, string path = "", int skip = 0, int limit = 25, string sortBy = "name", string order = "asc")
    {
        try
        {
            _logger.LogInformation("Searching for '{Query}' in path: {Path}", query, path);
            var response = await _api.SearchFilesAsync(query, path, skip, limit, sortBy, order);
            _logger.LogInformation("Search returned {ResultCount} results for '{Query}'",
                response.Files?.Count ?? 0, query);
            return response;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to search for '{Query}' in path: {Path}", query, path);
            return null;
        }
    }

    public async Task<bool> CreateFolderAsync(string path, string folderName)
    {
        try
        {
            _logger.LogInformation("Creating folder '{FolderName}' in path: {Path}", folderName, path);
            var response = await _api.CreateFolderAsync(path, folderName);
            var success = response.IsSuccessStatusCode;

            if (success)
                _logger.LogInformation("Successfully created folder '{FolderName}' in {Path}", folderName, path);
            else
                _logger.LogWarning("Failed to create folder '{FolderName}' in {Path}. Status: {StatusCode}",
                    folderName, path, response.StatusCode);

            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error creating folder '{FolderName}' in {Path}", folderName, path);
            return false;
        }
    }

    public async Task<bool> UploadFileAsync(IBrowserFile file, string location, string user = "web-user")
    {
        try
        {
            if (file.Name.Contains("..") || file.Name.IndexOfAny(Path.GetInvalidFileNameChars()) >= 0)
            {
                _logger.LogWarning("Invalid file name attempted: {FileName}", file.Name);
                return false;
            }
            _logger.LogInformation("Uploading file '{FileName}' to location: {Location}", file.Name, location);

            using (var stream = file.OpenReadStream(maxAllowedSize: 100 * 1024 * 1024 * 5))
            {
                var streamPart = new StreamPart(stream, file.Name, file.ContentType);

                var response = await _api.UploadFileAsync(streamPart, location, user);
                var success = response.IsSuccessStatusCode;

                if (success)
                    _logger.LogInformation("Successfully uploaded file '{FileName}' to {Location}", file.Name, location);
                else
                    _logger.LogWarning("Failed to upload file '{FileName}' to {Location}. Status: {StatusCode}",
                        file.Name, location, response.StatusCode);

                return success;
            }

        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error uploading file '{FileName}' to {Location}", file.Name, location);
            return false;
        }
    }

    public async Task<bool> MoveFileAsync(string oldPath, string newPath)
    {
        try
        {
            _logger.LogInformation("Moving file from '{OldPath}' to '{NewPath}'", oldPath, newPath);

            var request = new MoveRequest() { OldPath = oldPath, NewPath = newPath };
            var response = await _api.MoveFileAsync(request);
            var success = response.IsSuccessStatusCode;

            if (success)
                _logger.LogInformation("Successfully moved file from '{OldPath}' to '{NewPath}'", oldPath, newPath);
            else
                _logger.LogWarning("Failed to move file from '{OldPath}' to '{NewPath}'. Status: {StatusCode}",
                    oldPath, newPath, response.StatusCode);

            return success;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error moving file from '{OldPath}' to '{NewPath}'", oldPath, newPath);
            return false;
        }
    }

    public async Task<Stream?> DownloadFileAsync(string filePath)
    {
        try
        {
            _logger.LogInformation("Downloading file: {FilePath}", filePath);
            var encodedPath = Uri.EscapeDataString(filePath);
            var stream = await _api.DownloadFileAsync(encodedPath);
            _logger.LogInformation("Successfully initiated download for: {FilePath}", encodedPath);
            return stream;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to download file: {FilePath}", filePath);
            return null;
        }
    }

    public async Task<Stream?> StreamVideoAsync(string filePath)
    {
        try
        {
            _logger.LogInformation("Streaming video: {FilePath}", filePath);
            return await _api.StreamVideoAsync(filePath);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to stream video: {FilePath}", filePath);
            return null;
        }
    }

    public async Task<Stream?> GetThumbnailAsync(string filePath)
    {
        try
        {
            return await _api.GetThumbnailAsync(filePath);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to get thumbnail for: {FilePath}", filePath);
            return null;
        }
    }
}