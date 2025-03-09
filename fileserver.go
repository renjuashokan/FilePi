package main

import (
	"crypto/md5"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"mime"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"

	"github.com/gofiber/fiber/v2/log"
)

func NewFileServer(rootDir, tempDir string) *FileServer {
	return &FileServer{
		RootDir:         rootDir,
		TempDir:         tempDir,
		ValidSortFields: []string{"name", "size", "modified_time", "created_time", "file_type"},
	}
}

func (fs *FileServer) getAbsolutePath(relativePath string) (string, error) {

	relativePath, err := url.QueryUnescape(relativePath)
	if err != nil {
		log.Error("Error unescaping path: ", err)
		return "", fmt.Errorf("error unescaping path: %w", err)
	}
	cleanPath := filepath.Clean(strings.TrimPrefix(relativePath, "/"))
	absPath := filepath.Join(fs.RootDir, cleanPath)
	log.Info("Absolute path: ", absPath)
	// if !strings.HasPrefix(absPath, fs.RootDir) {
	// 	return "", errors.New("path is outside of root directory")
	// }
	return absPath, nil
}

func (fs *FileServer) getFileOwner(info os.FileInfo) string {
	return "user1"
}

func (fs *FileServer) GetFiles(path string, skip, limit int, sortBy, order string) (*FilePiResonse, error) {

	absPath, err := fs.getAbsolutePath(path)
	if err != nil {
		return nil, err
	}
	entries, err := os.ReadDir(absPath)
	if err != nil {
		return nil, fmt.Errorf("error reading directory: %w", err)
	}
	var contents []FileInfo
	for _, entry := range entries {
		info, err := entry.Info()
		if err != nil {
			continue
		}
		fileType := mime.TypeByExtension(filepath.Ext(entry.Name()))
		if entry.IsDir() {
			fileType = "inode/directory"
		}
		contents = append(contents, FileInfo{
			Name:         entry.Name(),
			Size:         info.Size(),
			IsDirectory:  entry.IsDir(),
			CreatedTime:  info.ModTime().UnixMilli(),
			ModifiedTime: info.ModTime().UnixMilli(),
			FileType:     fileType,
			Owner:        fs.getFileOwner(info),
			FullName:     entry.Name(),
		})
	}

	if sortBy != "" && !contains(fs.ValidSortFields, sortBy) {
		return nil, fmt.Errorf("invalid sort field: %s", sortBy)
	}

	if sortBy != "" {
		reverse := order == "desc"
		sort.Slice(contents, func(i, j int) bool {
			switch sortBy {
			case "name":
				if reverse {
					return contents[i].Name > contents[j].Name
				}
				return contents[i].Name < contents[j].Name
			case "size":
				if reverse {
					return contents[i].Size > contents[j].Size
				}
				return contents[i].Size < contents[j].Size
			case "modified_time":
				if reverse {
					return contents[i].ModifiedTime > contents[j].ModifiedTime
				}
				return contents[i].ModifiedTime < contents[j].ModifiedTime
			case "created_time":
				if reverse {
					return contents[i].CreatedTime > contents[j].CreatedTime
				}
				return contents[i].CreatedTime < contents[j].CreatedTime
			case "file_type":
				if reverse {
					return contents[i].FileType > contents[j].FileType
				}
				return contents[i].FileType < contents[j].FileType
			default:
				return false
			}
		})
	}
	totalFiles := len(contents)
	paginatedFiles := contents[skip:]
	if limit > 0 && limit < len(paginatedFiles) {
		paginatedFiles = paginatedFiles[:limit]
	}
	return &FilePiResonse{
		TotalFiles: totalFiles,
		Files:      paginatedFiles,
		Skip:       skip,
		Limit:      limit,
	}, nil
}

func (fo *FileServer) GetVideos(path string, skip, limit int, recursive bool, sortBy, order string) (*FilePiResonse, error) {
	absPath, err := fo.getAbsolutePath(path)
	if err != nil {
		return nil, err
	}
	var videoContents []FileInfo
	err = filepath.Walk(absPath, func(path string, info os.FileInfo, err error) error {

		if err != nil {
			return err
		}
		if info.IsDir() && !recursive {
			return filepath.SkipDir
		}
		if !info.IsDir() {
			relPath, _ := filepath.Rel(absPath, path)
			mimeType := mime.TypeByExtension(filepath.Ext(info.Name()))
			if mimeType != "" && strings.HasPrefix(mimeType, "video/") {
				videoContents = append(videoContents, FileInfo{
					Name:         info.Name(),
					Size:         info.Size(),
					IsDirectory:  false,
					CreatedTime:  info.ModTime().UnixMilli(),
					ModifiedTime: info.ModTime().UnixMilli(),
					FileType:     mimeType,
					Owner:        fo.getFileOwner(info),
					FullName:     relPath,
				})
			}
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("error reading directory: %w", err)
	}
	if sortBy != "" && !contains(fo.ValidSortFields, sortBy) {
		return nil, fmt.Errorf("invalid sort field: %s", sortBy)
	}
	if sortBy != "" {
		reverse := order == "desc"
		sort.Slice(videoContents, func(i, j int) bool {
			switch sortBy {
			case "name":
				if reverse {
					return videoContents[i].Name > videoContents[j].Name
				}
				return videoContents[i].Name < videoContents[j].Name
			case "size":
				if reverse {
					return videoContents[i].Size > videoContents[j].Size
				}
				return videoContents[i].Size < videoContents[j].Size
			case "modified_time":
				if reverse {
					return videoContents[i].ModifiedTime > videoContents[j].ModifiedTime
				}
				return videoContents[i].ModifiedTime < videoContents[j].ModifiedTime
			case "created_time":
				if reverse {
					return videoContents[i].CreatedTime > videoContents[j].CreatedTime
				}
				return videoContents[i].CreatedTime < videoContents[j].CreatedTime
			case "file_type":
				if reverse {
					return videoContents[i].FileType > videoContents[j].FileType
				}
				return videoContents[i].FileType < videoContents[j].FileType
			default:
				return false
			}
		})
	}
	totalFiles := len(videoContents)
	paginatedFiles := videoContents[skip:]
	if limit > 0 && limit < len(paginatedFiles) {
		paginatedFiles = paginatedFiles[:limit]
	}
	return &FilePiResonse{
		TotalFiles: totalFiles,
		Files:      paginatedFiles,
		Skip:       skip,
		Limit:      limit,
	}, nil
}

func (fo *FileServer) Search(query, path string, skip, limit int, sortBy, order string) (*FilePiResonse, error) {
	absPath, err := fo.getAbsolutePath(path)
	if err != nil {
		return nil, err
	}
	var matchingFiles []FileInfo
	err = filepath.Walk(absPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.Contains(strings.ToLower(info.Name()), strings.ToLower(query)) {
			matchingFiles = append(matchingFiles, FileInfo{
				Name:         info.Name(),
				Size:         info.Size(),
				IsDirectory:  false,
				CreatedTime:  info.ModTime().UnixMilli(),
				ModifiedTime: info.ModTime().UnixMilli(),
				FileType:     mime.TypeByExtension(filepath.Ext(info.Name())),
				Owner:        fo.getFileOwner(info),
				FullName:     strings.TrimPrefix(path, fo.RootDir),
			})
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("error reading directory: %w", err)
	}
	if sortBy != "" && !contains(fo.ValidSortFields, sortBy) {
		return nil, fmt.Errorf("invalid sort field: %s", sortBy)
	}
	if sortBy != "" {
		reverse := order == "desc"
		sort.Slice(matchingFiles, func(i, j int) bool {
			switch sortBy {
			case "name":
				if reverse {
					return matchingFiles[i].Name > matchingFiles[j].Name
				}
				return matchingFiles[i].Name < matchingFiles[j].Name
			case "size":
				if reverse {
					return matchingFiles[i].Size > matchingFiles[j].Size
				}
				return matchingFiles[i].Size < matchingFiles[j].Size
			case "modified_time":
				if reverse {
					return matchingFiles[i].ModifiedTime > matchingFiles[j].ModifiedTime
				}
				return matchingFiles[i].ModifiedTime < matchingFiles[j].ModifiedTime
			case "created_time":
				if reverse {
					return matchingFiles[i].CreatedTime > matchingFiles[j].CreatedTime
				}
				return matchingFiles[i].CreatedTime < matchingFiles[j].CreatedTime
			case "file_type":
				if reverse {
					return matchingFiles[i].FileType > matchingFiles[j].FileType
				}
				return matchingFiles[i].FileType < matchingFiles[j].FileType
			default:
				return false
			}
		})
	}
	totalFiles := len(matchingFiles)
	paginatedFiles := matchingFiles[skip:]
	if limit > 0 && limit < len(paginatedFiles) {
		paginatedFiles = paginatedFiles[:limit]
	}
	return &FilePiResonse{
		TotalFiles: totalFiles,
		Files:      paginatedFiles,
		Skip:       skip,
		Limit:      limit,
	}, nil
}

func (fo *FileServer) ServeFile(path string) (string, error) {
	absPath, err := fo.getAbsolutePath(path)
	if err != nil {
		return "", err
	}
	if _, err := os.Stat(absPath); os.IsNotExist(err) {
		return "", fmt.Errorf("file %s does not exist", absPath)
	}
	return absPath, nil
}

func (fo *FileServer) StreamFile(path string) (string, error) {
	absPath, err := fo.getAbsolutePath(path)
	if err != nil {
		return "", err
	}
	if _, err := os.Stat(absPath); os.IsNotExist(err) {
		return "", fmt.Errorf("video file %s does not exist", absPath)
	}
	mimeType := mime.TypeByExtension(filepath.Ext(absPath))
	if mimeType == "" || !strings.HasPrefix(mimeType, "video/") {
		return "", errors.New("not a video file")
	}
	return absPath, nil
}

func (fs *FileServer) GetThumbnail(path string) (string, error) {
	absPath, err := fs.getAbsolutePath(path)
	if err != nil {
		return "", err
	}
	if _, err := os.Stat(absPath); os.IsNotExist(err) {
		log.Error("File does not exist: ", absPath)
		return "", fmt.Errorf("file %s does not exist", absPath)
	}
	hash := fs.getMD5Hash(absPath)
	outputDir := filepath.Join(fs.TempDir, hash)
	outputPath := filepath.Join(outputDir, "thumbnail.jpg")
	if _, err := os.Stat(outputPath); err == nil {
		return outputPath, nil
	}
	if err := os.MkdirAll(outputDir, os.ModePerm); err != nil {
		log.Error("Error creating thumbnail directory: ", err)
		return "", fmt.Errorf("failed to create thumbnail directory: %w", err)
	}
	cmd := exec.Command("ffmpeg", "-i", absPath, "-ss", "00:00:01", "-vframes", "1", "-vf", "scale=320:-1", "-y", outputPath)
	if err := cmd.Run(); err != nil {
		log.Error("Error generating thumbnail: ", err)
		return "", fmt.Errorf("error generating thumbnail: %w", err)
	}
	return outputPath, nil
}

func (fs *FileServer) getMD5Hash(input string) string {
	hash := md5.Sum([]byte(input))
	return hex.EncodeToString(hash[:])
}

func (fs *FileServer) CreateFolder(path, folderName string) (string, error) {
	absPath, err := fs.getAbsolutePath(path)
	if err != nil {
		return "", err
	}
	newFolderPath := filepath.Join(absPath, folderName)
	if err := os.MkdirAll(newFolderPath, os.ModePerm); err != nil {
		return "", fmt.Errorf("unable to create folder: %w", err)
	}
	return newFolderPath, nil
}

func (fs *FileServer) saveUploadedFile(file io.Reader, location, fileName string) (string, error) {
	absPath, err := fs.getAbsolutePath(location)
	if err != nil {
		return "", err
	}
	if err := os.MkdirAll(absPath, os.ModePerm); err != nil {
		return "", fmt.Errorf("failed to create directory: %w", err)
	}
	filePath := filepath.Join(absPath, fileName)
	out, err := os.Create(filePath)
	if err != nil {
		return "", fmt.Errorf("failed to create file: %w", err)
	}
	defer out.Close()
	if _, err := io.Copy(out, file); err != nil {
		return "", fmt.Errorf("failed to save file: %w", err)
	}
	return filePath, nil
}

func contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}
