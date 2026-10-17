# Test-bench dependency-security cycle — October 17, 2026

**EXECUTED / component VALIDATED / repository BLOCKED — not merged.**

Resumed [existing non-draft PR #24](https://github.com/LLM-Dev-Ops/test-bench/pull/24). Baseline
recovery head: `1c4f445288f360120ebae6fb00c6fbf312c16931`. Implementation:
`a97c8aface1480232fa24b36524f798979617253`, tree `c2e6119165368b10ae40a0891d0112e5e44204df`. Main
stayed at `4d58c46426fe085d25fef3fcb35fa5e2cce83b30`. The evidence-only follow-up changes no SDK
dependency, test or CI source. No conflict resolution, force push or merge was required/performed.

## What changed

- Raise same-major minimums for AJV, Express, glob and YAML; refresh the compatible locked graph.
  All changed dependency majors stayed unchanged. Resolved versions include AJV 8.20.0, Express
  4.22.3, glob 10.5.0, YAML 2.9.1, and proxy-addr 2.0.8.
- Reproduce and prevent the
  [proxy-addr trust-subnet spoofing defect](https://github.com/advisories/GHSA-jqcg-44mw-7w3h) with
  five real loopback HTTP cases against the installed Express graph. The vulnerable baseline
  accepted a forged `X-Forwarded-For` address with `::ffff:10.0.0.0/8`; the patched graph returns
  the actual peer. Four ordinary trust configurations retain their behavior. This is a
  dependency-level fixture, **not evidence that a production deployment enables that trust
  configuration or was exploited**.
- Add a pinned Ubuntu 24.04 / Node 20.19.5 production-security job using immutable
  checkout/setup-node action SHAs, real HTTP regression execution, production audit and lockfile
  checksum/diff verification.
- Make `CI Success` depend on integration and production security as well as the existing gates, and
  reject failures, cancellations **and skips**. Twenty-six tests exercise the aggregate's actual
  result expression/dependency list. These are unit gate checks, not simulated hosted execution
  proof.

## Actual measurements

The [contract](contract.json) was frozen before dependency changes. The native HTTP fixture is
byte-identical in baseline/candidate; no existing test, coverage threshold or exclusion was
weakened. The Flywheel anchor checks every one of the original 411 passing assertion names remains
passing.

| Measurement                       |        Baseline |         Candidate |
| --------------------------------- | --------------: | ----------------: |
| Production advisories             |              14 |                 3 |
| Production critical / high        |           1 / 6 |             0 / 0 |
| Production moderate / low         |           6 / 1 |             3 / 0 |
| Frozen holdout                    |             0/3 |               3/3 |
| Frozen anchors                    |             5/5 |               5/5 |
| Native HTTP spoof-resistance case | fail, forged IP | pass, actual peer |
| Native HTTP compatibility cases   |             4/4 |               4/4 |
| Unit tests actually passed        |             411 |               438 |
| Pre-existing skips                |              31 |                31 |
| ESLint / typecheck / build-all    |            pass |              pass |

Candidate `npm ci --ignore-scripts`, `npm test`, `npm run lint`, `npm run typecheck`,
`npm run build:all` and `npm run format:check` exited **0**, both locally and on an exact
implementation-commit RuOS checkout (Node 20.20.2). Its status was clean and lock checksum
unchanged. `npm audit --omit=dev --audit-level=high` exited **0**; full production audit exited
**1** because three moderate findings remain. Do not label this a zero-advisory result.

The **all-dependencies** audit, including development tools, exited **1** with **29 findings: four
critical, 19 high, six moderate**. This is separate from the production-only improvement. The
suggested `npm audit fix --force` was not executed: its proposed Functions Framework 3→2 downgrade
violates the frozen compatibility boundary. Production Functions Framework/CloudEvents/uuid
remediation and development-tool advisories remain work.

The **unchanged 80% coverage gate remains red**. The initial local baseline reported 42.46% lines,
52.84% functions, 69.4% branches; the clean implementation-commit hosted job reported 42.11% lines,
54.98% functions, 69.54% branches. Local generated tooling/assets and new fixture files alter the
broad `all: true` population, so these are not a controlled coverage-lift measurement. No coverage
improvement is claimed, no exclusions were added, and full-repository acceptance is rejected. The
final evidence checkout receives its own gate run rather than inheriting an earlier CI conclusion.

## Hosted evidence and Rust limits

[Implementation CI run 39056681265](https://github.com/LLM-Dev-Ops/test-bench/actions/runs/39056681265)
completed:

- PASS: Node 18, 20 and 22 (**438 actual passes + 31 skips each**), formatting, lint, typecheck,
  build, integration, production security.
- FAIL: coverage and, consequently, `CI Success`. The new security job verified the committed
  lockfile stayed unchanged.

RuOS bootstrap was executed again; OpenSSL **3.0.13** was verified. The durable research-lab
checkout was inspected at `6454d374a939f2b5f94709b6909f4251b7204252` against upstream main
`83c4850c04d6243c1f700ec4a6a0e406a7d7198a` before using its recovery tooling.

`cargo test --workspace --locked` began real compilation but consumed remaining desktop disk. Its
owned process was stopped (exit **143**, not a compiler/test verdict). Only this run's generated
`/home/ruv/work/test-bench-oct17c/target` artifacts were cleaned: 3579 files / 2.2 GiB; source and
logs were retained and artifacts can be rebuilt. A retry with
`timeout 240s env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo test --workspace --locked`
exited **124**, still compiling, with **zero completed Rust tests**. `cargo fmt --all -- --check`
exited **1**. Separate Rust build/Clippy were deferred, not reported passing. Rust sources,
manifests and lockfile are unchanged by this cycle. No missing-OpenSSL or current
private-authentication failure is inferred from the timeout.

## Real tool execution and reproducibility

Pinned versions: **Ruflo 3.56.3**, **Transformers 4.3.1**, **MetaHarness 0.4.17**, **Darwin
0.10.3**, **Flywheel 0.1.12**. Upstream refreshed revision:
`e0dfd44da72b7adfb7c57fdd42d124be58ed7086`. Local isolated tools:
`/workspace/scratch/4f25900b71ff/cycle-tools-20261017`.

- Ruflo initialized hierarchical/specialized coordination, max-agents 1, swarm
  `swarm-1792259929105-ypiqot`, actual task `test-bench-security-oct17c`. No delegated workers are
  claimed. Actual implementation is the dependency/test/CI change above, not the coordination
  invocation.
- `metaharness score . --json`: **82→82**, hard constraints **6/6**, exit 0. Static compile
  confidence is not Rust validation.
- `darwin evolve . --generations 1 --children 2 --concurrency 1 --seed 2026101714 --sandbox real --selection pareto`:
  baseline, `g1_v0` (toolPolicy), `g1_v1` (memoryPolicy) tied **0.985**. Nine real `npm test`
  invocations passed, each at the earlier 412-pass/31-skip stage before aggregate tests were added.
  Both children were rejected: **NO_IMPROVEMENT**. This substrate cannot discriminate harness
  surfaces; initialization is not improvement proof.
- `node docs/metaharness/2026-10-17/evaluate.mjs ../test-bench-baseline-oct17c . ../cycle-tools-20261017 docs/metaharness/2026-10-17/flywheel-results.json`:
  exit **0**. Flywheel actually reran audits, native HTTP and repository commands, promoted the
  **component only** from 0→1, retained anchors 1→1, and passed **8/8** signed replay checks. The
  first attempt failed closed because Vitest 1's aggregate JSON called all 442 assertions passed,
  although individual statuses were 411 passed/31 skipped. The corrected parser uses individual
  statuses, preserves the frozen criteria, and reran the evaluator; no score from the rejected
  attempt is used.

Gate fingerprint: `c5942e7c9bf8f28fe7af47a6ec3887d11eeb3f8c8e8601cc6a7a54969ea0b981`. Lineage:
`gen1(dependencies) → gen0(root)`. See [compact results](flywheel-summary.json) and
[complete gzip-compressed JSON](flywheel-results.json.gz), including executed output, signed
receipts, lineage and exact parent dependency bytes. Independent replay verified the compressed file
after decompression. Generate plain JSON again with the evaluator, or decode with
`gzip -dc flywheel-results.json.gz`. No speed, paid-cost, energy or LLM efficacy improvement is
claimed; HTTP durations are observations, not a load benchmark.

Offline post-task learning used the current research-lab kernel-restricted launcher/preload; both
blob hashes matched its upstream main. Checksum-pinned MiniLM revision
`751bff37182d3f1213fa05d7196b954e230abad9` generated real **384-dimensional** embeddings with remote
model loading disabled. `learningUpdates.available`, `outcomePersisted`, `feedback.recorded`, and
`trajectory.recorded` were **all true**, with two learning/feedback updates. The task outcome was
explicitly `success=false` because whole-repository gates remain blocked. No active pheromone swarm,
hash embeddings, task-data egress or approval workaround is claimed. Exact invocations and outcomes
are in the ledger.

## Coverage, architecture and rollback

Fully paginated **58 accessible nonarchived repositories** (offsets 0/100 → 58/0), refreshed each
main/latest CI, and paginated **117 open PRs** (100+17). Deep-worked only test-bench; observatory
#11, sentinel #22 and inference-gateway #1 received read-only PR inspection. All **57 named
deferrals**, SHAs, CI states and reasons are in the [ledger](ledger.json); metadata coverage is not
a claim of source/build coverage for every repository.

Core-memory returned no playbook entries or anti-patterns and marked the response unscreened.
Explicit user constraints governed the implementation: Rust/TypeScript retained; no persistence,
service database, IAM, deployment, publication or paid-provisioning changes. Existing CI/test
constraints remain intact or stricter. No branch protection was bypassed.

Rollback: revert this cycle's evidence commit, then `a97c8aface1480232fa24b36524f798979617253`, to
return to the prior recovery head while retaining its earlier fixes. After a future squash merge,
revert that merge commit. The signed inverse retains exact parent package/lock bytes and hashes;
reverse-patch applicability passed for package and CI changes. No production rollback or data
migration was performed.
