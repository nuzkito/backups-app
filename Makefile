# Run `make` without arguments to list the commands.

APP     := backups
TARGET  := x86_64-pc-windows-gnu
# -T: no terminal, so the commands also work in scripts and CI.
EXEC    := docker compose exec -T rust
EXE_DEV := target/$(TARGET)/debug/$(APP).exe
EXE_REL := target/$(TARGET)/release/$(APP).exe
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
PORTABLE := target/portable/$(APP)-$(VERSION)-portable.zip

.DEFAULT_GOAL := help
.PHONY: help up down shell build release run run-release portable check test screenshots icon clippy audit fmt check-all clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

up: ## Build the image and start the container
	docker compose up -d --build

down: ## Stop and remove the container
	docker compose down

build: up ## Build in debug mode
	$(EXEC) cargo build

release: up ## Build in release mode
	$(EXEC) cargo build --release

run: build ## Build in debug and run the application on Windows
	./$(EXE_DEV)

run-release: release ## Build in release and run the application on Windows
	./$(EXE_REL)

portable: release ## Build the portable Windows package (zip) in target/portable/
	$(EXEC) mkdir -p target/portable
	$(EXEC) rm -f $(PORTABLE)
	$(EXEC) zip -j -9 $(PORTABLE) $(EXE_REL)
	@echo "Portable: $(PORTABLE)"

check: up ## Check the code without building
	$(EXEC) cargo check

test: up ## Run the tests on Linux (the engine does not depend on Windows)
	$(EXEC) cargo test --target x86_64-unknown-linux-gnu

screenshots: up ## Generate UI screenshots in screenshots/ (filter with S=name1,name2)
	$(EXEC) env SCREENSHOTS=$(S) cargo test --target x86_64-unknown-linux-gnu --bin $(APP) -- --ignored --exact ui::screenshots::screenshots --nocapture

icon: up ## Generate assets/icon.ico, icon.png and icon.svg from the logo (theme::logo)
	$(EXEC) cargo test --target x86_64-unknown-linux-gnu --bin $(APP) -- --ignored --exact ui::icon::icon --nocapture

clippy: up ## Run the clippy linter
	$(EXEC) cargo clippy

audit: up ## Check dependencies for known vulnerabilities (cargo-audit)
	$(EXEC) cargo audit

fmt: up ## Format the code
	$(EXEC) cargo fmt

check-all: fmt clippy test audit ## Run fmt, clippy, test and audit (before a commit)

clean: up ## Remove build artifacts
	$(EXEC) cargo clean
