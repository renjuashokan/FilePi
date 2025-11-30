namespace FilePiWeb.Models;

public class FileInfo
{
    public string Name { get; set; } = "";
    public string FullName { get; set; } = "";
    public long Size { get; set; }
    public bool IsDirectory { get; set; }
    public long CreatedTime { get; set; }
    public long ModifiedTime { get; set; }
    public string? FileType { get; set; }
    public string Owner { get; set; } = "";
    public string? ParentDir { get; set; }
    public string RelPath { get; set; } = "";
}