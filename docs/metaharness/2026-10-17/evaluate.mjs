import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';

// node evaluate.mjs BASELINE CANDIDATE PINNED_TOOLS OUTPUT
const [baseline, candidate, runtime, output] = process.argv.slice(2).map((x) => resolve(x));
const fw = await import(
  pathToFileURL(join(runtime, 'node_modules/@metaharness/flywheel/dist/index.js'))
);
assert.equal(
  JSON.parse(readFileSync(join(runtime, 'node_modules/@metaharness/flywheel/package.json')))
    .version,
  '0.1.12',
);
const contract = JSON.parse(readFileSync(new URL('./contract.json', import.meta.url)));
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const fixture = 'scripts/verify-proxy-boundary.mjs';
assert.equal(
  hash(readFileSync(join(baseline, fixture))),
  hash(readFileSync(join(candidate, fixture))),
);
const lockHash = (dir) => hash(readFileSync(join(dir, 'package-lock.json')));
const rootHash = lockHash(baseline),
  fixedHash = lockHash(candidate);
assert.notEqual(rootHash, fixedHash);
const parentBytes = JSON.stringify(
  Object.fromEntries(
    ['package.json', 'package-lock.json'].map((file) => [
      file,
      readFileSync(join(baseline, file), 'utf8'),
    ]),
  ),
);
const measurements = [];
let baselineNames;
const run = (dir, command, args) => {
  const result = spawnSync(command, args, {
    cwd: dir,
    encoding: 'utf8',
    timeout: 120000,
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(result.signal, null, `${command}: timeout/signal is not an evaluation result`);
  assert.notEqual(result.status, null);
  return {
    command: [command, ...args],
    exit: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  };
};
const compact = (command) => ({
  ...command,
  stdout: command.stdout.slice(-5000),
  stderr: command.stderr.slice(-3000),
});
const evaluator = async (policy, suite) => {
  const dir = policy.dependencies === rootHash ? baseline : candidate;
  assert.equal(lockHash(dir), policy.dependencies);
  const native = run(dir, process.execPath, [fixture, dir]);
  assert.ok(native.exit === 0 || native.exit === 1);
  const http = JSON.parse(native.stdout);
  assert.equal(http.results.length, 5, 'all native cases executed');
  const commands = [compact(native)];
  let itemWins, detail;
  if (suite.id === 'holdout') {
    const audit = run(dir, 'npm', ['audit', '--omit=dev', '--json']);
    assert.ok(audit.exit === 0 || audit.exit === 1);
    const report = JSON.parse(audit.stdout);
    assert.ok(!report.error && report.metadata?.vulnerabilities, 'actual audit response required');
    detail = { audit: report, http };
    commands.push({ command: audit.command, exit: audit.exit, stderr: audit.stderr });
    itemWins = [
      report.metadata.vulnerabilities.critical === 0,
      report.metadata.vulnerabilities.high === 0,
      http.results.find((x) => x.suite === 'holdout').passed,
    ];
  } else {
    const tests = run(dir, 'npm', ['test']);
    commands.push(compact(tests));
    const testReport = JSON.parse(readFileSync(join(dir, 'test-results/results.json')));
    const passedNames = testReport.testResults
      .flatMap((x) =>
        x.assertionResults.filter((y) => y.status === 'passed').map((y) => y.fullName),
      )
      .sort();
    // Vitest 1's aggregate JSON counter includes skipped assertions as passed.
    // Derive execution counts from individual assertion statuses, never totals.
    const assertions = testReport.testResults.flatMap((x) => x.assertionResults);
    const skipped = assertions.filter(
      (x) => x.status === 'skipped' || x.status === 'pending',
    ).length;
    if (dir === baseline) {
      assert.equal(passedNames.length, 411);
      assert.equal(skipped, 31);
      baselineNames = passedNames;
    }
    assert.ok(baselineNames, 'baseline anchor must execute first');
    const retained = baselineNames.every((name) => passedNames.includes(name));
    const gates = [['lint'], ['typecheck'], ['build:all']].map((args) =>
      run(dir, 'npm', ['run', ...args]),
    );
    commands.push(...gates.map(compact));
    itemWins = [
      tests.exit === 0 && retained && skipped === 31,
      ...gates.map((x) => x.exit === 0),
      http.results.filter((x) => x.suite === 'anchor').every((x) => x.passed),
    ];
    detail = {
      passed: passedNames.length,
      skipped,
      retainedExistingNames: retained,
      existingNamesSha256: hash(JSON.stringify(baselineNames)),
      http,
    };
  }
  const primary = itemWins.filter(Boolean).length / itemWins.length;
  const score = {
    primary,
    noopRate: 1 - primary,
    costPerWin: 0,
    regressed: suite.id === 'anchor' && itemWins.some((x) => !x),
    itemWins,
  };
  measurements.push({ source: policy.dependencies, suite: suite.id, score, detail, commands });
  assert.equal(lockHash(dir), policy.dependencies, 'evaluation must not rewrite lockfile');
  return score;
};
const fingerprint = fw.gateFingerprint(fw.meetsPromotionRule);
const result = await fw.runFlywheelGenerations({
  rootPolicy: { dependencies: rootHash },
  proposer: async () => ({
    value: fixedHash,
    summary: 'Same-major security floor and lockfile refresh; native proxy-trust regression fixed',
    inverse: { path: 'dependency-parent-files.json', parentBytes, hash: hash(parentBytes) },
  }),
  evaluator,
  holdout: { id: 'holdout', items: contract.holdout },
  anchor: { id: 'anchor', items: contract.anchor },
  maxGenerations: 1,
  signer: fw.makeSigner(),
  promotionRule: fw.meetsPromotionRule,
  dataSource: 'LIVE_NPM_AUDIT_NATIVE_LOOPBACK_HTTP_AND_REPOSITORY_COMMANDS',
});
const replay = fw.verifyReplayBundle(result.replayBundle, { pinnedGateFingerprint: fingerprint });
assert.equal(replay.pass, true);
assert.equal(result.promotions.length, 1);
writeFileSync(
  output,
  JSON.stringify(
    {
      contract,
      baselineLock: rootHash,
      candidateLock: fixedHash,
      fixtureSha256: hash(readFileSync(join(candidate, fixture))),
      measurements,
      promotions: result.promotions.length,
      fingerprint,
      replay,
      replayBundle: result.replayBundle,
      boundary:
        'component dependency security only; coverage, moderate advisories and Rust remain independent merge gates; cost/latency/energy unmeasured',
    },
    null,
    2,
  ) + '\n',
);
console.log(
  JSON.stringify(
    {
      promotions: result.promotions.length,
      replay,
      measurements: measurements.map((m) => ({
        suite: m.suite,
        score: m.score,
        passed: m.detail.passed,
        advisories: m.detail.audit?.metadata.vulnerabilities,
      })),
    },
    null,
    2,
  ),
);
