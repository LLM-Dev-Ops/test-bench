# Contributing to LLM Test Bench

Thank you for your interest in contributing to LLM Test Bench! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [CI/CD Pipeline](#cicd-pipeline)
- [Code Quality Standards](#code-quality-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)

## Code of Conduct

Please be respectful and constructive in all interactions. We're building a welcoming community.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/llm-test-bench.git`
3. Add upstream remote: `git remote add upstream https://github.com/ORIGINAL_OWNER/llm-test-bench.git`
4. Create a branch: `git checkout -b feature/your-feature-name`

## Development Setup

### Prerequisites

- Node.js 18+ (we recommend using nvm: `nvm use`)
- npm 9+
- Git

### Installation

```bash
# Install dependencies
npm install

# Setup git hooks (for automatic linting)
npm run prepare
```

### Development Workflow

```bash
# Start development mode with watch
npm run dev

# Run type checking
npm run typecheck

# Run linter
npm run lint

# Run tests in watch mode
npm run test:watch

# Run all checks (like CI does)
npm run ci
```

## CI/CD Pipeline

Our CI/CD pipeline ensures code quality and consistency. All checks must pass before a PR can be merged.

### CI Workflow (Automated on Push/PR)

The CI workflow runs on every push and pull request to `main` or `develop` branches:

1. **Format Check** - Verifies code is formatted with Prettier
2. **Lint** - Runs ESLint with zero warnings allowed
3. **Type Check** - Ensures TypeScript compiles without errors
4. **Tests** - Runs unit tests on Node.js 18, 20, and 22
5. **Coverage** - Generates code coverage report
6. **Build** - Verifies the package builds successfully

### Running CI Checks Locally

Before submitting a PR, run all CI checks locally:

```bash
# Run all CI checks
npm run ci

# Or run individually:
npm run format:check  # Check formatting
npm run lint          # Check linting
npm run typecheck     # Check types
npm run test          # Run tests
npm run build         # Build package
```

### Security Workflow

The security workflow runs daily and on pull requests:

- **NPM Audit** - Checks for known vulnerabilities in dependencies
- **CodeQL Analysis** - Static analysis for security issues
- **License Check** - Ensures all dependencies use approved licenses
- **Secret Scanning** - Detects accidentally committed secrets
- **SBOM Generation** - Creates Software Bill of Materials

### Release Workflow

Releases are automated via git tags:

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

This triggers:
- Full test suite
- Build for multiple platforms
- GitHub release creation
- NPM package publication
- Docker image build (if Dockerfile exists)

## Code Quality Standards

### TypeScript

- Use strict mode (already configured)
- Avoid `any` type - use proper typing
- Export types alongside implementations
- Use ES modules (`import`/`export`)

### ESLint Rules

- **Zero warnings policy** - All warnings are treated as errors in CI
- Maximum line length: 100 characters
- Use `const` over `let` when possible
- Prefer arrow functions
- Use async/await over promises

### Prettier Formatting

- Single quotes for strings
- 2 space indentation
- Trailing commas
- Semicolons required
- Line width: 100 characters

Auto-format on save is recommended. Configure your editor:

**VS Code (.vscode/settings.json):**
```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  }
}
```

## Testing Requirements

### Coverage Thresholds

- **Phase 1**: 80% coverage (lines, functions, branches, statements)
- **Phase 5**: 90% coverage (target for production)

### Writing Tests

```typescript
// Example test structure
import { describe, it, expect, beforeEach } from 'vitest';

describe('MyComponent', () => {
  beforeEach(() => {
    // Setup
  });

  it('should do something', () => {
    // Arrange
    const input = 'test';

    // Act
    const result = myFunction(input);

    // Assert
    expect(result).toBe('expected');
  });
});
```

### Test Types

1. **Unit Tests** - Test individual functions/classes
   - Location: `src/**/*.test.ts`
   - Run: `npm test`

2. **Integration Tests** - Test component interactions
   - Location: `tests/integration/**/*.test.ts`
   - Run: `npm run test:integration`

3. **Coverage** - Generate coverage reports
   - Run: `npm run test:coverage`
   - View: Open `coverage/lcov-report/index.html`

## Pull Request Process

### Before Submitting

1. ✅ All tests pass: `npm test`
2. ✅ Coverage meets threshold: `npm run test:coverage:check`
3. ✅ No linting errors: `npm run lint`
4. ✅ No type errors: `npm run typecheck`
5. ✅ Code is formatted: `npm run format:check`
6. ✅ Build succeeds: `npm run build`

Or run everything at once:
```bash
npm run ci
```

### PR Checklist

- [ ] Branch is up to date with main
- [ ] All CI checks pass
- [ ] Tests added/updated for changes
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
- [ ] Commit messages follow convention
- [ ] PR description explains the change

### PR Template

When creating a PR, include:

```markdown
## Description
Brief description of the change

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How has this been tested?

## Checklist
- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] No new warnings
```

## Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding/updating tests
- `chore`: Build process, dependencies, etc.
- `ci`: CI/CD changes

### Examples

```bash
feat(cli): add new run command

Implement the 'ltb run' command with support for glob patterns
and filtering options.

Closes #123
```

```bash
fix(providers): handle rate limit errors

Add retry logic with exponential backoff when OpenAI rate limits
are hit.
```

```bash
docs(readme): update installation instructions

Add troubleshooting section for common installation issues.
```

### Scopes

Common scopes:
- `cli` - CLI interface
- `config` - Configuration system
- `providers` - LLM providers
- `assertions` - Assertion engine
- `reporting` - Report generation
- `core` - Core functionality
- `tests` - Test infrastructure
- `ci` - CI/CD pipeline

## Development Tips

### Running Tests for Specific Files

```bash
# Run tests for a specific file
npm test -- src/cli/index.test.ts

# Run tests in watch mode for development
npm run test:watch
```

### Debugging Tests

```bash
# Run tests with UI
npm run test:ui
```

### Husky Git Hooks

Pre-commit hooks automatically run:
- ESLint on staged TypeScript files
- Prettier on staged files

To bypass hooks (not recommended):
```bash
git commit --no-verify
```

### Common Issues

**"Module not found" errors:**
```bash
rm -rf node_modules package-lock.json
npm install
```

**Type errors after update:**
```bash
npm run typecheck
```

**Coverage threshold failures:**
```bash
npm run test:coverage
# Check which files need more coverage
```

## Getting Help

- 📖 Read the [documentation](./docs/)
- 💬 Ask questions in [GitHub Discussions](https://github.com/yourusername/llm-test-bench/discussions)
- 🐛 Report bugs in [GitHub Issues](https://github.com/yourusername/llm-test-bench/issues)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
