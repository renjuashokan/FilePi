package main

import (
	"strconv"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	log "github.com/sirupsen/logrus"
)

func RequestLogger() fiber.Handler {
	return func(c *fiber.Ctx) error {
		// Record the start time
		start := time.Now()

		// Process the request
		err := c.Next()

		// Calculate the latency
		latency := time.Since(start)

		// Log the request details using Logrus
		log.WithFields(log.Fields{
			"status":       c.Response().StatusCode(),
			"method":       c.Method(),
			"path":         c.Path(),
			"latency":      latency.String(),
			"ip":           c.IP(),
			"query_params": c.Request().URI().QueryArgs().String(),
		}).Info("HTTP request")

		return err
	}
}

func setupRoutes(fs *FileServer) {

	// Configure Fiber with appropriate settings for file uploads
	app := fiber.New(fiber.Config{
		// Increase the body limit to handle larger file uploads (100MB)
		BodyLimit: 100 * 1024 * 1024,
		// Increase read timeout
		ReadTimeout: 60 * time.Second,
		// Increase write timeout
		WriteTimeout: 60 * time.Second,
		// Enable streaming for multipart form
		StreamRequestBody: true,
	})

	// Add CORS middleware - BEFORE the route definitions
	app.Use(cors.New(cors.Config{
		AllowOrigins: "*",                   // Allow all origins
		AllowMethods: "GET,POST,PUT,DELETE", // Allowed methods
		AllowHeaders: "*",                   // Allow all headers
	}))

	// Add request logger middleware
	app.Use(RequestLogger())

	api := app.Group(API_PREFIX)

	api.Get("/files", func(c *fiber.Ctx) error {
		path := c.Query("path", "")
		skip, _ := strconv.Atoi(c.Query("skip", "0"))
		limit, _ := strconv.Atoi(c.Query("limit", "25"))
		sortBy := c.Query("sort_by", "")
		order := c.Query("order", "asc")

		log.Info("Getting files from path: ", path)

		result, err := fs.GetFiles(path, skip, limit, sortBy, order)
		if err != nil {
			log.Error("Error getting files: ", err)
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": err.Error()})
		}
		return c.JSON(result)
	})

	api.Get("/videos", func(c *fiber.Ctx) error {
		path := c.Query("path", "")
		skip, _ := strconv.Atoi(c.Query("skip", "0"))
		limit, _ := strconv.Atoi(c.Query("limit", "25"))
		recursive, _ := strconv.ParseBool(c.Query("recursive", "true"))
		sortBy := c.Query("sort_by", "")
		order := c.Query("order", "asc")

		result, err := fs.GetVideos(path, skip, limit, recursive, sortBy, order)
		if err != nil {
			log.Error("Error getting files: ", err)
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": err.Error()})
		}
		return c.JSON(result)
	})

	api.Get("/search", func(c *fiber.Ctx) error {
		query := c.Query("query")
		if query == "" {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "Query parameter is required",
			})
		}
		path := c.Query("path", "")
		skip, _ := strconv.Atoi(c.Query("skip", "0"))
		limit, _ := strconv.Atoi(c.Query("limit", "25"))
		sortBy := c.Query("sort_by", "")
		order := c.Query("order", "asc")

		result, err := fs.Search(query, path, skip, limit, sortBy, order)
		if err != nil {
			log.Error("Error getting files: ", err)
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": err.Error()})
		}
		return c.JSON(result)
	})

	// Serve file
	api.Get("/file/*", func(c *fiber.Ctx) error {
		filePath := strings.TrimPrefix(c.Params("*"), "/")
		result, err := fs.ServeFile(filePath)
		if err != nil {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": "File not found",
			})
		}
		return c.SendFile(result)
	})

	// Stream file
	api.Get("/stream/*", func(c *fiber.Ctx) error {
		filePath := strings.TrimPrefix(c.Params("*"), "/")
		result, err := fs.StreamFile(filePath)
		if err != nil {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": "File not found",
			})
		}
		return c.SendFile(result)
	})

	// Get thumbnail
	api.Get("/thumbnail/*", func(c *fiber.Ctx) error {
		filePath := strings.TrimPrefix(c.Params("*"), "/")
		result, err := fs.GetThumbnail(filePath)
		if err != nil {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": "Thumbnail not found",
			})
		}
		return c.SendFile(result)
	})

	// Create folder
	api.Post("/createfolder", func(c *fiber.Ctx) error {
		path := c.FormValue("path")
		folderName := c.FormValue("foldername")

		_, err := fs.CreateFolder(path, folderName)
		if err != nil {
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
		return c.JSON(fiber.Map{
			"message": "Folder created successfully",
		})
	})

	// Modified file upload handler
	api.Post("/uploadfile", func(c *fiber.Ctx) error {
		// Log the beginning of file upload
		log.Info("Starting file upload process")

		// Get form values directly
		location := c.FormValue("location")
		user := c.FormValue("user")

		log.Info("Upload parameters - location: ", location, ", user: ", user)

		if location == "" || user == "" {
			log.Warn("Missing required fields in upload request")
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "Missing required fields: location or user",
			})
		}

		// Get the uploaded file
		file, err := c.FormFile("file")
		if err != nil {
			log.Error("Failed to get form file: ", err)
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "Failed to get file from request: " + err.Error(),
			})
		}

		log.Info("Received file: ", file.Filename, " - Size: ", file.Size)

		// Open the uploaded file
		fileContent, err := file.Open()
		if err != nil {
			log.Error("Failed to open uploaded file: ", err)
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": "Failed to open uploaded file: " + err.Error(),
			})
		}
		defer fileContent.Close()

		// Save the uploaded file
		log.Info("Saving file to location: ", location)
		savedPath, err := fs.saveUploadedFile(fileContent, location, file.Filename)
		if err != nil {
			log.Error("Failed to save uploaded file: ", err)
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": err.Error(),
			})
		}

		log.Info("File uploaded successfully: ", file.Filename, " to path: ", savedPath)

		// Return success response
		return c.JSON(fiber.Map{
			"message":     "File uploaded successfully",
			"filename":    file.Filename,
			"location":    savedPath,
			"uploaded_by": user,
		})
	})

	log.Info("Starting server on port 8080")
	log.Fatal(app.Listen(":8080"))
}
