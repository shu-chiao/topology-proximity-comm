"""Docker helpers to build and quick-run the three topology-proximity-comm samples."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

SAMPLE1_COMPOSE = REPO_ROOT / "samples/01-traditional-dds/docker-compose.yml"
SAMPLE2_COMPOSE = REPO_ROOT / "samples/02-rmw-zenoh/docker-compose.yml"
SAMPLE3_COMPOSE = REPO_ROOT / "samples/03-dds-zenoh-bridge/docker-compose.yml"


def _which(name: str) -> str | None:
    return shutil.which(name)


def check_prerequisites() -> dict[str, bool | str | None]:
    """Return a quick readiness map for the Docker notebook workflow."""
    return {
        "repo_root": str(REPO_ROOT),
        "docker": _which("docker"),
        "docker_compose": _which("docker-compose") or (
            "docker compose" if _which("docker") else None
        ),
        "sample1_compose": str(SAMPLE1_COMPOSE),
        "sample2_compose": str(SAMPLE2_COMPOSE),
        "sample3_compose": str(SAMPLE3_COMPOSE),
    }


def run_shell(
    cmd: str,
    *,
    cwd: Path | None = None,
    check: bool = True,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "-lc", cmd],
        cwd=cwd or REPO_ROOT,
        text=True,
        capture_output=True,
        check=check,
        timeout=timeout,
    )


def build_sample1_docker() -> str:
    """Build the Jazzy Docker image for sample 1."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    if not SAMPLE1_COMPOSE.is_file():
        raise FileNotFoundError(SAMPLE1_COMPOSE)
    result = run_shell(
        f"docker compose -f {SAMPLE1_COMPOSE} build",
        check=False,
        timeout=600,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def run_sample1_docker_demo(*, duration_sec: float = 6.0, ros_domain_id: int = 42) -> str:
    """Run sample 1 talker+listener inside the Jazzy container."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    result = run_shell(
        f"docker compose -f {SAMPLE1_COMPOSE} run --rm "
        f"-e ROS_DOMAIN_ID={ros_domain_id} "
        f"-e DEMO_DURATION_SEC={duration_sec} demo",
        check=False,
        timeout=duration_sec + 120,
    )
    output = (result.stdout or "") + (result.stderr or "")
    if result.returncode not in (0, 124):
        raise RuntimeError(output or f"docker demo failed with code {result.returncode}")
    return output


def build_sample2_docker() -> str:
    """Build the Jazzy Docker image for sample 2 (rmw_zenoh_cpp)."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    if not SAMPLE2_COMPOSE.is_file():
        raise FileNotFoundError(SAMPLE2_COMPOSE)
    result = run_shell(
        f"docker compose -f {SAMPLE2_COMPOSE} build",
        check=False,
        timeout=600,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def run_sample2_docker_demo(*, duration_sec: float = 6.0, ros_domain_id: int = 42) -> str:
    """Run sample 2 rmw_zenohd + talker + listener inside the Jazzy container."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    result = run_shell(
        f"docker compose -f {SAMPLE2_COMPOSE} run --rm "
        f"-e ROS_DOMAIN_ID={ros_domain_id} "
        f"-e DEMO_DURATION_SEC={duration_sec} demo",
        check=False,
        timeout=duration_sec + 120,
    )
    output = (result.stdout or "") + (result.stderr or "")
    if result.returncode not in (0, 124):
        raise RuntimeError(output or f"docker demo failed with code {result.returncode}")
    return output


def build_sample3_docker() -> str:
    """Build the two-container Docker images for sample 3."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    if not SAMPLE3_COMPOSE.is_file():
        raise FileNotFoundError(SAMPLE3_COMPOSE)
    result = run_shell(
        f"docker compose -f {SAMPLE3_COMPOSE} build",
        check=False,
        timeout=900,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def run_sample3_docker_demo(*, duration_sec: float = 8.0, ros_domain_id: int = 42) -> str:
    """Run sample 3 ros2 + rust containers (two-container bridge demo)."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    wait_ros = int(min(4, max(2, duration_sec * 0.5)))
    result = run_shell(
        f"ROS_DOMAIN_ID={ros_domain_id} "
        f"DEMO_DURATION_SEC={duration_sec} "
        f"RUST_WAIT_ROS_SEC={wait_ros} "
        f"docker compose -f {SAMPLE3_COMPOSE} up --build --abort-on-container-exit",
        check=False,
        timeout=duration_sec + 300,
    )
    output = (result.stdout or "") + (result.stderr or "")
    run_shell(f"docker compose -f {SAMPLE3_COMPOSE} down", check=False)
    if result.returncode != 0:
        raise RuntimeError(output or f"docker demo failed with code {result.returncode}")
    return output
