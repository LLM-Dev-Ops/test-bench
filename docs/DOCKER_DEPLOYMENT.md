# Docker Deployment Guide

## Quick Start

### Build the Docker image

```bash
docker build -t llm-test-bench:latest .
```

### Run with Docker Compose

```bash
# Set API keys in .env file
echo "OPENAI_API_KEY=your_key_here" > .env
echo "ANTHROPIC_API_KEY=your_key_here" >> .env

# Run benchmark
docker-compose run llm-test-bench bench \
  --dataset /data/datasets/coding-tasks.json \
  --providers openai \
  --metrics faithfulness,relevance \
  --output /data/results
```

## Docker Image Details

### Image Size
- **Builder stage**: ~2.5 GB (includes Rust toolchain)
- **Runtime image**: ~150 MB (Debian slim + binary)

### Security Features
- ✅ Non-root user (uid 1000)
- ✅ Minimal runtime dependencies
- ✅ No unnecessary packages
- ✅ CA certificates included
- ✅ Read-only filesystem support

### Multi-Architecture Support
Build for multiple platforms:

```bash
docker buildx create --use
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t llm-test-bench:latest \
  --push .
```

## Usage Examples

### 1. Run Benchmark

```bash
docker run --rm \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  -v $(pwd)/datasets:/data/datasets:ro \
  -v $(pwd)/results:/data/results:rw \
  llm-test-bench:latest \
  bench \
    --dataset /data/datasets/coding-tasks.json \
    --providers openai \
    --metrics faithfulness,relevance \
    --output /data/results
```

### 2. Compare Models

```bash
docker run --rm \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
  -v $(pwd)/results:/data/results:rw \
  llm-test-bench:latest \
  compare \
    --prompt "Explain quantum entanglement" \
    --models openai:gpt-4,anthropic:claude-3-opus \
    --statistical-tests \
    --output /data/results/comparison.html
```

### 3. Generate Dashboard

```bash
docker run --rm \
  -v $(pwd)/results:/data/results:rw \
  llm-test-bench:latest \
  dashboard \
    --results /data/results/*.json \
    --theme dark \
    --output /data/results/dashboard.html
```

### 4. Analyze Performance

```bash
docker run --rm \
  -v $(pwd)/results:/data/results:ro \
  llm-test-bench:latest \
  analyze \
    --baseline /data/results/baseline.json \
    --comparison /data/results/latest.json \
    --fail-on-regression
```

### 5. Optimize Costs

```bash
docker run --rm \
  -v $(pwd)/results:/data/results:ro \
  llm-test-bench:latest \
  optimize \
    --current-model gpt-4 \
    --monthly-requests 100000 \
    --quality-threshold 0.80
```

## Docker Compose

### Configuration

Edit `docker-compose.yml` to customize:
- API keys (set in `.env` file)
- Volume mounts
- Resource limits
- Command overrides

### Common Operations

```bash
# Build
docker-compose build

# Run benchmark
docker-compose run llm-test-bench bench --dataset /data/datasets/test.json

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Remove volumes
docker-compose down -v
```

## Kubernetes Deployment

### Create Secret

```bash
kubectl create secret generic llm-api-keys \
  --from-literal=openai-key=$OPENAI_API_KEY \
  --from-literal=anthropic-key=$ANTHROPIC_API_KEY
```

### Job Example

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: llm-benchmark
spec:
  template:
    spec:
      containers:
      - name: llm-test-bench
        image: llm-test-bench:latest
        command:
          - llm-test-bench
          - bench
          - --dataset
          - /data/datasets/coding-tasks.json
          - --providers
          - openai,anthropic
          - --metrics
          - faithfulness,relevance
          - --output
          - /data/results
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: openai-key
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: anthropic-key
        volumeMounts:
        - name: datasets
          mountPath: /data/datasets
          readOnly: true
        - name: results
          mountPath: /data/results
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
      volumes:
      - name: datasets
        configMap:
          name: llm-datasets
      - name: results
        persistentVolumeClaim:
          claimName: llm-results
      restartPolicy: Never
  backoffLimit: 3
```

### CronJob Example

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: daily-benchmark
spec:
  schedule: "0 0 * * *"  # Daily at midnight
  jobTemplate:
    spec:
      template:
        spec:
          # Same as Job above
```

## CI/CD Integration

### GitHub Actions

See `.github/workflows/llm-benchmark.yml` for complete example.

Key features:
- Daily scheduled benchmarks
- Regression detection
- Artifact upload
- PR comments with results

### GitLab CI

```yaml
benchmark:
  stage: test
  image: llm-test-bench:latest
  script:
    - llm-test-bench bench
        --dataset datasets/coding-tasks.json
        --providers openai
        --metrics faithfulness,relevance
        --output results
  artifacts:
    paths:
      - results/
    expire_in: 90 days
  only:
    - schedules
```

## Performance Tuning

### Resource Limits

Recommended limits:
- **CPU**: 0.5-2.0 cores
- **Memory**: 512MB-2GB
- **Disk**: 1GB for cache

### Caching

Enable evaluation caching to reduce costs:

```yaml
environment:
  - LLM_TEST_BENCH_EVALUATION__CACHE_ENABLED=true
volumes:
  - llm-cache:/data/cache:rw
```

Cache can reduce API costs by 80%+.

### Concurrency

Control parallel execution:

```yaml
environment:
  - LLM_TEST_BENCH_ORCHESTRATION__MAX_PARALLEL_MODELS=5
```

## Troubleshooting

### Image won't build

```bash
# Clear build cache
docker builder prune -a

# Build with no cache
docker build --no-cache -t llm-test-bench:latest .
```

### Permission errors

```bash
# Ensure volumes have correct permissions
chmod -R 755 results/
chown -R 1000:1000 results/
```

### API key errors

```bash
# Verify keys are set
docker run --rm llm-test-bench:latest env | grep API_KEY

# Test with simple command
docker run --rm \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  llm-test-bench:latest \
  config show
```

### Out of memory

```bash
# Increase memory limit
docker run --memory=2g llm-test-bench:latest ...

# Or in docker-compose.yml:
deploy:
  resources:
    limits:
      memory: 2G
```

## Security Best Practices

1. **Never commit API keys** - Use environment variables or secrets
2. **Use non-root user** - Already configured in Dockerfile
3. **Read-only volumes** - Mount datasets as read-only
4. **Resource limits** - Set CPU and memory limits
5. **Network isolation** - Use Docker networks
6. **Image scanning** - Scan for vulnerabilities regularly

```bash
# Scan with Trivy
trivy image llm-test-bench:latest
```

## Monitoring

### Health Checks

Add to docker-compose.yml:

```yaml
healthcheck:
  test: ["CMD", "llm-test-bench", "--version"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

### Logging

```bash
# View logs
docker logs llm-test-bench

# Follow logs
docker logs -f llm-test-bench

# Export logs
docker logs llm-test-bench > llm-bench.log 2>&1
```

## Production Deployment Checklist

- [ ] API keys stored in secure secrets manager
- [ ] Resource limits configured
- [ ] Health checks enabled
- [ ] Logging configured
- [ ] Monitoring alerts set up
- [ ] Backup strategy for results
- [ ] Cache persistence configured
- [ ] Image vulnerability scan passed
- [ ] Network security configured
- [ ] Documentation updated

## Support

For issues or questions:
- GitHub Issues: https://github.com/your-org/llm-test-bench/issues
- Documentation: https://docs.llm-test-bench.io
