import sys
import os
import json
import subprocess
from typing import Optional
from dataclasses import dataclass

# The first argument is the github context
GITHUB_CONTEXT = json.loads(sys.argv[1])

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
    shared_key: str
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
    target: str
    valgrind: str
    job_name: str
    rust: RustSetup

@dataclass
class DocsResponse:
    docs: list[DocsMatrix]

@dataclass
class ClippyResponse:
    clippy: list[ClippyMatrix]

@dataclass
class TestResponse:
    test: list[TestMatrix]

@dataclass
class GrindResponse:
    grind: list[GrindMatrix]

@dataclass
class Response:
    docs: DocsResponse
    clippy: ClippyResponse
    test: TestResponse
    grind: GrindResponse


def create_docs_response() -> DocsResponse:
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

    return DocsResponse(docs=jobs)


def create_clippy_response() -> ClippyResponse:
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

    return ClippyResponse(clippy=jobs)


def create_test_response() -> TestResponse:
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

    return TestResponse(test=jobs)


def create_grind_response() -> GrindResponse:
    jobs: list[GrindMatrix] = []

    if is_brawl():
        jobs.append(
            GrindMatrix(
                os=LINUX_X86_64,
                job_name=f"Grind (Linux x86_64)",
                target="X86_64_UNKNOWN_LINUX_GNU",
                valgrind="valgrind",
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-valgrind",
                    shared_key="grind-linux-x86_64",
                    cache_backend="ubicloud",
                ),
            )
        )

        jobs.append(
            GrindMatrix(
                os=LINUX_ARM64,
                job_name=f"Grind (Linux arm64)",
                target="AARCH64_UNKNOWN_LINUX_GNU",
                valgrind="valgrind",
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-valgrind",
                    shared_key="grind-linux-arm64",
                    tools="",
                    cache_backend="ubicloud",
                ),
            )
        )

    return GrindResponse(grind=jobs)


def main():
    response = Response(
        docs=create_docs_response(),
        clippy=create_clippy_response(),
        test=create_test_response(),
        grind=create_grind_response(),
    )

    print(f"matrix={json.dumps(response)}")


if __name__ == "__main__":
    main()
