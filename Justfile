mod? local

# By default we use the nightly toolchain, however you can override this by setting the RUST_TOOLCHAIN environment variable.
export RUST_TOOLCHAIN := env_var_or_default('RUST_TOOLCHAIN', 'nightly')

# An alias for cargo xtask check
powerset *args:
    cargo +{{RUST_TOOLCHAIN}} xtask powerset {{args}}

# An alias for cargo fmt --all
fmt *args:
    cargo +{{RUST_TOOLCHAIN}} fmt --all {{args}}

lint *args:
    cargo +{{RUST_TOOLCHAIN}} clippy --fix --allow-dirty --all-targets --allow-staged {{args}}

test *args:
    #!/bin/bash
    set -euo pipefail

    INSTA_FORCE_PASS=1 cargo +{{RUST_TOOLCHAIN}} llvm-cov clean --workspace
    INSTA_FORCE_PASS=1 cargo +{{RUST_TOOLCHAIN}} llvm-cov nextest --include-build-script --no-report -- {{args}}
    # Coverage for doctests is currently broken in llvm-cov.
    # Once it fully works we can add the `--doctests` flag to the test and report command again.
    cargo +{{RUST_TOOLCHAIN}} llvm-cov test --doc --no-report -- {{args}}

    # Do not generate the coverage report on CI
    cargo insta review
    cargo +{{RUST_TOOLCHAIN}} llvm-cov report --lcov --output-path ./lcov.info
    cargo +{{RUST_TOOLCHAIN}} llvm-cov report --html

doc *args:
    cargo +{{RUST_TOOLCHAIN}} doc --no-deps --all-features {{args}}

deny *args:
    cargo +{{RUST_TOOLCHAIN}} deny {{args}} --all-features check

workspace-hack:
    cargo +{{RUST_TOOLCHAIN}} hakari manage-deps
    cargo +{{RUST_TOOLCHAIN}} hakari generate
