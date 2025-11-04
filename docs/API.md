# API Server Documentation

The LLM Test Bench API Server provides comprehensive REST, GraphQL, and WebSocket APIs for interacting with the benchmarking system.

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Authentication](#authentication)
4. [REST API](#rest-api)
5. [GraphQL API](#graphql-api)
6. [WebSocket API](#websocket-api)
7. [Error Handling](#error-handling)
8. [Rate Limiting](#rate-limiting)
9. [Examples](#examples)

## Overview

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    API Server (Axum)                        │
│                                                             │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │   REST API     │  │  GraphQL API   │  │   WebSocket    │ │
│  │  (OpenAPI)     │  │  (async-gql)   │  │  (real-time)   │ │
│  └────────┬───────┘  └────────┬───────┘  └────────┬───────┘ │
│           │                   │                   │          │
│           └───────────────────┼───────────────────┘          │
│                               ▼                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Middleware Layer                         │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │  │
│  │  │  Auth   │ │  CORS   │ │  Rate   │ │ Logging │   │  │
│  │  │  (JWT)  │ │         │ │ Limit   │ │         │   │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
│                               ▼                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Core Business Logic                      │  │
│  │  Providers | Benchmarks | Evaluators | Plugins       │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Features

- **REST API**: OpenAPI-documented RESTful endpoints
- **GraphQL API**: Flexible querying with GraphiQL playground
- **WebSocket API**: Real-time updates and event streaming
- **Authentication**: JWT tokens and API keys with role-based access
- **Rate Limiting**: Per-user request throttling
- **CORS**: Configurable cross-origin support
- **Swagger UI**: Interactive API documentation
- **API Versioning**: Backward-compatible versioning

## Getting Started

### Starting the Server

```rust
use llm_test_bench_core::api::{ApiServer, ApiConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ApiConfig::default();
    let server = ApiServer::new(config);

    server.start().await?;
    Ok(())
}
```

### Configuration

```rust
use llm_test_bench_core::api::{ApiConfig, CorsConfig};

let config = ApiConfig::builder()
    .bind_address("0.0.0.0:3000".parse()?)
    .enable_rest(true)
    .enable_graphql(true)
    .enable_websocket(true)
    .jwt_secret(std::env::var("JWT_SECRET")?)
    .jwt_expiration(3600)
    .rate_limit(100, 50)  // 100 rps, burst of 50
    .cors(CorsConfig {
        allowed_origins: vec!["https://myapp.com".to_string()],
        ..Default::default()
    })
    .build();
```

### Environment Variables

```bash
JWT_SECRET=your-secret-key-change-in-production
BIND_ADDRESS=0.0.0.0:3000
ENABLE_SWAGGER=true
CORS_ORIGINS=https://myapp.com,https://staging.myapp.com
```

## Authentication

### User Roles

- **Admin**: Full access to all endpoints
- **User**: Standard access to most endpoints
- **Viewer**: Read-only access
- **Service**: API-to-API access

### JWT Authentication

#### Obtaining a Token

```bash
curl -X POST http://localhost:3000/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user@example.com",
    "password": "password"
  }'
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2024-03-20T15:30:00Z"
}
```

#### Using the Token

```bash
curl -X GET http://localhost:3000/v1/benchmarks \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### API Key Authentication

#### Creating an API Key

```bash
curl -X POST http://localhost:3000/v1/auth/api-keys \
  -H "Authorization: Bearer <admin-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My API Key",
    "role": "service"
  }'
```

Response:
```json
{
  "key": "ltb_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2024-03-20T10:00:00Z"
}
```

#### Using the API Key

```bash
curl -X GET http://localhost:3000/v1/benchmarks \
  -H "X-API-Key: ltb_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
```

## REST API

### Base URL

```
http://localhost:3000/v1
```

### Endpoints

#### Health Check

```http
GET /health
```

Response:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2024-03-20T10:00:00Z",
  "uptime_seconds": 3600
}
```

#### Create Completion

```http
POST /v1/completions
```

Request:
```json
{
  "provider": "openai",
  "model": "gpt-4",
  "prompt": "What is the capital of France?",
  "max_tokens": 100,
  "temperature": 0.7
}
```

Response:
```json
{
  "text": "The capital of France is Paris.",
  "model": "gpt-4",
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 8,
    "total_tokens": 18
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2024-03-20T10:00:00Z"
}
```

#### Create Evaluation

```http
POST /v1/evaluations
```

Request:
```json
{
  "provider": "openai",
  "model": "gpt-4",
  "input": "What is 2+2?",
  "output": "The answer is 4.",
  "expected": "4",
  "metrics": ["faithfulness", "relevance", "coherence"]
}
```

Response:
```json
{
  "score": 0.95,
  "metrics": {
    "faithfulness": 1.0,
    "relevance": 0.9,
    "coherence": 0.95
  },
  "evaluation_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2024-03-20T10:00:00Z"
}
```

#### Create Benchmark

```http
POST /v1/benchmarks
```

Request:
```json
{
  "name": "GPT-4 vs Claude Performance",
  "providers": [
    {
      "provider": "openai",
      "model": "gpt-4"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-opus-20240229"
    }
  ],
  "dataset": "mmlu",
  "metrics": ["accuracy", "latency", "cost"],
  "iterations": 100
}
```

Response:
```json
{
  "benchmark_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "progress": 0.0,
  "results": null
}
```

#### Get Benchmark

```http
GET /v1/benchmarks/:id
```

Response:
```json
{
  "benchmark_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "progress": 0.45,
  "results": null
}
```

#### List Plugins

```http
GET /v1/plugins?page=1&page_size=20
```

Response:
```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "custom-evaluator",
      "version": "1.0.0",
      "plugin_type": "evaluator",
      "status": "ready"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

#### Execute Plugin

```http
POST /v1/plugins/:id/execute
```

Request:
```json
{
  "input": {
    "text": "Sample input",
    "parameters": {
      "threshold": 0.8
    }
  }
}
```

Response:
```json
{
  "output": {
    "score": 0.92,
    "details": "Evaluation completed successfully"
  },
  "execution_time_ms": 150,
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Pagination

Most list endpoints support pagination:

```http
GET /v1/benchmarks?page=2&page_size=50
```

Parameters:
- `page`: Page number (default: 1)
- `page_size`: Items per page (default: 20, max: 100)

## GraphQL API

### Endpoint

```
http://localhost:3000/graphql
```

### GraphiQL Playground

Open `http://localhost:3000/graphql` in a browser to access the interactive GraphiQL playground.

### Schema

#### Queries

```graphql
type Query {
  # Get API version
  version: String!

  # Health check
  health: Boolean!

  # List plugins
  plugins: [PluginNode!]!

  # Get plugin by ID
  plugin(id: String!): PluginNode
}
```

#### Mutations

```graphql
type Mutation {
  # Create completion
  createCompletion(input: CompletionInput!): CompletionResult!

  # Create evaluation
  createEvaluation(input: EvaluationInput!): EvaluationResult!
}
```

#### Types

```graphql
input CompletionInput {
  provider: String!
  model: String!
  prompt: String!
  maxTokens: Int
  temperature: Float
}

type CompletionResult {
  text: String!
  model: String!
  requestId: String!
  createdAt: DateTime!
}

input EvaluationInput {
  provider: String!
  model: String!
  input: String!
  output: String!
  expected: String
  metrics: [String!]!
}

type EvaluationResult {
  score: Float!
  evaluationId: String!
  createdAt: DateTime!
}

type PluginNode {
  id: String!
  name: String!
  version: String!
  pluginType: String!
  status: String!
}
```

### Example Queries

#### Get Version and Health

```graphql
query {
  version
  health
}
```

#### List Plugins

```graphql
query {
  plugins {
    id
    name
    version
    pluginType
    status
  }
}
```

#### Create Completion

```graphql
mutation {
  createCompletion(input: {
    provider: "openai"
    model: "gpt-4"
    prompt: "What is the capital of France?"
    maxTokens: 100
  }) {
    text
    model
    requestId
    createdAt
  }
}
```

#### Create Evaluation

```graphql
mutation {
  createEvaluation(input: {
    provider: "openai"
    model: "gpt-4"
    input: "What is 2+2?"
    output: "The answer is 4."
    expected: "4"
    metrics: ["faithfulness", "relevance"]
  }) {
    score
    evaluationId
    createdAt
  }
}
```

## WebSocket API

### Endpoint

```
ws://localhost:3000/ws
```

### Message Format

All WebSocket messages are JSON with a `type` field:

```json
{
  "type": "message_type",
  ...
}
```

### Message Types

#### Subscribe

Subscribe to event topics:

```json
{
  "type": "subscribe",
  "topics": ["benchmark.progress", "evaluation.results"]
}
```

#### Unsubscribe

Unsubscribe from topics:

```json
{
  "type": "unsubscribe",
  "topics": ["benchmark.progress"]
}
```

#### Event

Server sends events:

```json
{
  "type": "event",
  "topic": "benchmark.progress",
  "data": {
    "benchmark_id": "550e8400-e29b-41d4-a716-446655440000",
    "progress": 0.45,
    "status": "running"
  },
  "timestamp": 1710936000000
}
```

#### Ping/Pong

Keep-alive messages:

```json
{
  "type": "ping",
  "timestamp": 1710936000000
}
```

```json
{
  "type": "pong",
  "timestamp": 1710936000000
}
```

#### Ack

Server acknowledgment:

```json
{
  "type": "ack",
  "request_id": "req_123",
  "message": "Subscribed to 2 topics"
}
```

#### Error

Server error:

```json
{
  "type": "error",
  "code": "INVALID_MESSAGE",
  "message": "Failed to parse message"
}
```

### Available Topics

- `benchmark.progress` - Benchmark execution updates
- `evaluation.results` - Evaluation completions
- `system.metrics` - System performance metrics
- `plugin.events` - Plugin lifecycle events
- `request.logs` - API request logs

### Example (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
  console.log('Connected');

  // Subscribe to topics
  ws.send(JSON.stringify({
    type: 'subscribe',
    topics: ['benchmark.progress']
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  if (msg.type === 'event') {
    console.log(`Event on ${msg.topic}:`, msg.data);
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

### Example (CLI with wscat)

```bash
# Install wscat
npm install -g wscat

# Connect
wscat -c ws://localhost:3000/ws

# Subscribe
> {"type": "subscribe", "topics": ["benchmark.progress"]}

# Receive events
< {"type":"event","topic":"benchmark.progress","data":{...},"timestamp":1710936000000}
```

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "additional context"
    }
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-03-20T10:00:00Z"
}
```

### HTTP Status Codes

- `200` - Success
- `201` - Created
- `400` - Bad Request (invalid input)
- `401` - Unauthorized (missing/invalid auth)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `409` - Conflict (resource already exists)
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error
- `503` - Service Unavailable

### Error Codes

- `BAD_REQUEST` - Invalid request parameters
- `UNAUTHORIZED` - Authentication required
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `CONFLICT` - Resource conflict
- `RATE_LIMIT_EXCEEDED` - Too many requests
- `INTERNAL_ERROR` - Server error
- `SERVICE_UNAVAILABLE` - Service temporarily unavailable

## Rate Limiting

### Limits

Default rate limits per user:
- 100 requests per second
- Burst of 50 requests

### Headers

Rate limit information is returned in response headers:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1710936060
```

### Rate Limit Exceeded

When rate limit is exceeded, you'll receive a `429` response:

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again in 5 seconds.",
    "retry_after": 5
  }
}
```

## Examples

### Complete REST Workflow

```bash
# 1. Create completion
COMPLETION=$(curl -s -X POST http://localhost:3000/v1/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "provider": "openai",
    "model": "gpt-4",
    "prompt": "What is AI?",
    "max_tokens": 100
  }')

echo $COMPLETION | jq .

# 2. Create evaluation
EVAL=$(curl -s -X POST http://localhost:3000/v1/evaluations \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"provider\": \"openai\",
    \"model\": \"gpt-4\",
    \"input\": \"What is AI?\",
    \"output\": $(echo $COMPLETION | jq -r .text),
    \"metrics\": [\"coherence\", \"relevance\"]
  }")

echo $EVAL | jq .

# 3. Create benchmark
BENCHMARK=$(curl -s -X POST http://localhost:3000/v1/benchmarks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Model Comparison",
    "providers": [
      {"provider": "openai", "model": "gpt-4"},
      {"provider": "anthropic", "model": "claude-3-opus-20240229"}
    ],
    "dataset": "mmlu",
    "metrics": ["accuracy"],
    "iterations": 10
  }')

BENCHMARK_ID=$(echo $BENCHMARK | jq -r .benchmark_id)

# 4. Poll for completion
while true; do
  STATUS=$(curl -s http://localhost:3000/v1/benchmarks/$BENCHMARK_ID \
    -H "Authorization: Bearer $TOKEN")

  PROGRESS=$(echo $STATUS | jq -r .progress)
  echo "Progress: $PROGRESS"

  if [ $(echo "$PROGRESS >= 1.0" | bc) -eq 1 ]; then
    break
  fi

  sleep 2
done

echo "Benchmark complete!"
echo $STATUS | jq .
```

### GraphQL Client (Python)

```python
import requests
import json

GRAPHQL_URL = "http://localhost:3000/graphql"
TOKEN = "your-jwt-token"

def graphql_query(query, variables=None):
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {TOKEN}"
    }

    payload = {"query": query}
    if variables:
        payload["variables"] = variables

    response = requests.post(
        GRAPHQL_URL,
        headers=headers,
        json=payload
    )

    return response.json()

# Query example
result = graphql_query("""
    query {
        version
        health
        plugins {
            id
            name
            version
        }
    }
""")

print(json.dumps(result, indent=2))

# Mutation example
result = graphql_query("""
    mutation($input: CompletionInput!) {
        createCompletion(input: $input) {
            text
            model
            requestId
        }
    }
""", {
    "input": {
        "provider": "openai",
        "model": "gpt-4",
        "prompt": "Hello, world!",
        "maxTokens": 50
    }
})

print(json.dumps(result, indent=2))
```

### WebSocket Client (Python)

```python
import asyncio
import websockets
import json

async def websocket_client():
    uri = "ws://localhost:3000/ws"

    async with websockets.connect(uri) as websocket:
        # Subscribe to topics
        await websocket.send(json.dumps({
            "type": "subscribe",
            "topics": ["benchmark.progress", "evaluation.results"]
        }))

        # Receive messages
        async for message in websocket:
            data = json.loads(message)

            if data["type"] == "event":
                topic = data["topic"]
                event_data = data["data"]
                print(f"Event on {topic}: {event_data}")
            elif data["type"] == "ack":
                print(f"Ack: {data['message']}")

asyncio.run(websocket_client())
```

## OpenAPI Documentation

The complete OpenAPI specification is available at:

```
http://localhost:3000/swagger-ui
```

You can also access the raw OpenAPI JSON:

```
http://localhost:3000/api-docs/openapi.json
```

## Support

For issues, questions, or contributions, please visit:
- GitHub: https://github.com/yourusername/llm-test-bench
- Documentation: https://docs.llm-test-bench.dev

## License

Licensed under Apache 2.0 or MIT license.
