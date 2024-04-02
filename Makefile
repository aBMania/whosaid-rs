.PHONY: tests

GIT_TAG := $(shell git describe --tags --candidates 1)
BIN_DIR = "bin"

X86_64_TAG = "x86_64-unknown-linux-gnu"
BUILD_PATH_X86_64 = "target/$(X86_64_TAG)/release"
AARCH64_TAG = "aarch64-unknown-linux-gnu"
BUILD_PATH_AARCH64 = "target/$(AARCH64_TAG)/release"

PINNED_NIGHTLY ?= nightly
CLIPPY_PINNED_NIGHTLY=nightly-2022-05-19

# List of features to use when cross-compiling. Can be overridden via the environment.
CROSS_FEATURES ?=

# Cargo profile for Cross builds. Default is for local builds, CI uses an override.
CROSS_PROFILE ?= release

# List of features to use when running CI tests.
TEST_FEATURES ?=

# Cargo profile for regular builds.
PROFILE ?= release

# Extra flags for Cargo
CARGO_INSTALL_EXTRA_FLAGS?=

# Builds the Lighthouse binary in release (optimized).
#
# Binaries will most likely be found in `./target/release`
install:
	cargo install --path whosaid --force --locked \
		--features "$(FEATURES)" \
		--profile "$(PROFILE)" \
		$(CARGO_INSTALL_EXTRA_FLAGS)

# The following commands use `cross` to build a cross-compile.
#
# These commands require that:
#
# - `cross` is installed (`cargo install cross`).
# - Docker is running.
# - The current user is in the `docker` group.
#
# The resulting binaries will be created in the `target/` directory.
#
# The *-portable options compile the blst library *without* the use of some
# optimized CPU functions that may not be available on some systems. This
# results in a more portable binary with ~20% slower BLS verification.
build-x86_64:
	cross build --bin whosaid --target x86_64-unknown-linux-gnu --features "modern,$(CROSS_FEATURES)" --profile "$(CROSS_PROFILE)" --locked
build-x86_64-portable:
	cross build --bin whosaid --target x86_64-unknown-linux-gnu --features "portable,$(CROSS_FEATURES)" --profile "$(CROSS_PROFILE)" --locked
build-aarch64:
	cross build --bin whosaid --target aarch64-unknown-linux-gnu --features "$(CROSS_FEATURES)" --profile "$(CROSS_PROFILE)" --locked
build-aarch64-portable:
	cross build --bin whosaid --target aarch64-unknown-linux-gnu --features "portable,$(CROSS_FEATURES)" --profile "$(CROSS_PROFILE)" --locked

# Create a `.tar.gz` containing a binary for a specific target.
define tarball_release_binary
	cp $(1)/whosaid $(BIN_DIR)/whosaid
	cd $(BIN_DIR) && \
		tar -czf whosaid-$(GIT_TAG)-$(2)$(3).tar.gz whosaid && \
		rm whosaid
endef

# Create a series of `.tar.gz` files in the BIN_DIR directory, each containing
# a `whosaid` binary for a different target.
#
# The current git tag will be used as the version in the output file names. You
# will likely need to use `git tag` and create a semver tag (e.g., `v0.2.3`).
build-release-tarballs:
	[ -d $(BIN_DIR) ] || mkdir -p $(BIN_DIR)
	$(MAKE) build-x86_64
	$(call tarball_release_binary,$(BUILD_PATH_X86_64),$(X86_64_TAG),"")
	$(MAKE) build-x86_64-portable
	$(call tarball_release_binary,$(BUILD_PATH_X86_64),$(X86_64_TAG),"-portable")
	$(MAKE) build-aarch64
	$(call tarball_release_binary,$(BUILD_PATH_AARCH64),$(AARCH64_TAG),"")
	$(MAKE) build-aarch64-portable
	$(call tarball_release_binary,$(BUILD_PATH_AARCH64),$(AARCH64_TAG),"-portable")

# Runs the full workspace tests in **release**, without downloading any additional
# test vectors.
test-release:
	cargo test --workspace --release --features "$(TEST_FEATURES)" \
 		--exclude ef_tests --exclude beacon_chain --exclude slasher --exclude network

# Runs the full workspace tests in **release**, without downloading any additional
# test vectors, using nextest.
nextest-release:
	cargo nextest run --workspace --release --features "$(TEST_FEATURES)" \
		--exclude ef_tests --exclude beacon_chain --exclude slasher --exclude network

# Runs the full workspace tests in **debug**, without downloading any additional test
# vectors.
test-debug:
	cargo test --workspace --features "$(TEST_FEATURES)" \
		--exclude ef_tests --exclude beacon_chain --exclude network

# Runs the full workspace tests in **debug**, without downloading any additional test
# vectors, using nextest.
nextest-debug:
	cargo nextest run --workspace --features "$(TEST_FEATURES)" \
		--exclude ef_tests --exclude beacon_chain --exclude network

# Runs cargo-fmt (linter).
cargo-fmt:
	cargo fmt --all -- --check

# Typechecks benchmark code
check-benches:
	cargo check --workspace --benches --features "$(TEST_FEATURES)"

# Runs the full workspace tests in release, without downloading any additional
# test vectors.
test: test-release

# Runs the entire test suite
test-full: cargo-fmt test-release test-debug

# Lints the code for bad style and potentially unsafe arithmetic using Clippy.
# Clippy lints are opt-in per-crate for now. By default, everything is allowed except for performance and correctness lints.
lint:
	cargo clippy --workspace --tests $(EXTRA_CLIPPY_OPTS) --features "$(TEST_FEATURES)" -- \
		-D clippy::fn_to_numeric_cast_any \
		-D clippy::manual_let_else \
		-D warnings \
		-A clippy::derive_partial_eq_without_eq \
		-A clippy::from-over-into \
		-A clippy::upper-case-acronyms \
		-A clippy::vec-init-then-push \
		-A clippy::question-mark \
		-A clippy::uninlined-format-args \
		-A clippy::enum_variant_names

# Lints the code using Clippy and automatically fix some simple compiler warnings.
lint-fix:
	EXTRA_CLIPPY_OPTS="--fix --allow-staged --allow-dirty" $(MAKE) lint

nightly-lint:
	cp .github/custom/clippy.toml .
	cargo +$(CLIPPY_PINNED_NIGHTLY) clippy --workspace --tests --release -- \
		-A clippy::all \
		-D clippy::disallowed_from_async
	rm clippy.toml

# Runs cargo audit (Audit Cargo.lock files for crates with security vulnerabilities reported to the RustSec Advisory Database)
audit: install-audit audit-CI

install-audit:
	cargo install --force cargo-audit

audit-CI:
	cargo audit

# Runs `cargo vendor` to make sure dependencies can be vendored for packaging, reproducibility and archival purpose.
vendor:
	cargo vendor

# Runs `cargo udeps` to check for unused dependencies
udeps:
	cargo +$(PINNED_NIGHTLY) udeps --tests --all-targets --release --features "$(TEST_FEATURES)"
