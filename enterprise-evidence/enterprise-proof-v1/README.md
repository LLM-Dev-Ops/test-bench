# Enterprise Proof v1 - Evidence Directory

**Purpose**: This directory contains benchmark execution artifacts for the canonical enterprise proof run.

**Status**: EVIDENCE STAGING AREA
**Manifest**: `/workspaces/test-bench/manifests/enterprise-proof-v1.yaml`
**Documentation**: `/workspaces/test-bench/docs/ENTERPRISE_PROOF_RUN.md`

---

## What Belongs Here

This directory stores **generated benchmark results** from executing the enterprise proof manifest. After running the benchmark, you will find:

```
enterprise-proof-v1/
├── README.md (this file)
└── enterprise-proof-v1-{timestamp}-{hash}/
    ├── fleet-results.json
    ├── fleet-results.yaml
    ├── csv/
    │   ├── fleet-summary.csv
    │   ├── repositories.csv
    │   ├── providers.csv
    │   └── categories.csv
    ├── executive-report.html
    └── llm-test-bench-core/
        ├── openai_gpt-4/
        │   ├── enterprise-coding/
        │   └── enterprise-reasoning/
        └── anthropic_claude-3-opus-20240229/
            ├── enterprise-coding/
            └── enterprise-reasoning/
```

---

## How This Directory Is Used

### During Sales Demonstrations

1. **Execute the benchmark**:
   ```bash
   llm-test-bench fleet manifests/enterprise-proof-v1.yaml
   ```

2. **Results appear here** automatically in a timestamped subdirectory

3. **Share these artifacts** with prospects:
   - `executive-report.html` - Visual dashboard for decision-makers
   - `csv/fleet-summary.csv` - Single-row summary for spreadsheets
   - `fleet-results.json` - Complete results for technical reviewers

### During Due Diligence

1. **Include in diligence package**:
   - The complete timestamped results directory
   - The source manifest (`manifests/enterprise-proof-v1.yaml`)
   - Documentation (`docs/ENTERPRISE_PROOF_RUN.md`)

2. **Reviewers can verify**:
   - Results are deterministic and reproducible
   - Methodology is transparent (manifest + docs)
   - Raw data is available (individual test responses)

3. **Reviewers can re-run**:
   - Using the same manifest file
   - Expecting similar results (within provider API variance)

### For Historical Comparison

Multiple runs of the same manifest can be stored here:

```
enterprise-proof-v1/
├── enterprise-proof-v1-20250101-abc123/  # Run 1
├── enterprise-proof-v1-20250115-def456/  # Run 2
├── enterprise-proof-v1-20250201-ghi789/  # Run 3
```

This enables:
- Trend analysis over time
- Provider performance comparison across dates
- Verification of reproducibility

---

## What Does NOT Belong Here

**Do NOT commit** to version control:
- ❌ Generated result directories (large, regenerable)
- ❌ Individual test response files (contain API outputs)
- ❌ HTML reports (generated from JSON)
- ❌ CSV files (generated from JSON)

**What IS committed** to version control:
- ✅ This README.md
- ✅ The manifest (`manifests/enterprise-proof-v1.yaml`)
- ✅ Documentation (`docs/ENTERPRISE_PROOF_RUN.md`)
- ✅ Source code and datasets

**Rationale**: Results are **generated artifacts**, not source code. The manifest is the source of truth. Anyone can regenerate results by executing the manifest.

---

## Git Configuration

This directory contains a `.gitignore` entry to prevent accidental commits of large result files:

```gitignore
# Ignore all generated result directories
enterprise-proof-v1-*/

# Keep the README
!README.md
```

If you need to **archive specific results** (e.g., for a critical customer demo), use:
- External storage (S3, Google Drive, Dropbox)
- Due diligence data rooms
- Encrypted archives

Do not bloat the git repository with regenerable artifacts.

---

## Evidence Packaging for Sales

When preparing evidence for a sales call or diligence review:

### Step 1: Execute Fresh Benchmark

```bash
cd /workspaces/test-bench
llm-test-bench fleet manifests/enterprise-proof-v1.yaml
```

Wait ~15 minutes for completion.

### Step 2: Identify Result Directory

The CLI will output:
```
Fleet Benchmark Complete
Run ID: enterprise-proof-v1-20250131-abc123
Artifacts: ./enterprise-evidence/enterprise-proof-v1/enterprise-proof-v1-20250131-abc123/
```

### Step 3: Package for Delivery

Create a ZIP or tarball:

```bash
cd enterprise-evidence/enterprise-proof-v1
tar -czf enterprise-proof-v1-20250131-abc123.tar.gz \
  enterprise-proof-v1-20250131-abc123/
```

Or for Windows-friendly ZIP:

```bash
zip -r enterprise-proof-v1-20250131-abc123.zip \
  enterprise-proof-v1-20250131-abc123/
```

### Step 4: Include Context

Package should contain:
1. **Results archive** (`enterprise-proof-v1-{timestamp}.tar.gz`)
2. **Manifest file** (`manifests/enterprise-proof-v1.yaml`)
3. **Documentation** (`docs/ENTERPRISE_PROOF_RUN.md`)
4. **Executive summary** (optional 1-page PDF)

### Step 5: Upload to Secure Location

- Sales team shared drive
- Customer-specific data room
- Encrypted email attachment (if small)
- S3 bucket with pre-signed URL (for large packages)

---

## Evidence Interpretation

Reviewers should:

1. **Read the documentation first** (`docs/ENTERPRISE_PROOF_RUN.md`)
2. **Review the manifest** to understand configuration
3. **Open the HTML dashboard** for visual summary
4. **Inspect the CSV summary** for quick metrics
5. **Examine raw JSON** if deep validation is needed

Key files for different audiences:

| Audience | Primary Artifacts |
|----------|------------------|
| Executive (CTO, VP) | `executive-report.html` |
| Procurement | `csv/fleet-summary.csv` |
| Architect | `fleet-results.json` |
| QA/Testing Lead | `llm-test-bench-core/*/` (raw responses) |
| Finance | Cost estimates in manifest + actual costs in summary |

---

## Reproducibility Notes

### What Should Be Identical Across Runs

- **Test count**: Always 160 tests (2 providers × 2 scenarios × 40 avg tests)
- **Dataset content**: Same prompts, same expected outputs
- **Provider models**: Same model versions (unless deprecated)
- **Metrics definitions**: Same formulas for success rate, p95, etc.

### What May Vary Slightly

- **Response content**: LLMs are stochastic even at temperature=0.0
- **Latency**: Provider API performance varies by time/region
- **Costs**: Provider pricing may change; estimates use published rates
- **Timestamps**: Obviously different for each run

### How to Verify Reproducibility

1. Run the benchmark twice on the same day
2. Compare `fleet-summary.csv` from both runs
3. Success rates should be within ±5%
4. Latency p95 should be within ±20%
5. Costs should be within ±10%

Larger variance suggests:
- Provider API changes
- Network/infrastructure issues
- Dataset or manifest corruption (verify git checksums)

---

## Maintenance

### Quarterly Review

Every quarter, the Technical Leadership team should:

1. **Execute a fresh benchmark run**
2. **Compare results to previous quarter**
3. **Update cost estimates** if provider pricing changed
4. **Verify provider model availability** (check for deprecations)
5. **Refresh documentation** if interpretation guidance needs updates

### When to Archive Old Runs

Delete result directories older than:
- **90 days** for routine demo runs
- **1 year** for quarterly baseline runs
- **Never** for runs included in closed deals (move to permanent archive)

### Archival Process

For critical runs that must be preserved:

1. Create archive: `tar -czf archive-{date}.tar.gz enterprise-proof-v1-{timestamp}/`
2. Upload to permanent storage (S3 Glacier, backup system)
3. Document in sales CRM which customer/deal this relates to
4. Delete local copy to save disk space

---

## Troubleshooting

### "Directory not found" error

**Cause**: Manifest specifies `base_dir: ./enterprise-evidence/enterprise-proof-v1`
but directory doesn't exist.

**Solution**: This directory should exist (contains this README). If missing:
```bash
mkdir -p enterprise-evidence/enterprise-proof-v1
```

### "Permission denied" error

**Cause**: Insufficient write permissions.

**Solution**:
```bash
chmod +w enterprise-evidence/enterprise-proof-v1
```

### Results not appearing

**Cause**: Benchmark failed before completion.

**Solution**:
1. Check CLI output for errors
2. Verify API keys are set: `echo $OPENAI_API_KEY`
3. Check network connectivity to provider APIs
4. Review logs in `./logs/` (if enabled)

### Disk space issues

**Cause**: Multiple large result directories accumulated.

**Solution**:
```bash
# Delete all but the most recent result
cd enterprise-evidence/enterprise-proof-v1
ls -t | tail -n +2 | xargs rm -rf
```

Or use the archival process above for selective retention.

---

## Support

For questions about:

- **Benchmark configuration**: See `docs/ENTERPRISE_PROOF_RUN.md`
- **Manifest syntax**: See `docs/FLEET_MANIFEST_SYSTEM.md`
- **Metrics definitions**: See `docs/FLEET_METRICS.md`
- **Sales packaging**: Contact Sales Engineering team
- **Due diligence requests**: Contact Legal/Compliance team

---

**Last Updated**: 2025-12-31
**Owner**: Sales Engineering & Technical Leadership
