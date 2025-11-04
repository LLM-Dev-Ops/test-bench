# CI/CD Pipeline Documentation

Complete guide to the LLM Test Bench CI/CD pipeline architecture and workflows.

## Table of Contents

- [Overview](#overview)
- [Pipeline Architecture](#pipeline-architecture)
- [Workflows](#workflows)
- [Quality Gates](#quality-gates)
- [Running CI Locally](#running-ci-locally)
- [GitHub Secrets Configuration](#github-secrets-configuration)
- [Badge Setup](#badge-setup)
- [Troubleshooting](#troubleshooting)

## Overview

The LLM Test Bench project uses GitHub Actions for continuous integration and continuous deployment. Our CI/CD pipeline ensures code quality, security, and reliability through automated testing, linting, and security scanning.

### Pipeline Goals

- ✅ Maintain high code quality standards
- ✅ Catch bugs before they reach production
- ✅ Ensure consistent code formatting and style
- ✅ Detect security vulnerabilities early
- ✅ Automate release process
- ✅ Provide fast feedback to developers

## Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CI/CD Pipeline                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Format    │  │    Lint     │  │  Type Check │        │
│  │   Check     │  │   (ESLint)  │  │ (TypeScript)│        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                │
│         └────────────────┴────────────────┘                │
│                          │                                 │
│                   ┌──────▼──────┐                         │
│                   │    Tests    │                         │
│                   │ (Vitest)    │                         │
│                   │ Node 18,20  │                         │
│                   │    ,22      │                         │
│                   └──────┬──────┘                         │
│                          │                                 │
│                   ┌──────▼──────┐                         │
│                   │  Coverage   │                         │
│                   │ (Codecov)   │                         │
│                   │   80%+      │                         │
│                   └──────┬──────┘                         │
│                          │                                 │
│                   ┌──────▼──────┐                         │
│                   │   Build     │                         │
│                   │ Verification│                         │
│                   └──────┬──────┘                         │
│                          │                                 │
│                   ┌──────▼──────┐                         │
│                   │Integration  │                         │
│                   │   Tests     │                         │
│                   └──────┬──────┘                         │
│                          │                                 │
│                   ┌──────▼──────┐                         │
│                   │ CI Success  │                         │
│                   └─────────────┘                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                 Security Workflow (Daily)                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   NPM    │  │  CodeQL  │  │ License  │  │ Secrets  │  │
│  │  Audit   │  │ Analysis │  │  Check   │  │ Scanning │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │              │              │        │
│       └─────────────┴──────────────┴──────────────┘        │
│                          │                                 │
│                   ┌──────▼──────┐                         │
│                   │    SBOM     │                         │
│                   │ Generation  │                         │
│                   └─────────────┘                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  Release Workflow (on tag)                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐       ┌──────────┐       ┌──────────┐       │
│  │ Validate │   ──▶ │  Build   │   ──▶ │  GitHub  │       │
│  │ Version  │       │ Artifacts│       │ Release  │       │
│  └──────────┘       └──────────┘       └────┬─────┘       │
│                                              │             │
│                          ┌───────────────────┴─────┐       │
│                          │                         │       │
│                   ┌──────▼──────┐         ┌────────▼────┐  │
│                   │  NPM        │         │   GitHub    │  │
│                   │  Publish    │         │   Packages  │  │
│                   └─────────────┘         └─────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Jobs:**

#### Format Check
- Runs Prettier to verify code formatting
- Zero tolerance for formatting issues
- Fast feedback (< 1 min)

```bash
# Run locally
npm run format:check
```

#### Lint
- Runs ESLint with TypeScript support
- Zero warnings policy - all warnings treated as errors
- Checks code style and best practices

```bash
# Run locally
npm run lint
```

#### Type Check
- Validates TypeScript types
- Ensures type safety across codebase
- Catches type errors early

```bash
# Run locally
npm run typecheck
```

#### Test
- Runs unit tests on Node.js 18, 20, and 22
- Matrix strategy for cross-version compatibility
- Uploads test results as artifacts
- Fails fast on first failure (optional)

```bash
# Run locally
npm test
```

#### Coverage
- Generates code coverage reports
- Uploads to Codecov for tracking
- Enforces 80% coverage threshold (Phase 1)
- Will increase to 90% in Phase 5

```bash
# Run locally
npm run test:coverage
```

**Coverage Requirements:**
- Lines: 80%+
- Functions: 80%+
- Branches: 80%+
- Statements: 80%+

#### Build
- Verifies package builds successfully
- Checks bundle size (< 5MB limit)
- Produces distribution artifacts
- Validates package.json configuration

```bash
# Run locally
npm run build
```

#### Integration Tests
- Runs integration tests if present
- Tests component interactions
- Longer timeout (15 min)

```bash
# Run locally
npm run test:integration
```

#### CI Success
- Summary job that requires all checks to pass
- Branch protection uses this status
- Single source of truth for CI status

### 2. Security Workflow (`.github/workflows/security.yml`)

**Triggers:**
- Daily at 2 AM UTC (scheduled)
- Pull requests to `main` or `develop`
- Manual dispatch

**Jobs:**

#### NPM Audit
- Scans for known vulnerabilities in dependencies
- Fails on high-severity issues in production dependencies
- Warns on moderate issues

```bash
# Run locally
npm audit
npm audit --production
```

#### CodeQL Analysis
- Static analysis for security vulnerabilities
- Scans JavaScript/TypeScript code
- Uploads results to GitHub Security tab
- Uses extended security queries

#### License Check
- Validates all dependency licenses
- Fails on GPL, AGPL, LGPL licenses
- Generates license report artifact

```bash
# Run locally
npx license-checker
```

#### Secret Scanning
- Detects accidentally committed secrets
- Uses Gitleaks for scanning
- Checks entire git history
- Prevents credential leaks

#### SBOM Generation
- Creates Software Bill of Materials
- Uses CycloneDX format
- Lists all dependencies and versions
- Useful for compliance and auditing

```bash
# Run locally
npx @cyclonedx/cyclonedx-npm --output-file sbom.json
```

#### Dependency Review (PR only)
- Reviews dependency changes in PRs
- Detects newly introduced vulnerabilities
- Fails on moderate+ severity issues

### 3. Release Workflow (`.github/workflows/release.yml`)

**Triggers:**
- Git tag push matching `v*.*.*` pattern
- Manual workflow dispatch with version input

**Jobs:**

#### Validate
- Extracts version from tag
- Validates semantic version format
- Outputs version for other jobs

#### Test
- Reuses CI workflow
- Ensures all tests pass before release

#### Build
- Builds on Ubuntu, macOS, and Windows
- Creates platform-specific artifacts
- Packages distribution files

#### GitHub Release
- Generates changelog from git commits
- Creates GitHub release
- Uploads build artifacts
- Marks as prerelease if version contains `-`

#### NPM Publish
- Updates package.json version
- Publishes to npm registry
- Uses `latest` tag for stable releases
- Uses `next` tag for prereleases

**Required Secret:** `NPM_TOKEN`

#### GitHub Packages
- Publishes to GitHub Package Registry
- Scoped to organization/user
- Automatic authentication via `GITHUB_TOKEN`

#### Docker (optional)
- Builds multi-platform Docker images
- Pushes to GitHub Container Registry
- Tags with semantic versions
- Only runs if Dockerfile exists

#### Documentation Update
- Updates version badges in README
- Auto-commits to main branch
- Only for stable releases

## Quality Gates

All PRs must pass these gates before merging:

### Required Checks

1. ✅ **Format** - All code properly formatted
2. ✅ **Lint** - Zero ESLint warnings/errors
3. ✅ **Type Check** - No TypeScript errors
4. ✅ **Tests** - All tests passing on all Node versions
5. ✅ **Coverage** - 80%+ code coverage maintained
6. ✅ **Build** - Package builds successfully

### Recommended Checks

- 🔒 Security scans pass
- 📝 Documentation updated
- 🧪 Integration tests pass
- 📦 Bundle size within limits

### Branch Protection

Configure in GitHub Settings → Branches → Branch Protection Rules:

```yaml
Branch: main
Require status checks:
  - ci-success
  - format
  - lint
  - typecheck
  - test (node 18)
  - test (node 20)
  - test (node 22)
  - coverage

Require pull request reviews: 1
Require branches to be up to date: true
```

## Running CI Locally

### Complete CI Check

Run everything that CI runs:

```bash
npm run ci
```

This executes:
1. ESLint
2. TypeScript type check
3. Tests with coverage
4. Build

### Individual Checks

```bash
# Format check
npm run format:check

# Format fix
npm run format

# Lint check
npm run lint

# Lint fix
npm run lint:fix

# Type check
npm run typecheck

# Tests
npm test

# Tests with coverage
npm run test:coverage

# Integration tests
npm run test:integration

# Build
npm run build
```

### Pre-commit Hook

Husky automatically runs on commit:
- ESLint (auto-fix)
- Prettier (auto-format)

Only on staged files for speed.

### Watch Mode Development

```bash
# Watch tests
npm run test:watch

# Watch build
npm run dev

# Watch with UI
npm run test:ui
```

## GitHub Secrets Configuration

Configure these secrets in GitHub Settings → Secrets and variables → Actions:

### Required Secrets

#### `CODECOV_TOKEN`
- **Purpose:** Upload coverage reports to Codecov
- **How to get:**
  1. Sign up at [codecov.io](https://codecov.io)
  2. Add your GitHub repository
  3. Copy the upload token
- **Required for:** Coverage job in CI workflow

#### `NPM_TOKEN` (for releases)
- **Purpose:** Publish packages to npm registry
- **How to get:**
  1. Login to [npmjs.com](https://www.npmjs.com)
  2. Settings → Access Tokens → Generate New Token
  3. Choose "Automation" type
- **Required for:** Release workflow

### Optional Secrets

#### `SNYK_TOKEN`
- **Purpose:** Advanced security scanning with Snyk
- **How to get:**
  1. Sign up at [snyk.io](https://snyk.io)
  2. Account Settings → API Token
- **Required for:** Snyk security scanning (optional)

### Environment Variables

#### `COVERAGE_PHASE`
- **Purpose:** Set coverage threshold (phase1 = 80%, phase5 = 90%)
- **Default:** phase1
- **Usage:** Set in workflow or as repo variable

## Badge Setup

Add these badges to your README.md:

### CI Status Badge

```markdown
[![CI](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml)
```

### Coverage Badge

```markdown
[![codecov](https://codecov.io/gh/USERNAME/llm-test-bench/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/llm-test-bench)
```

### Security Badge

```markdown
[![Security](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml)
```

### NPM Version Badge

```markdown
[![npm version](https://badge.fury.io/js/llm-test-bench.svg)](https://www.npmjs.com/package/llm-test-bench)
```

### Node Version Badge

```markdown
[![node version](https://img.shields.io/node/v/llm-test-bench.svg)](https://nodejs.org)
```

### License Badge

```markdown
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

### Complete Badge Section

```markdown
## Status

[![CI](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml)
[![Security](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/USERNAME/llm-test-bench/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/llm-test-bench)
[![npm version](https://badge.fury.io/js/llm-test-bench.svg)](https://www.npmjs.com/package/llm-test-bench)
[![node version](https://img.shields.io/node/v/llm-test-bench.svg)](https://nodejs.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

## Troubleshooting

### Common Issues

#### "Cannot find module" in CI

**Cause:** Dependencies not installed or cache issues

**Solution:**
```yaml
- name: Install dependencies
  run: npm ci  # Use ci instead of install
```

#### Tests pass locally but fail in CI

**Cause:** Environment differences or timezone issues

**Solution:**
- Check Node.js version matches
- Use `process.env.CI` to detect CI environment
- Mock time-dependent functions

#### Coverage threshold failures

**Cause:** New code not fully tested

**Solution:**
```bash
# Check coverage locally
npm run test:coverage

# View HTML report
open coverage/lcov-report/index.html

# Focus on untested files
```

#### Build fails on Windows

**Cause:** Path separator differences

**Solution:**
- Use `path.join()` instead of string concatenation
- Test locally on Windows or use GitHub Actions matrix

#### Slow CI runs

**Optimization tips:**
- Use `npm ci` instead of `npm install`
- Enable caching: `cache: 'npm'`
- Run jobs in parallel when possible
- Reduce test timeout if tests are fast

### Getting Help

1. Check [Actions logs](https://github.com/USERNAME/llm-test-bench/actions)
2. Review [GitHub Actions documentation](https://docs.github.com/en/actions)
3. Search [GitHub Discussions](https://github.com/USERNAME/llm-test-bench/discussions)
4. Open an [issue](https://github.com/USERNAME/llm-test-bench/issues)

## Performance Metrics

Target CI performance:
- ⚡ Format check: < 30 seconds
- ⚡ Lint: < 1 minute
- ⚡ Type check: < 1 minute
- ⚡ Tests: < 3 minutes
- ⚡ Coverage: < 5 minutes
- ⚡ Build: < 2 minutes
- ⚡ **Total CI time: < 10 minutes**

## Continuous Improvement

The CI/CD pipeline evolves with the project:

### Phase 1 (Current)
- ✅ Basic CI/CD setup
- ✅ 80% coverage requirement
- ✅ Essential security scans

### Phase 5 (Future)
- 🎯 90% coverage requirement
- 🎯 Performance benchmarking
- 🎯 Visual regression testing
- 🎯 Automated dependency updates

### Future Enhancements
- 🔮 Automated changelog generation
- 🔮 Semantic release automation
- 🔮 Preview deployments for PRs
- 🔮 E2E testing in CI
- 🔮 Performance monitoring
- 🔮 Docker layer caching

---

**Last Updated:** 2025-11-04

**Maintained By:** DevOps Team
