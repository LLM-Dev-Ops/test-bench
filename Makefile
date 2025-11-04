# Makefile for LLM Test Bench
# Provides convenient shortcuts for common development tasks

.PHONY: help install dev build test lint format clean ci release

# Default target
help:
	@echo "LLM Test Bench - Available Commands:"
	@echo ""
	@echo "  make install       Install dependencies"
	@echo "  make dev           Start development mode with watch"
	@echo "  make build         Build the package"
	@echo "  make test          Run tests"
	@echo "  make test-watch    Run tests in watch mode"
	@echo "  make test-ui       Run tests with UI"
	@echo "  make coverage      Generate coverage report"
	@echo "  make lint          Run linter"
	@echo "  make lint-fix      Fix linting issues"
	@echo "  make format        Format code"
	@echo "  make format-check  Check code formatting"
	@echo "  make typecheck     Run TypeScript type checking"
	@echo "  make ci            Run all CI checks locally"
	@echo "  make clean         Clean build artifacts"
	@echo "  make release       Create a new release (requires VERSION)"
	@echo ""

# Install dependencies
install:
	npm install

# Development mode
dev:
	npm run dev

# Build package
build:
	npm run build

# Clean build
build-clean:
	npm run build:clean

# Run tests
test:
	npm test

# Watch mode for tests
test-watch:
	npm run test:watch

# Test UI
test-ui:
	npm run test:ui

# Generate coverage
coverage:
	npm run test:coverage
	@echo "Opening coverage report..."
	@open coverage/lcov-report/index.html || xdg-open coverage/lcov-report/index.html || echo "Open coverage/lcov-report/index.html manually"

# Integration tests
test-integration:
	npm run test:integration

# Lint code
lint:
	npm run lint

# Fix linting issues
lint-fix:
	npm run lint:fix

# Format code
format:
	npm run format

# Check formatting
format-check:
	npm run format:check

# Type checking
typecheck:
	npm run typecheck

# Run all CI checks
ci:
	@echo "Running all CI checks..."
	npm run ci

# Clean artifacts
clean:
	npm run clean

# Create a release (usage: make release VERSION=1.0.0)
release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release VERSION=1.0.0"; \
		exit 1; \
	fi
	@echo "Creating release v$(VERSION)..."
	@git tag -a v$(VERSION) -m "Release v$(VERSION)"
	@echo "Pushing tag v$(VERSION)..."
	@git push origin v$(VERSION)
	@echo "Release v$(VERSION) created and pushed!"

# Setup development environment
setup:
	@echo "Setting up development environment..."
	npm install
	npm run prepare
	@echo "Setup complete! Run 'make dev' to start development."

# Quick check before commit
check: format-check lint typecheck test
	@echo "All checks passed! Ready to commit."
