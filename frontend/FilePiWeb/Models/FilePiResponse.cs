namespace FilePiWeb.Models;

public class FilePiResponse
{
    public int TotalFiles { get; set; }
    public List<FileInfo> Files { get; set; } = new();
    public int Skip { get; set; }
    public int Limit { get; set; }
}