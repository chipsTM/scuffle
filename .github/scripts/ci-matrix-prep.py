import sys
import os
import json
import subprocess
from typing import Optional
from dataclasses import dataclass, asdict

# The first argument is the github context
GITHUB_CONTEXT = json.loads(sys.argv[1])

GITHUB_DEFAULT_RUNNER = "ubuntu-24.04"
LINUX_X86_64 = "ubicloud-standard-8"
LINUX_ARM64 = "ubicloud-standard-8-arm"

def is_brawl(mode: Optional[str] = None) -> bool:
    if mode is None:
        mode = ""
    else:
        mode = f"{mode}/"

    return GITHUB_CONTEXT["event_name"] == "push" and GITHUB_CONTEXT["ref"].startswith(
        f"refs/heads/brawl/{mode}"
    )


def is_pr() -> bool:
    return GITHUB_CONTEXT["event_name"] == "pull_request"


def pr_number() -> Optional[int]:
    if is_pr():
        return GITHUB_CONTEXT["event"]["number"]
    elif is_brawl("try"):
        return int(GITHUB_CONTEXT["ref"].strip("refs/heads/brawl/try/"))

    return None

# The output should be in the form
# matrix=<json>

@dataclass
class RustSetup:
    toolchain: str
    shared_key: Optional[str]
    components: str = ""
    tools: str = ""
    cache_backend: str = "ubicloud"

@dataclass
class DocsMatrix:
    os: str
    artifact_name: Optional[str]
    deploy_docs: bool
    job_name: str
    pr_number: Optional[int]
    rust: RustSetup

@dataclass
class ClippyMatrix:
    os: str
    powerset: bool
    job_name: str
    rust: RustSetup

@dataclass
class TestMatrix:
    os: str
    job_name: str
    pr_number: Optional[int]
    commit_sha: str
    rust: RustSetup

@dataclass
class GrindMatrix:
    os: str
    job_name: str
    env: str
    rust: RustSetup

@dataclass
class FmtMatrix:
    os: str
    job_name: str
    rust: RustSetup

@dataclass
class HakariMatrix:
    os: str
    job_name: str
    rust: RustSetup

@dataclass
class Job:
    job_name: str
    inputs: GrindMatrix | DocsMatrix | ClippyMatrix | TestMatrix | FmtMatrix | HakariMatrix

def create_docs_jobs() -> list[DocsMatrix]:
    jobs: list[DocsMatrix] = []

    jobs.append(
        DocsMatrix(
            os=LINUX_X86_64,
            artifact_name="docs",
            # if its brawl merge, we don't want to deploy docs
            # since that will be deployed after the merge is successful
            deploy_docs=not is_brawl("merge"),
            job_name=f"Docs (Linux x86_64)",
            pr_number=pr_number(),
            rust=RustSetup(
                toolchain="nightly",
                components="rust-docs",
                shared_key="docs-linux-x86_64",
                tools="",
                cache_backend="ubicloud",
            ),
        )
    )

    if is_brawl():
        jobs.append(
            DocsMatrix(
                os=LINUX_ARM64,
                artifact_name=None,
                deploy_docs=False,
                job_name=f"Docs (Linux arm64)",
                pr_number=pr_number(),
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-docs",
                    shared_key="docs-linux-arm64",
                    tools="",
                    cache_backend="ubicloud",
                ),
            )
        )

    return jobs


def create_clippy_jobs() -> list[ClippyMatrix]:
    jobs: list[ClippyMatrix] = []

    jobs.append(
        ClippyMatrix(
            os=LINUX_X86_64,
            powerset=is_brawl(),
            job_name=f"Clippy (Linux x86_64)",
            rust=RustSetup(
                toolchain="nightly",
                components="rust-clippy",
                shared_key="clippy-linux-x86_64",
                tools="cargo-nextest,cargo-llvm-cov",
                cache_backend="ubicloud",
            ),
        )
    )

    if is_brawl():
        jobs.append(
            ClippyMatrix(
                os=LINUX_ARM64,
                powerset=True,
                job_name=f"Clippy (Linux arm64)",
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-clippy",
                    shared_key="clippy-linux-arm64",
                    tools="cargo-nextest,cargo-llvm-cov",
                    cache_backend="ubicloud",
                ),
            )
        )

    return jobs


def create_test_jobs() -> list[TestMatrix]:
    jobs: list[TestMatrix] = []

    commit_sha = os.environ["SHA"]
    if is_brawl("try"):
        commit_sha = (
            subprocess.check_output(["git", "log", "-n", "1", "--pretty=format:%H"])
            .decode()
            .strip()
        )

    jobs.append(
        TestMatrix(
            os=LINUX_X86_64,
            job_name=f"Test (Linux x86_64)",
            pr_number=pr_number(),
            commit_sha=commit_sha,
            rust=RustSetup(
                toolchain="nightly",
                components="llvm-tools-preview",
                shared_key="test-linux-x86_64",
                tools="cargo-nextest,cargo-llvm-cov",
                cache_backend="ubicloud",
            ),
        )
    )

    if is_brawl():
        jobs.append(
            TestMatrix(
                os=LINUX_ARM64,
                job_name=f"Test (Linux arm64)",
                pr_number=pr_number(),
                commit_sha=commit_sha,
                rust=RustSetup(
                    toolchain="nightly",
                    components="llvm-tools-preview",
                    shared_key="test-linux-arm64",
                    tools="cargo-nextest,cargo-llvm-cov",
                    cache_backend="ubicloud",
                ),
            )
        )

    return jobs


def create_grind_jobs() -> list[GrindMatrix]:
    jobs: list[GrindMatrix] = []

    if is_brawl():
        jobs.append(
            GrindMatrix(
                os=LINUX_X86_64,
                job_name=f"Grind (Linux x86_64)",
                env="X86_64_UNKNOWN_LINUX_GNU=valgrind --error-exitcode=1 --leak-check=full --gen-suppressions=all --suppressions=$(pwd)/valgrind_suppressions.log",
                rust=RustSetup(
                    toolchain="nightly",
                    shared_key="grind-linux-x86_64",
                    tools="cargo-nextest",
                    cache_backend="ubicloud",
                ),
            )
        )

        jobs.append(
            GrindMatrix(
                os=LINUX_ARM64,
                job_name=f"Grind (Linux arm64)",
                env="AARCH64_UNKNOWN_LINUX_GNU=valgrind --error-exitcode=1 --leak-check=full --gen-suppressions=all --suppressions=$(pwd)/valgrind_suppressions.log",
                rust=RustSetup(
                    toolchain="nightly",
                    shared_key="grind-linux-arm64",
                    tools="cargo-nextest",
                    cache_backend="ubicloud",
                ),
            )
        )

    return jobs

def create_fmt_jobs() -> list[FmtMatrix]:
    jobs: list[FmtMatrix] = []

    jobs.append(
        FmtMatrix(
            os=GITHUB_DEFAULT_RUNNER,
            job_name=f"Fmt",
            rust=RustSetup(
                toolchain="nightly",
                components="rustfmt",
                shared_key=None,
                cache_backend="github",
            ),
        )
    )

    return jobs

def create_hakari_jobs() -> list[HakariMatrix]:
    jobs: list[HakariMatrix] = []

    jobs.append(
        HakariMatrix(
            os=GITHUB_DEFAULT_RUNNER,
            job_name=f"Hakari",
            rust=RustSetup(
                toolchain="nightly",
                components="rustfmt",
                tools="cargo-hakari",
                shared_key=None,
                cache_backend="github",
            ),
        )
    )

    return jobs

def create_jobs() -> list[Job]:
    jobs: list[Job] = []
    jobs.extend([Job(job_name="docs", inputs=job) for job in create_docs_jobs()])
    jobs.extend([Job(job_name="clippy", inputs=job) for job in create_clippy_jobs()])
    jobs.extend([Job(job_name="test", inputs=job) for job in create_test_jobs()])
    jobs.extend([Job(job_name="grind", inputs=job) for job in create_grind_jobs()])
    jobs.extend([Job(job_name="fmt", inputs=job) for job in create_fmt_jobs()])
    jobs.extend([Job(job_name="hakari", inputs=job) for job in create_hakari_jobs()])

    return jobs


def main():
    print(f"matrix={json.dumps([asdict(job) for job in create_jobs()])}")

if __name__ == "__main__":
    main()
