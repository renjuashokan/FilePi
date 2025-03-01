import os
import sys

from setuptools import find_packages, setup

version = os.environ.get("PACKAGE_VERSION")

# Check if version is passed as an argument
for i, arg in enumerate(sys.argv):
    if arg == "--package-version" and i + 1 < len(sys.argv):
        version = sys.argv[i + 1]
        # Remove these arguments so setuptools doesn't see them
        sys.argv.pop(i)
        sys.argv.pop(i)
        break

if not version:
    print("Error: Version is required. Please set PACKAGE_VERSION environment variable or use --package-version")
    sys.exit(1)

print(f"Building filepi version {version}")

setup(
    name="filepi",
    version=version,
    packages=find_packages(),
    include_package_data=True,
    install_requires=[
        "annotated-types==0.7.0",
        "anyio==4.8.0",
        "appdirs==1.4.4",
        "appnope==0.1.4",
        "argon2-cffi==23.1.0",
        "argon2-cffi-bindings==21.2.0",
        "arrow==1.3.0",
        "asttokens==3.0.0",
        "async-lru==2.0.4",
        "attrs==25.1.0",
        "babel==2.17.0",
        "bdfr==2.6.2",
        "beautifulsoup4==4.13.3",
        "black==25.1.0",
        "bleach==6.1.0",
        "certifi==2025.1.31",
        "cffi==1.17.1",
        "charset-normalizer==3.4.1",
        "click==8.1.8",
        "comm==0.2.2",
        "cryptography==44.0.1",
        "debugpy==1.8.12",
        "decorator==5.2.1",
        "defusedxml==0.7.1",
        "dict2xml==1.7.6",
        "ecdsa==0.19.0",
        "exceptiongroup==1.2.2",
        "executing==2.2.0",
        "fastapi==0.115.8",
        "fastjsonschema==2.21.1",
        "fqdn==1.5.1",
        "h11==0.14.0",
        "httpcore==1.0.7",
        "httptools==0.6.4",
        "httpx==0.28.1",
        "idna==3.10",
        "importlib_metadata==8.6.1",
        "ipykernel==6.29.5",
        "ipython==8.18.1",
        "isoduration==20.11.0",
        "isort==6.0.0",
        "jedi==0.19.2",
        "Jinja2==3.1.5",
        "jose==1.0.0",
        "json5==0.10.0",
        "jsonpointer==3.0.0",
        "jsonschema==4.23.0",
        "jsonschema-specifications==2024.10.1",
        "jupyter-events==0.12.0",
        "jupyter-lsp==2.2.5",
        "jupyter_client==8.6.3",
        "jupyter_core==5.7.2",
        "jupyter_server==2.15.0",
        "jupyter_server_terminals==0.5.3",
        "jupyterlab==4.3.5",
        "jupyterlab_pygments==0.3.0",
        "jupyterlab_server==2.27.3",
        "MarkupSafe==3.0.2",
        "matplotlib-inline==0.1.7",
        "mistune==3.1.2",
        "mypy-extensions==1.0.0",
        "nbclient==0.10.2",
        "nbconvert==7.16.6",
        "nbformat==5.10.4",
        "nest-asyncio==1.6.0",
        "notebook==7.3.2",
        "notebook_shim==0.2.4",
        "overrides==7.7.0",
        "packaging==24.2",
        "pandocfilters==1.5.1",
        "parso==0.8.4",
        "passlib==1.7.4",
        "pathspec==0.12.1",
        "pexpect==4.9.0",
        "pip-review==1.3.0",
        "platformdirs==4.3.6",
        "praw==7.8.1",
        "prawcore==2.4.0",
        "prometheus_client==0.21.1",
        "prompt_toolkit==3.0.50",
        "psutil==7.0.0",
        "ptyprocess==0.7.0",
        "pure_eval==0.2.3",
        "pyasn1==0.4.8",
        "pycparser==2.22",
        "pydantic==2.10.6",
        "pydantic_core==2.27.2",
        "Pygments==2.19.1",
        "python-dateutil==2.9.0.post0",
        "python-dotenv==1.0.1",
        "python-jose==3.4.0",
        "python-json-logger==3.2.1",
        "python-multipart==0.0.20",
        "PyYAML==6.0.2",
        "pyzmq==26.2.1",
        "referencing==0.36.2",
        "requests==2.32.3",
        "rfc3339-validator==0.1.4",
        "rfc3986-validator==0.1.1",
        "rpds-py==0.23.1",
        "rsa==4.9",
        "ruff==0.9.7",
        "Send2Trash==1.8.3",
        "six==1.17.0",
        "sniffio==1.3.1",
        "soupsieve==2.6",
        "stack-data==0.6.3",
        "starlette==0.45.3",
        "terminado==0.18.1",
        "tinycss2==1.2.1",
        "tomli==2.2.1",
        "tornado==6.4.2",
        "traitlets==5.14.3",
        "types-python-dateutil==2.9.0.20241206",
        "typing_extensions==4.12.2",
        "update-checker==0.18.0",
        "uri-template==1.3.0",
        "urllib3==2.3.0",
        "uvicorn==0.34.0",
        "uvloop==0.21.0",
        "watchfiles==1.0.4",
        "wcwidth==0.2.13",
        "webcolors==24.11.1",
        "webencodings==0.5.1",
        "websocket-client==1.8.0",
        "websockets==15.0",
        "yt-dlp==2025.2.19",
        "zipp==3.21.0",
    ],
    entry_points={
        "console_scripts": [
            "filepi=src.main:run_server",
            "filepi-install-service=src.service:install_service",
        ],
    },
)
