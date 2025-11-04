# Publishing Guide

This guide covers how to publish LLM Test Bench to crates.io and create npm bindings.

## Publishing to crates.io

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Create a crates.io account**:
   - Visit https://crates.io/
   - Sign in with GitHub
   - Generate an API token at https://crates.io/settings/tokens

3. **Login to crates.io**:
   ```bash
   cargo login <your-api-token>
   ```

### Publication Order

Publish packages in dependency order:

#### 1. Publish datasets package first (no dependencies)
```bash
cd datasets
cargo publish --dry-run  # Test first
cargo publish            # Publish
```

#### 2. Publish core package (depends on datasets)
```bash
cd ../core
cargo publish --dry-run  # Test first
cargo publish            # Publish
```

#### 3. Publish CLI package (depends on core and datasets)
```bash
cd ../cli
cargo publish --dry-run  # Test first
cargo publish            # Publish
```

### Verification

After publishing, verify the packages:

```bash
# Check that packages are published
cargo search llm-test-bench

# Test installation
cargo install llm-test-bench

# Verify the CLI works
llm-test-bench --version
```

### Troubleshooting

#### "crate already uploaded" error
- You cannot republish the same version
- Increment version in `Cargo.toml` workspace package section

#### "dependency not found" error
- Ensure datasets and core are published before CLI
- Wait a few minutes for crates.io to index

#### "missing documentation" warnings
- Ensure all public items have doc comments
- Run `cargo doc --no-deps` to check documentation

---

## npm Package Strategy

Since LLM Test Bench is written in Rust, there are several options for npm:

### Option 1: Binary Distribution via npm

Distribute pre-compiled binaries through npm (recommended for CLI tools):

1. **Create an npm package** that downloads the appropriate binary:
   ```json
   {
     "name": "@llm-test-bench/cli",
     "version": "0.1.0",
     "description": "LLM Test Bench CLI",
     "bin": {
       "llm-test-bench": "./bin/llm-test-bench"
     },
     "scripts": {
       "postinstall": "node install.js"
     }
   }
   ```

2. **Create install.js** to download the binary:
   ```javascript
   const { execSync } = require('child_process');
   const os = require('os');

   const platform = os.platform();
   const arch = os.arch();

   // Download appropriate binary from GitHub releases
   // Based on platform and architecture
   ```

3. **Publish to npm**:
   ```bash
   npm publish
   ```

### Option 2: WASM Bindings

Create WebAssembly bindings for browser/Node.js use:

1. **Add wasm-pack dependencies**:
   ```bash
   cargo install wasm-pack
   ```

2. **Create wasm package**:
   ```bash
   # Add to Cargo.toml
   [lib]
   crate-type = ["cdylib", "rlib"]

   [dependencies]
   wasm-bindgen = "0.2"
   ```

3. **Build and publish**:
   ```bash
   wasm-pack build --target web
   wasm-pack publish
   ```

### Option 3: Node.js Native Bindings

Use napi-rs for Node.js native modules:

1. **Install napi-rs**:
   ```bash
   cargo install napi-cli
   ```

2. **Create napi project**:
   ```bash
   napi new
   ```

3. **Build and publish**:
   ```bash
   npm run build
   npm publish
   ```

### Recommended Approach

For LLM Test Bench, **Option 1 (Binary Distribution)** is recommended because:
- It's a CLI tool, not a library
- Users want the full functionality, not JS bindings
- Simpler to maintain and distribute
- Better performance (no overhead)

### Example npm Binary Package Structure

```
@llm-test-bench/cli/
├── package.json
├── README.md
├── install.js         # Downloads binary from GitHub releases
├── bin/
│   └── llm-test-bench # Symlink to actual binary
└── scripts/
    └── build-all.sh   # CI script to build for all platforms
```

---

## Continuous Deployment

### Automated crates.io Publishing

Add to `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Publish datasets
        run: cd datasets && cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}

      - name: Publish core
        run: cd core && cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}

      - name: Publish CLI
        run: cd cli && cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}
```

### Automated npm Publishing

Add to the same workflow:

```yaml
  npm-publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Publish to npm
        run: npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

---

## Version Management

### Updating Versions

Update the version in the workspace `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Update this
```

All packages will inherit this version.

### Creating a Release

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes:
   ```bash
   git commit -am "chore: bump version to 0.2.0"
   ```
4. Create and push tag:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```
5. Publish to crates.io (see above)
6. Create GitHub release with binaries

---

## Checklist Before Publishing

- [ ] All tests pass: `cargo test --all`
- [ ] Clippy is happy: `cargo clippy --all -- -D warnings`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] README is up to date
- [ ] CHANGELOG is updated
- [ ] Version is bumped appropriately
- [ ] All Cargo.toml metadata is correct
- [ ] LICENSE files are present
- [ ] Dry-run succeeds: `cargo publish --dry-run`

---

## Support

For issues with publishing:
- crates.io: https://crates.io/policies
- npm: https://docs.npmjs.com/cli/v10/commands/npm-publish
- GitHub: https://github.com/globalbusinessadvisors/llm-test-bench/issues
