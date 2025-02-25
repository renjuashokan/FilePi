

def run_server():
    """
    Entrypoint for the filepi command.
    """
    from . import LOGGING_CONFIG, app
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8080, log_config=LOGGING_CONFIG, log_level="info")

if __name__ == "__main__":
    run_server()
