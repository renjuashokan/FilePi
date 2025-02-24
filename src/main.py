import uvicorn

from . import LOGGING_CONFIG, app

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8080, log_config=LOGGING_CONFIG, log_level="info")
