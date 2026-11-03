#!/usr/bin/env -S uv run --quiet --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["httpx>=0.27", "pillow>=10.4"]
# ///
"""Provision a throwaway Cloudreve instance for local development.

Starts (or reuses) a podman container whose data lives in `tmp/cloudreve-data`,
registers the dev administrator, uploads a handful of generated example images
to `cloudreve://my/example-dir`, and writes `tmp/cloudreve.json` so a dev build
of the desktop app can log in without any manual setup.

The script stays in the foreground: the container lives exactly as long as it
does, so `Ctrl-C` (or SIGTERM) removes it and nothing is left running behind
your back. `--clean` additionally drops `tmp/cloudreve-data` before starting.
"""

from __future__ import annotations

import argparse
import json
import math
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

import httpx
from PIL import Image

REPOSITORY = Path(__file__).resolve().parent.parent
TMP = REPOSITORY / "tmp"
DATA_DIR = TMP / "cloudreve-data"
CONFIG_PATH = TMP / "cloudreve.json"
EXAMPLE_DIR = TMP / "example" / "example-dir"

CONTAINER = "source-dev-cloudreve"
DEFAULT_IMAGE = "docker.io/cloudreve/cloudreve:4.19.1"
CONTAINER_DATA = "/cloudreve/data"
CONTAINER_PORT = 5212

EMAIL = "dev@source.local"
PASSWORD = "source-dev"
LIBRARY_ROOT = "cloudreve://my/"
REMOTE_DIR = "cloudreve://my/example-dir"

IMAGE_COUNT = 6
IMAGE_SIZE = 64

# Set once this process adopts or creates the container; from then on it owns the
# container and has to take it down on the way out.
MANAGING = False


def run(*command: str, capture_output: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=REPOSITORY,
        check=True,
        text=True,
        capture_output=capture_output,
    )


def log(message: str) -> None:
    print(message, flush=True)


class ApiError(RuntimeError):
    """A Cloudreve reply whose envelope carries a non-zero `code`."""

    def __init__(self, method: str, path: str, payload: dict[str, object]) -> None:
        super().__init__(f"{method} {path} failed: {payload!r}")
        self.code = payload.get("code")


def api(
        client: httpx.Client,
        method: str,
        path: str,
        *,
        token: str | None = None,
        body: object | None = None,
        content: bytes | None = None,
        headers: dict[str, str] | None = None,
) -> object:
    """Call a Cloudreve endpoint and return the payload inside the envelope."""
    request_headers = dict(headers or {})
    if token is not None:
        request_headers["Authorization"] = f"Bearer {token}"
    response = client.request(
        method,
        path,
        json=body,
        content=content,
        headers=request_headers,
        timeout=60,
    )
    response.raise_for_status()
    payload = response.json()
    if payload.get("code") != 0:
        raise ApiError(method, path, payload)
    return payload.get("data")


def http_client(endpoint: str, timeout: float = 60) -> httpx.Client:
    # The container is always local; ignore any proxy configured for the shell.
    return httpx.Client(base_url=endpoint, timeout=timeout, trust_env=False)


def wait_until_ready(endpoint: str) -> None:
    deadline = time.monotonic() + 120
    last_error: Exception | None = None
    with http_client(endpoint, timeout=5) as client:
        while time.monotonic() < deadline:
            try:
                version = api(client, "GET", "/api/v4/site/ping")
                log(f"Cloudreve {version} is ready at {endpoint}")
                return
            except (httpx.HTTPError, RuntimeError, ValueError) as error:
                last_error = error
                time.sleep(1)
    raise TimeoutError(f"Cloudreve did not become ready: {last_error}")


def image_available(image: str) -> bool:
    return (
            subprocess.run(
                ("podman", "image", "exists", image),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            ).returncode
            == 0
    )


def container_running() -> bool:
    result = subprocess.run(
        ("podman", "ps", "--filter", f"name=^{CONTAINER}$", "--format", "{{.Names}}"),
        text=True,
        capture_output=True,
        check=False,
    )
    return result.returncode == 0 and result.stdout.strip() == CONTAINER


def remove_container() -> None:
    subprocess.run(
        ("podman", "rm", "--force", CONTAINER),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )


def container_endpoint() -> str:
    published = run(
        "podman", "port", CONTAINER, f"{CONTAINER_PORT}/tcp", capture_output=True
    ).stdout.strip()
    if not published:
        raise RuntimeError(f"container {CONTAINER} publishes no port")
    # "127.0.0.1:41234" (one line per address; the first one is the IPv4 mapping).
    port = published.splitlines()[0].rsplit(":", 1)[-1]
    return f"http://127.0.0.1:{port}"


def start_container(image: str) -> str:
    global MANAGING
    MANAGING = True
    if container_running():
        endpoint = container_endpoint()
        try:
            wait_until_ready(endpoint)
            log(f"Reusing the running Cloudreve container {CONTAINER}")
            return endpoint
        except TimeoutError:
            log("The running container is unhealthy; recreating it")
    remove_container()

    DATA_DIR.mkdir(parents=True, exist_ok=True)
    if not image_available(image):
        log(f"Pulling the Cloudreve image {image}")
        run("podman", "pull", image)

    log(f"Starting Cloudreve from {image}")
    run(
        "podman",
        "run",
        "--detach",
        "--rm",
        "--pull",
        "never",
        "--image-volume",
        "ignore",
        "--name",
        CONTAINER,
        "--publish",
        f"127.0.0.1::{CONTAINER_PORT}",
        "--volume",
        f"{DATA_DIR}:{CONTAINER_DATA}:z",
        image,
        capture_output=True,
    )
    endpoint = container_endpoint()
    log(f"Waiting for Cloudreve at {endpoint}")
    wait_until_ready(endpoint)
    return endpoint


def ensure_admin(client: httpx.Client) -> str:
    try:
        api(
            client,
            "POST",
            "/api/v4/user",
            body={"email": EMAIL, "password": PASSWORD, "language": "zh-CN"},
        )
        log(f"Registered the first user {EMAIL} as administrator")
    except (httpx.HTTPError, ApiError) as error:
        log(f"Administrator {EMAIL} is already registered ({error}); logging in")
    data = api(
        client,
        "POST",
        "/api/v4/session/token",
        body={"email": EMAIL, "password": PASSWORD},
    )
    return data["token"]["access_token"]


def ensure_remote_dir(client: httpx.Client, token: str) -> None:
    api(
        client,
        "POST",
        "/api/v4/file/create",
        token=token,
        body={"uri": REMOTE_DIR, "type": "folder", "err_on_conflict": False},
    )


def example_images(count: int) -> list[Path]:
    EXAMPLE_DIR.mkdir(parents=True, exist_ok=True)
    images = []
    for index in range(1, count + 1):
        path = EXAMPLE_DIR / f"example-image-{index}.png"
        if not path.exists():
            shade = 40 + index * 30
            image = Image.new(
                "RGB", (IMAGE_SIZE, IMAGE_SIZE), (shade, 255 - shade, (index * 37) % 256)
            )
            image.save(path)
        images.append(path)
    return images


def upload_file(client: httpx.Client, token: str, path: Path) -> str:
    size = path.stat().st_size
    remote_uri = f"{REMOTE_DIR}/{path.name}"
    data = path.read_bytes()
    credential = api(
        client,
        "PUT",
        "/api/v4/file/upload",
        token=token,
        body={
            "uri": remote_uri,
            "size": size,
            "policy_id": "",
            "last_modified": int(path.stat().st_mtime * 1000),
            "mime_type": "image/png",
        },
    )

    session_id = credential["session_id"]
    chunk_size = int(credential["chunk_size"]) or size
    upload_urls = credential.get("upload_urls") or []
    chunks = max(1, math.ceil(size / chunk_size))
    for index in range(chunks):
        chunk = data[index * chunk_size: (index + 1) * chunk_size]
        url = upload_urls[index] if index < len(upload_urls) else None
        if url is None or url.startswith("/"):
            url = f"/api/v4/file/upload/{session_id}/{index}"
        if not url.startswith("/"):
            raise RuntimeError(f"remote storage policies are not supported here: {url}")
        api(
            client,
            "POST",
            url,
            token=token,
            content=chunk,
            headers={"Content-Type": "application/octet-stream"},
        )
    return remote_uri


def write_config(endpoint: str) -> None:
    TMP.mkdir(parents=True, exist_ok=True)
    CONFIG_PATH.write_text(
        json.dumps(
            {
                "version": 1,
                "endpoint": endpoint,
                "email": EMAIL,
                "password": PASSWORD,
                "uri": LIBRARY_ROOT,
            },
            indent=2,
        )
        + "\n"
    )
    log(f"Wrote {CONFIG_PATH}")


def clean() -> None:
    log(f"Removing the container {CONTAINER} and {DATA_DIR}")
    remove_container()
    shutil.rmtree(DATA_DIR, ignore_errors=True)


def wait_until_stopped() -> None:
    """Block for the container's lifetime; Ctrl-C or SIGTERM tears it down."""
    try:
        subprocess.run(
            ("podman", "wait", CONTAINER),
            stdout=subprocess.DEVNULL,
            check=False,
        )
    except KeyboardInterrupt:
        pass
    finally:
        log(f"Stopping the container {CONTAINER}")
        remove_container()


def stop_on_signal(_signum: int, _frame: object) -> None:
    raise KeyboardInterrupt


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--clean",
        action="store_true",
        help="remove the container and its data directory before starting",
    )
    parser.add_argument(
        "--image",
        default=DEFAULT_IMAGE,
        help=f"Cloudreve image to run (default: {DEFAULT_IMAGE})",
    )
    parser.add_argument(
        "--count",
        type=int,
        default=IMAGE_COUNT,
        help=f"how many example images to upload (default: {IMAGE_COUNT})",
    )
    args = parser.parse_args()

    for command in ("podman",):
        if shutil.which(command) is None:
            raise RuntimeError(f"required command not found: {command}")

    if args.clean:
        clean()

    endpoint = start_container(args.image)
    with http_client(endpoint) as client:
        token = ensure_admin(client)
        ensure_remote_dir(client, token)
        images = example_images(args.count)
        for path in images:
            try:
                log(f"Uploading {upload_file(client, token, path)}")
            except ApiError as error:
                # Re-running the script must not fail on files it already uploaded.
                if error.code != 40004:
                    raise
                log(f"{path.name} already exists; skipping")

    write_config(endpoint)
    log(f"Cloudreve is ready; press Ctrl-C to stop the container {CONTAINER}")
    wait_until_stopped()


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, stop_on_signal)
    failed = False
    try:
        main()
    except KeyboardInterrupt:
        # Ctrl-C/SIGTERM is how this script is meant to be stopped.
        pass
    except SystemExit as error:
        # `--help` and argument errors must not touch a running container.
        failed = bool(error.code)
        raise
    except BaseException:
        failed = True
        raise
    finally:
        if MANAGING and shutil.which("podman") is not None:
            if failed:
                subprocess.run(
                    ("podman", "logs", CONTAINER),
                    stdout=sys.stderr,
                    stderr=subprocess.DEVNULL,
                    check=False,
                    text=True,
                )
            remove_container()
