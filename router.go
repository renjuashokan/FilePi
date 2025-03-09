package main

import (
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/requestid"
	log "github.com/sirupsen/logrus"
)

func setupRoutes(fs *FileServer) {

	app := fiber.New()
	api := app.Group(API_PREFIX)

	app.Use(requestid.New())
	app.Use(logger.New(logger.Config{
		Format:     "${time} ${pid} ${locals:requestid} ${status} - ${method} ${path} ${queryParams}\n",
		TimeFormat: "2006/01/02 15:04:05.000000",
		TimeZone:   "Local",
	}))

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

	// Upload file
	api.Post("/uploadfile", func(c *fiber.Ctx) error {
		// Parse multipart form data
		form, err := c.MultipartForm()
		if err != nil {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "Failed to parse form data",
			})
		}

		// Extract fields from the form
		location := form.Value["location"]
		user := form.Value["user"]
		files, err := c.FormFile("file")
		if err != nil {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "Failed to get form file",
			})
		}

		if len(location) == 0 || len(user) == 0 || files == nil {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "Missing required fields: location, user, or file",
			})
		}

		// Open the uploaded file
		file, err := files.Open()
		if err != nil {
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": "Failed to open uploaded file",
			})
		}
		defer file.Close()

		// Save the uploaded file
		savedPath, err := fs.saveUploadedFile(file, location[0], files.Filename)
		if err != nil {
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": err.Error(),
			})
		}

		// Return success response
		return c.JSON(fiber.Map{
			"message":     "File uploaded successfully",
			"filename":    files.Filename,
			"location":    savedPath,
			"uploaded_by": user[0],
		})
	})

	// Start the server
	log.Fatal(app.Listen(":8080"))
}
