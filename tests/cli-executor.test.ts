import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'fs';
import { tmpdir } from 'os';
import { delimiter, join } from 'path';
import { afterEach, describe, expect, it } from 'vitest';

import { findCLIPath } from '../src/utils/cli-executor.js';

const originalEnv = { ...process.env };
const temporaryDirectories: string[] = [];

function makeExecutable(directory: string): string {
  const binaryName = process.platform === 'win32' ? 'llm-test-bench.exe' : 'llm-test-bench';
  mkdirSync(directory, { recursive: true });
  const binaryPath = join(directory, binaryName);
  writeFileSync(binaryPath, '#!/bin/sh\nexit 0\n');
  chmodSync(binaryPath, 0o755);
  return binaryPath;
}

function makeTemporaryDirectory(): string {
  const directory = mkdtempSync(join(tmpdir(), 'test-bench-cli-'));
  temporaryDirectories.push(directory);
  return directory;
}

afterEach(() => {
  process.env = { ...originalEnv };
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

describe('findCLIPath', () => {
  it('skips a configured Cargo location when the binary is absent', () => {
    const project = makeTemporaryDirectory();
    const localBinary = makeExecutable(join(project, 'node_modules', '.bin'));
    process.env.CARGO_HOME = join(project, 'missing-cargo-home');
    process.env.HOME = join(project, 'missing-home');
    process.env.PATH = '';

    expect(findCLIPath(project)).toBe(localBinary);
  });

  it('discovers an executable from PATH', () => {
    const root = makeTemporaryDirectory();
    const pathDirectory = join(root, 'custom-bin');
    const pathBinary = makeExecutable(pathDirectory);
    process.env.CARGO_HOME = join(root, 'missing-cargo-home');
    process.env.HOME = join(root, 'missing-home');
    process.env.PATH = [join(root, 'missing-bin'), pathDirectory].join(delimiter);

    expect(findCLIPath(root)).toBe(pathBinary);
  });

  it('returns undefined when no executable exists', () => {
    const root = makeTemporaryDirectory();
    process.env.CARGO_HOME = join(root, 'missing-cargo-home');
    process.env.HOME = join(root, 'missing-home');
    process.env.PATH = join(root, 'missing-bin');

    expect(findCLIPath(root)).toBeUndefined();
  });
});
