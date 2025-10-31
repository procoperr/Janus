.PHONY: help format lint test bench build release clean check install docs setup

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
NC := \033[0m # No Color

help: ## Show this help message
	@echo "$(BLUE)Janus - Makefile Commands$(NC)"
	@echo "================================"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}'
	@echo ""

setup: ## Setup development environment
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@./scripts/setup-dev.sh

format: ## Format code with rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	@cargo fmt --all
	@echo "$(GREEN)✓ Formatting complete$(NC)"

format-check: ## Check code formatting without modifying files
	@echo "$(BLUE)Checking code formatting...$(NC)"
	@cargo fmt --all -- --check

lint: ## Run clippy lints
	@echo "$(BLUE)Running clippy...$(NC)"
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✓ Linting complete$(NC)"

test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	@cargo test --all-features
	@echo "$(GREEN)✓ Tests complete$(NC)"

test-verbose: ## Run tests with verbose output
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	@cargo test --all-features -- --nocapture --test-threads=1

test-unit: ## Run only unit tests
	@echo "$(BLUE)Running unit tests...$(NC)"
	@cargo test --lib --all-features

test-integration: ## Run only integration tests
	@echo "$(BLUE)Running integration tests...$(NC)"
	@cargo test --test '*' --all-features

bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	@cargo bench
	@echo "$(GREEN)✓ Benchmarks complete$(NC)"
	@echo ""
	@echo "$(YELLOW)View HTML report: open target/criterion/report/index.html$(NC)"

bench-baseline: ## Create benchmark baseline
	@echo "$(BLUE)Creating benchmark baseline...$(NC)"
	@cargo bench -- --save-baseline main

bench-compare: ## Compare benchmarks against baseline
	@echo "$(BLUE)Comparing benchmarks against baseline...$(NC)"
	@cargo bench -- --baseline main

build: ## Build in debug mode
	@echo "$(BLUE)Building (debug)...$(NC)"
	@cargo build
	@echo "$(GREEN)✓ Build complete$(NC)"

build-release: ## Build in release mode
	@echo "$(BLUE)Building (release)...$(NC)"
	@cargo build --release
	@echo "$(GREEN)✓ Release build complete$(NC)"

release-local: ## Build release artifacts for current platform
	@echo "$(BLUE)Building release artifacts...$(NC)"
	@./scripts/build-release.sh
	@echo "$(GREEN)✓ Release artifacts created$(NC)"

release-all: ## Build release artifacts for all platforms
	@echo "$(BLUE)Building release artifacts for all platforms...$(NC)"
	@./scripts/build-release.sh --all
	@echo "$(GREEN)✓ All release artifacts created$(NC)"

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf release-artifacts
	@echo "$(GREEN)✓ Clean complete$(NC)"

check: format-check lint test ## Run all checks (CI equivalent)
	@echo ""
	@echo "$(GREEN)✓ All checks passed!$(NC)"

check-quick: format-check lint ## Run quick checks (no tests)
	@echo ""
	@echo "$(GREEN)✓ Quick checks passed!$(NC)"

install: ## Install janus binary locally
	@echo "$(BLUE)Installing janus...$(NC)"
	@cargo install --path . --locked
	@echo "$(GREEN)✓ Installation complete$(NC)"

install-release: ## Install janus binary (release mode)
	@echo "$(BLUE)Installing janus (release)...$(NC)"
	@cargo install --path . --locked --release
	@echo "$(GREEN)✓ Installation complete$(NC)"

docs: ## Generate and open documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	@cargo doc --no-deps --all-features --open
	@echo "$(GREEN)✓ Documentation generated$(NC)"

docs-private: ## Generate documentation including private items
	@echo "$(BLUE)Generating documentation (including private)...$(NC)"
	@cargo doc --no-deps --all-features --document-private-items --open

update: ## Update dependencies
	@echo "$(BLUE)Updating dependencies...$(NC)"
	@cargo update
	@echo "$(GREEN)✓ Dependencies updated$(NC)"

audit: ## Run security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	@cargo audit
	@echo "$(GREEN)✓ Audit complete$(NC)"

watch-test: ## Watch for changes and run tests
	@echo "$(BLUE)Watching for changes and running tests...$(NC)"
	@cargo watch -x test

watch-check: ## Watch for changes and run checks
	@echo "$(BLUE)Watching for changes and running checks...$(NC)"
	@cargo watch -x 'clippy --all-targets --all-features -- -D warnings'

bloat: ## Analyze binary size
	@echo "$(BLUE)Analyzing binary size...$(NC)"
	@cargo bloat --release
	@echo ""
	@echo "$(YELLOW)Detailed analysis: cargo bloat --release -n 50$(NC)"

profile-build: ## Profile build time
	@echo "$(BLUE)Profiling build time...$(NC)"
	@cargo clean
	@cargo build --timings
	@echo "$(GREEN)✓ Build profiling complete$(NC)"
	@echo ""
	@echo "$(YELLOW)View report: open target/cargo-timings/cargo-timing.html$(NC)"

coverage: ## Generate test coverage report (requires cargo-tarpaulin)
	@echo "$(BLUE)Generating coverage report...$(NC)"
	@cargo tarpaulin --out Html --output-dir coverage
	@echo "$(GREEN)✓ Coverage report generated$(NC)"
	@echo ""
	@echo "$(YELLOW)View report: open coverage/index.html$(NC)"

tree: ## Show dependency tree
	@echo "$(BLUE)Dependency tree:$(NC)"
	@cargo tree

features: ## List all features
	@echo "$(BLUE)Available features:$(NC)"
	@grep -A 10 '\[features\]' Cargo.toml | grep -v '\[features\]'

version: ## Show current version
	@echo "$(BLUE)Current version:$(NC)"
	@grep '^version = ' Cargo.toml | head -n 1 | cut -d '"' -f 2

example-sync: ## Run example sync operation
	@echo "$(BLUE)Running example sync...$(NC)"
	@./examples/basic_sync.sh

ci: check bench-baseline ## Run full CI checks locally
	@echo ""
	@echo "$(GREEN)✓ Full CI checks passed!$(NC)"

pre-commit: format lint test-unit ## Quick pre-commit checks
	@echo ""
	@echo "$(GREEN)✓ Pre-commit checks passed!$(NC)"

pre-push: check test ## Checks before pushing
	@echo ""
	@echo "$(GREEN)✓ Pre-push checks passed!$(NC)"
