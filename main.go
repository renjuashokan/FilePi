package main

import (
	"fmt"
	"os"
	"path/filepath"

	log "github.com/sirupsen/logrus"
)

func main() {
	logLevl := os.Getenv("FILE_PI_LOGLEVEL")
	if logLevl == "" {
		logLevl = "DEBUG"
	}

	lv, err := log.ParseLevel(logLevl)
	if err != nil {
		fmt.Printf("Invalid log level '%s', defaulting to INFO\n", logLevl)
		lv = log.InfoLevel
	}
	log.SetLevel(lv)
	log.SetFormatter(&log.TextFormatter{TimestampFormat: "2006-01-02 15:04:05", FullTimestamp: true})
	log.Info("Starting!")

	wd, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}

	rootDir := os.Getenv("FILE_PI_ROOT_DIR")
	if rootDir == "" {
		rootDir = wd
	}
	log.Infof("Root directory: %s", rootDir)

	tempDir := filepath.Join(rootDir, ".cache")

	fs := NewFileServer(rootDir, tempDir)
	log.Info("Starting server on root directory: ->", rootDir)
	setupRoutes(fs)
}
