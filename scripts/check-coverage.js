#!/usr/bin/env node

/**
 * Check if code coverage meets the required thresholds
 * Used in CI/CD pipeline
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const COVERAGE_THRESHOLDS = {
  phase1: {
    lines: 80,
    functions: 80,
    branches: 80,
    statements: 80,
  },
  phase5: {
    lines: 90,
    functions: 90,
    branches: 90,
    statements: 90,
  },
};

// Read coverage summary
const coveragePath = path.join(__dirname, '../coverage/coverage-summary.json');

if (!fs.existsSync(coveragePath)) {
  console.error('❌ Coverage summary not found. Run tests with coverage first.');
  process.exit(1);
}

const coverage = JSON.parse(fs.readFileSync(coveragePath, 'utf-8'));
const total = coverage.total;

// Determine current phase (default to phase1)
const currentPhase = process.env.COVERAGE_PHASE || 'phase1';
const thresholds = COVERAGE_THRESHOLDS[currentPhase];

console.log(`\n📊 Coverage Report (${currentPhase.toUpperCase()}):\n`);
console.log(`Lines:      ${total.lines.pct.toFixed(2)}% (threshold: ${thresholds.lines}%)`);
console.log(`Functions:  ${total.functions.pct.toFixed(2)}% (threshold: ${thresholds.functions}%)`);
console.log(`Branches:   ${total.branches.pct.toFixed(2)}% (threshold: ${thresholds.branches}%)`);
console.log(`Statements: ${total.statements.pct.toFixed(2)}% (threshold: ${thresholds.statements}%)\n`);

// Check thresholds
const failures = [];

if (total.lines.pct < thresholds.lines) {
  failures.push(`Lines: ${total.lines.pct.toFixed(2)}% < ${thresholds.lines}%`);
}
if (total.functions.pct < thresholds.functions) {
  failures.push(`Functions: ${total.functions.pct.toFixed(2)}% < ${thresholds.functions}%`);
}
if (total.branches.pct < thresholds.branches) {
  failures.push(`Branches: ${total.branches.pct.toFixed(2)}% < ${thresholds.branches}%`);
}
if (total.statements.pct < thresholds.statements) {
  failures.push(`Statements: ${total.statements.pct.toFixed(2)}% < ${thresholds.statements}%`);
}

if (failures.length > 0) {
  console.error('❌ Coverage thresholds not met:\n');
  failures.forEach((failure) => console.error(`  - ${failure}`));
  console.error('\n');
  process.exit(1);
} else {
  console.log('✅ All coverage thresholds met!\n');
  process.exit(0);
}
