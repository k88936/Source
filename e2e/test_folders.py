#!/usr/bin/env -S uv run --quiet --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["httpx>=0.27"]
# ///
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from typing import Any

import httpx

REPOSITORY = Path(__file__).resolve().parent.parent
IMAGE = os.environ.get(
    "CLOUDREVE_E2E_IMAGE", "docker.io/cloudreve/cloudreve:4.19.1"
)
CONTAINER = f"source-e2e-folders-{os.getpid()}"
CONTAINER_PORT = 5212
EMAIL = "e2e-admin@example.test"
PASSWORD = "e2e-password-change-me"

ROOT_URI = "cloudreve://my"
A0_URI = "cloudreve://my/a0"
SEED_FOLDERS = (A0_URI, f"{A0_URI}/b0", "cloudreve://my/a1")


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
) -> Any:
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


def remove_container() -> None:
    subprocess.run(
        ("podman", "rm", "--force", CONTAINER),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )


def stop_on_signal(_signum: int, _frame: object) -> None:
    raise KeyboardInterrupt


def container_endpoint() -> str:
    published = run(
        "podman", "port", CONTAINER, f"{CONTAINER_PORT}/tcp", capture_output=True
    ).stdout.strip()
    if not published:
        raise RuntimeError(f"container {CONTAINER} publishes no port")
    # "127.0.0.1:41234" (one line per address; the first one is the IPv4 mapping).
    port = published.splitlines()[0].rsplit(":", 1)[-1]
    return f"http://127.0.0.1:{port}"


def start_container() -> str:
    if image_available(IMAGE):
        log(f"Reusing the local Cloudreve image {IMAGE}")
    else:
        log(f"Pulling the Cloudreve image {IMAGE}")
        run("podman", "pull", IMAGE)

    log(f"Starting ephemeral Cloudreve container from {IMAGE}")
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
        IMAGE,
        capture_output=True,
    )

    inspection = json.loads(
        run("podman", "inspect", CONTAINER, capture_output=True).stdout
    )[0]
    mounts = inspection.get("Mounts", [])
    if mounts:
        raise RuntimeError(f"ephemeral container unexpectedly has mounts: {mounts!r}")
    log("Verified that the container has no mounts")

    endpoint = container_endpoint()
    log(f"Waiting for Cloudreve at {endpoint}")
    wait_until_ready(endpoint)
    return endpoint


def ensure_admin(client: httpx.Client) -> str:
    api(
        client,
        "POST",
        "/api/v4/user",
        body={"email": EMAIL, "password": PASSWORD, "language": "en-US"},
    )
    log(f"Registered the first user {EMAIL} as administrator")
    data = api(
        client,
        "POST",
        "/api/v4/session/token",
        body={"email": EMAIL, "password": PASSWORD},
    )
    return data["token"]["access_token"]


def create_folder(client: httpx.Client, token: str, uri: str) -> None:
    api(
        client,
        "POST",
        "/api/v4/file/create",
        token=token,
        body={"uri": uri, "type": "folder", "err_on_conflict": False},
    )


def seed(client: httpx.Client, token: str) -> None:
    for uri in SEED_FOLDERS:
        create_folder(client, token, uri)
    log(f"Seeded {', '.join(SEED_FOLDERS)}")


def list_page(server: str, catalog: Path, uri: str) -> dict[str, Any]:
    """One `list_remote_folders` call, through the example that drives it."""
    environment = os.environ.copy()
    environment.update(
        {
            "CLOUDREVE_SERVER": server,
            "CLOUDREVE_EMAIL": EMAIL,
            "CLOUDREVE_PASSWORD": PASSWORD,
            "SOURCE_E2E_CATALOG": str(catalog),
        }
    )
    result = subprocess.run(
        (
            "cargo",
            "run",
            "--quiet",
            "-p",
            "source-core",
            "--example",
            "list_remote_folders",
            "--",
            uri,
        ),
        cwd=REPOSITORY,
        env=environment,
        check=True,
        text=True,
        capture_output=True,
    )
    return json.loads(result.stdout)


def folder_names(page: dict[str, Any]) -> list[str]:
    return sorted(folder["name"] for folder in page["folders"])


def main() -> None:
    for command in ("podman", "cargo"):
        if shutil.which(command) is None:
            raise RuntimeError(f"required command not found: {command}")

    server = start_container()

    with http_client(server) as client:
        token = ensure_admin(client)
        seed(client, token)

    with tempfile.TemporaryDirectory(prefix="source-e2e-folders-") as directory:
        catalog = Path(directory) / "catalog.sqlite3"

        log("Listing the drive root")
        root = list_page(server, catalog, ROOT_URI)
        assert root["current"]["name"] == "", f"root name: {root!r}"
        assert root["current"]["uri"] == ROOT_URI, f"root uri: {root!r}"
        assert root["current"]["path"] == "/", f"root path: {root!r}"
        assert folder_names(root) == ["a0", "a1"], f"root folders: {root!r}"
        assert sorted(folder["path"] for folder in root["folders"]) == [
            "/a0",
            "/a1",
        ], f"root folder paths: {root!r}"
        assert all(folder["id"] for folder in root["folders"]), f"root ids: {root!r}"
        assert root["next"] is None, f"root next: {root!r}"

        log("Listing cloudreve://my/a0")
        a0 = list_page(server, catalog, A0_URI)
        assert a0["current"]["name"] == "a0", f"a0 name: {a0!r}"
        assert a0["current"]["uri"] == A0_URI, f"a0 uri: {a0!r}"
        assert a0["current"]["path"] == "/a0", f"a0 path: {a0!r}"
        assert folder_names(a0) == ["b0"], f"a0 folders: {a0!r}"
        assert a0["folders"][0]["path"] == "/a0/b0", f"a0 folder path: {a0!r}"
        assert a0["next"] is None, f"a0 next: {a0!r}"

    log("E2E test passed")


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, stop_on_signal)
    failed = False
    try:
        main()
    except BaseException:
        failed = True
        raise
    finally:
        if failed:
            subprocess.run(
                ("podman", "logs", CONTAINER),
                stdout=sys.stderr,
                stderr=subprocess.DEVNULL,
                check=False,
                text=True,
            )
        remove_container()
