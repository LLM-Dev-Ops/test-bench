import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

describe('dependency proxy trust security and compatibility', () => {
  const script = fileURLToPath(new URL('../scripts/verify-proxy-boundary.mjs', import.meta.url));
  it('retains the frozen real HTTP trust-boundary contract', () => {
    const output = execFileSync(process.execPath, [script, process.cwd()], {
      encoding: 'utf8',
      timeout: 15000,
    });
    const result = JSON.parse(output) as { results: { id: string; passed: boolean }[] };
    expect(result.results).toHaveLength(5);
    expect(result.results.every((item) => item.passed)).toBe(true);
  });
});
