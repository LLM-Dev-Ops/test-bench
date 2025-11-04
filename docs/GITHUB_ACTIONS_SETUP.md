# GitHub Actions Setup Guide

Step-by-step guide to configure GitHub Actions for the LLM Test Bench project.

## Prerequisites

- GitHub repository created
- Admin access to repository settings
- Accounts created on external services (Codecov, npm)

## Step 1: Repository Setup

### 1.1 Push Code to GitHub

```bash
# Initialize git (if not already done)
git init

# Add all files
git add .

# Create initial commit
git commit -m "feat: initial CI/CD setup with GitHub Actions"

# Add remote repository
git remote add origin https://github.com/YOUR_USERNAME/llm-test-bench.git

# Push to GitHub
git push -u origin main
```

### 1.2 Create Development Branch

```bash
# Create and push develop branch
git checkout -b develop
git push -u origin develop
```

## Step 2: Configure Branch Protection

Navigate to: **Repository Settings → Branches → Add branch protection rule**

### For `main` branch:

```yaml
Branch name pattern: main

Settings:
✅ Require a pull request before merging
  ✅ Require approvals (1)
  ✅ Dismiss stale pull request approvals when new commits are pushed
  ✅ Require review from Code Owners

✅ Require status checks to pass before merging
  ✅ Require branches to be up to date before merging
  Status checks:
    - ci-success
    - format
    - lint
    - typecheck
    - test (node 18)
    - test (node 20)
    - test (node 22)
    - coverage

✅ Require conversation resolution before merging
✅ Require signed commits (optional)
✅ Include administrators
✅ Restrict who can push to matching branches
```

### For `develop` branch:

Similar settings but may be less strict (e.g., no approval required for faster iteration).

## Step 3: Configure Codecov

### 3.1 Sign Up for Codecov

1. Go to [codecov.io](https://codecov.io)
2. Sign in with GitHub
3. Grant access to your repositories

### 3.2 Add Repository

1. Click "Add new repository"
2. Select `llm-test-bench`
3. Copy the upload token

### 3.3 Add Token to GitHub Secrets

1. Go to **Repository Settings → Secrets and variables → Actions**
2. Click "New repository secret"
3. Name: `CODECOV_TOKEN`
4. Value: Paste the token from Codecov
5. Click "Add secret"

### 3.4 Configure Codecov (Optional)

Create `.codecov.yml` in repository root:

```yaml
coverage:
  status:
    project:
      default:
        target: 80%
        threshold: 1%
    patch:
      default:
        target: 80%

comment:
  layout: "reach, diff, flags, files"
  behavior: default
  require_changes: false
```

## Step 4: Configure NPM Publishing (Optional)

Only needed if you plan to publish to npm.

### 4.1 Create NPM Account

1. Sign up at [npmjs.com](https://www.npmjs.com)
2. Verify your email

### 4.2 Generate NPM Token

1. Login to npm: `npm login`
2. Go to [npmjs.com/settings/tokens](https://www.npmjs.com/settings/YOUR_USERNAME/tokens)
3. Click "Generate New Token"
4. Select "Automation" type
5. Copy the token

### 4.3 Add Token to GitHub Secrets

1. Go to **Repository Settings → Secrets and variables → Actions**
2. Click "New repository secret"
3. Name: `NPM_TOKEN`
4. Value: Paste the npm token
5. Click "Add secret"

### 4.4 Update package.json

Update repository URLs in `package.json`:

```json
{
  "repository": {
    "type": "git",
    "url": "https://github.com/YOUR_USERNAME/llm-test-bench.git"
  },
  "bugs": {
    "url": "https://github.com/YOUR_USERNAME/llm-test-bench/issues"
  },
  "homepage": "https://github.com/YOUR_USERNAME/llm-test-bench#readme"
}
```

## Step 5: Configure Snyk (Optional)

Enhanced security scanning with Snyk.

### 5.1 Sign Up for Snyk

1. Go to [snyk.io](https://snyk.io)
2. Sign up with GitHub
3. Import your repository

### 5.2 Get API Token

1. Go to Account Settings
2. Copy your API token

### 5.3 Add Token to GitHub Secrets

1. Go to **Repository Settings → Secrets and variables → Actions**
2. Click "New repository secret"
3. Name: `SNYK_TOKEN`
4. Value: Paste the Snyk token
5. Click "Add secret"

## Step 6: Configure Dependabot

Dependabot is already configured in `.github/dependabot.yml`. Update reviewer/assignee:

```yaml
reviewers:
  - "YOUR_GITHUB_USERNAME"
assignees:
  - "YOUR_GITHUB_USERNAME"
```

### Enable Dependabot Alerts

1. Go to **Repository Settings → Security & analysis**
2. Enable:
   - ✅ Dependency graph
   - ✅ Dependabot alerts
   - ✅ Dependabot security updates
   - ✅ Dependabot version updates

## Step 7: Enable GitHub Actions

### 7.1 Verify Actions are Enabled

1. Go to **Repository Settings → Actions → General**
2. Ensure "Allow all actions and reusable workflows" is selected

### 7.2 Configure Workflow Permissions

In the same section:

```
Workflow permissions:
○ Read repository contents and packages permissions
● Read and write permissions

✅ Allow GitHub Actions to create and approve pull requests
```

## Step 8: Test the CI Pipeline

### 8.1 Create Test PR

```bash
# Create a new branch
git checkout -b test/ci-pipeline

# Make a small change
echo "# CI Test" >> test.md

# Commit and push
git add test.md
git commit -m "test: verify CI pipeline"
git push -u origin test/ci-pipeline
```

### 8.2 Create Pull Request

1. Go to GitHub repository
2. Click "Pull requests" → "New pull request"
3. Select `test/ci-pipeline` → `main`
4. Create the PR
5. Watch the CI checks run

### 8.3 Verify All Checks Pass

You should see:
- ✅ Format check
- ✅ Lint
- ✅ Type check
- ✅ Tests (Node 18, 20, 22)
- ✅ Coverage
- ✅ Build
- ✅ CI Success

If any fail, check the logs and fix issues.

## Step 9: Test Release Workflow

### 9.1 Create a Test Release

```bash
# Ensure you're on main branch
git checkout main
git pull

# Create and push a tag
git tag v0.1.0
git push origin v0.1.0
```

### 9.2 Verify Release Workflow

1. Go to **Actions** tab
2. Find "Release" workflow run
3. Verify all jobs complete successfully
4. Check **Releases** tab for new release

## Step 10: Add Status Badges to README

Update `README.md` with badges (replace USERNAME):

```markdown
[![CI](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/ci.yml)
[![Security](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml/badge.svg)](https://github.com/USERNAME/llm-test-bench/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/USERNAME/llm-test-bench/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/llm-test-bench)
```

## Troubleshooting

### CI Workflow Not Running

**Check:**
1. Actions are enabled in repository settings
2. Workflow file is in `.github/workflows/` directory
3. YAML syntax is valid (use a YAML validator)
4. Branch name matches trigger conditions

### Coverage Upload Fails

**Solutions:**
1. Verify `CODECOV_TOKEN` is set correctly
2. Check Codecov has access to repository
3. Ensure coverage files are generated (`npm run test:coverage`)

### NPM Publish Fails

**Solutions:**
1. Verify `NPM_TOKEN` is set correctly
2. Check package name is available on npm
3. Ensure version number is incremented
4. Verify you have publish rights to the package

### Permission Denied Errors

**Solutions:**
1. Check workflow permissions in repository settings
2. Ensure `GITHUB_TOKEN` has write access
3. Verify branch protection rules aren't blocking

## Monitoring and Maintenance

### Weekly Tasks
- Review Dependabot PRs
- Check security alerts
- Monitor CI performance

### Monthly Tasks
- Review and update dependencies
- Check coverage trends
- Update workflow actions to latest versions

### Quarterly Tasks
- Audit security settings
- Review and optimize CI performance
- Update documentation

## Getting Help

- 📚 [GitHub Actions Documentation](https://docs.github.com/en/actions)
- 📊 [Codecov Documentation](https://docs.codecov.com)
- 📦 [npm Documentation](https://docs.npmjs.com)
- 🔒 [Snyk Documentation](https://docs.snyk.io)

## Next Steps

After setup is complete:

1. ✅ All CI checks passing
2. ✅ Coverage reporting to Codecov
3. ✅ Branch protection enabled
4. ✅ Dependabot configured
5. ✅ Badges added to README

You're ready to start development! 🎉

Read [CONTRIBUTING.md](../CONTRIBUTING.md) for development workflow and guidelines.
