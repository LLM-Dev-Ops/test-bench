import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { runInNewContext } from 'node:vm';
import { describe, expect, it } from 'vitest';
import YAML from 'yaml';

type Job = { needs?: string[]; steps?: { run?: string }[] };
const file = fileURLToPath(new URL('../.github/workflows/ci.yml', import.meta.url));
const workflow = YAML.parse(readFileSync(file, 'utf8')) as { jobs: Record<string, Job> };
const required = Object.keys(workflow.jobs).filter((id) => id !== 'ci-success');
const aggregate = workflow.jobs['ci-success'];
const expression = aggregate.steps?.[0].run?.match(/\$\{\{ (.*?) \}\}/)?.[1];

describe('aggregate CI fails closed on every required job', () => {
  it('depends on every other job, including integration and production security', () => {
    expect(new Set(aggregate.needs)).toEqual(new Set(required));
    expect(aggregate.needs).toContain('integration');
    expect(aggregate.needs).toContain('dependency-security');
  });

  const evaluate = (results: string[]) => {
    expect(expression).toBeDefined();
    const runnable = expression?.replaceAll('needs.*.result', 'results') ?? '';
    return runInNewContext(
      runnable,
      { results, contains: (values: string[], value: string) => values.includes(value) },
      { timeout: 100 },
    ) as boolean;
  };

  it('accepts only the all-success result vector', () => {
    expect(evaluate(required.map(() => 'success'))).toBe(false);
  });

  for (const job of required) {
    for (const status of ['failure', 'cancelled', 'skipped']) {
      it(`rejects ${job} ${status}`, () => {
        expect(evaluate(required.map((id) => (id === job ? status : 'success')))).toBe(true);
      });
    }
  }
});
