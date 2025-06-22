package main

const (
	API_PREFIX = "/api/v1"
	HTTP_PORT  = "8080"
)

type FileServer struct {
	RootDir         string
	TempDir         string
	ValidSortFields []string
}

type FileInfo struct {
	Name         string `json:"name"`
	FullName     string `json:"full_name"`
	Size         int64  `json:"size"`
	IsDirectory  bool   `json:"is_directory"`
	CreatedTime  int64  `json:"created_time"`
	ModifiedTime int64  `json:"modified_time"`
	FileType     string `json:"file_type,omitempty"`
	Owner        string `json:"owner"`
	// the parent directory of the file
	ParentDir string `json:"parent_dir,omitempty"`
	// path to file from $FILE_PI_ROOT_DIR
	RelPath string `json:"rel_path,omitempty"`
}

type FilePiResonse struct {
	TotalFiles int        `json:"total_files"`
	Files      []FileInfo `json:"files"`
	Skip       int        `json:"skip"`
	Limit      int        `json:"limit"`
}

type MoveRequest struct {
	OldPath string `json:"old_path"`
	NewPath string `json:"new_path"`
}
