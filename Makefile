# Use bash
SHELL := /bin/bash

# Default target
.DEFAULT_GOAL := help


.PHONY: help
help: ## Show this help
	@echo "Usage: make [target]"
	@echo
	@echo "Targets:"
	@awk '/^[a-zA-Z\-\_0-9]+:/ { \
		helpMessage = match(lastLine, /^## (.*)/); \
		if (helpMessage) { \
			helpCommand = substr($$1, 0, index($$1, ":")-1); \
			helpMessage = substr(lastLine, RSTART + 3, RLENGTH); \
			printf "  \033[36m%-15s\033[0m %s
			


.PHONY: lint
lint: ## Lint project
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: test
test: ## Test project
	cargo test --all-targets --all-features

.PHONY: build
build: ## Build binary
	cargo build --all-targets --all-features