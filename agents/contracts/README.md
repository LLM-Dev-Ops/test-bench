# Agentics Contracts

This directory contains canonical schemas and agent registry for the Agentics Dev platform.

## Structure

- `schemas/` - Zod schemas for all agent contracts
- `registry/` - Agent registration metadata
- `types/` - TypeScript type exports derived from schemas

## Usage

All agents MUST import schemas exclusively from this directory.

```typescript
import { BenchmarkRunnerInputSchema, DecisionEventSchema } from '@agents/contracts';
```

## Contract Rules

1. All inputs and outputs MUST be validated against these schemas
2. Schemas are versioned - breaking changes require version bumps
3. DecisionEvents MUST conform to the base schema
4. All agents emit exactly ONE DecisionEvent per invocation
