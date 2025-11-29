/**
 * Unit tests for benchmarking functionality
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Benchmark Runner', () => {
  describe('Benchmark Configuration', () => {
    it('should validate benchmark config', () => {
      const config = {
        name: 'Test Benchmark',
        provider: 'openai',
        model: 'gpt-4',
        prompts: ['Hello', 'How are you?'],
        iterations: 1,
        parallel: false
      };

      expect(config.name).toBeTruthy();
      expect(config.provider).toBeTruthy();
      expect(config.model).toBeTruthy();
      expect(config.prompts.length).toBeGreaterThan(0);
      expect(config.iterations).toBeGreaterThan(0);
      expect(typeof config.parallel).toBe('boolean');
    });

    it('should require minimum iterations', () => {
      const validIterations = [1, 5, 10, 100];
      const invalidIterations = [0, -1, -5];

      validIterations.forEach(n => {
        expect(n).toBeGreaterThan(0);
      });

      invalidIterations.forEach(n => {
        expect(n).not.toBeGreaterThan(0);
      });
    });

    it('should support batch processing', () => {
      const batchConfig = {
        batch_size: 10,
        max_concurrent: 3,
        timeout_seconds: 60
      };

      expect(batchConfig.batch_size).toBeGreaterThan(0);
      expect(batchConfig.max_concurrent).toBeGreaterThan(0);
      expect(batchConfig.max_concurrent).toBeLessThanOrEqual(batchConfig.batch_size);
    });
  });

  describe('Performance Metrics', () => {
    it('should track latency metrics', () => {
      interface LatencyMetrics {
        min: number;
        max: number;
        mean: number;
        median: number;
        p95: number;
        p99: number;
      }

      const calculateLatencyMetrics = (latencies: number[]): LatencyMetrics => {
        const sorted = [...latencies].sort((a, b) => a - b);
        const sum = sorted.reduce((a, b) => a + b, 0);

        return {
          min: sorted[0],
          max: sorted[sorted.length - 1],
          mean: sum / sorted.length,
          median: sorted[Math.floor(sorted.length / 2)],
          p95: sorted[Math.floor(sorted.length * 0.95)],
          p99: sorted[Math.floor(sorted.length * 0.99)]
        };
      };

      const latencies = [100, 150, 120, 180, 110, 200, 130];
      const metrics = calculateLatencyMetrics(latencies);

      expect(metrics.min).toBeLessThanOrEqual(metrics.mean);
      expect(metrics.max).toBeGreaterThanOrEqual(metrics.mean);
      expect(metrics.p95).toBeGreaterThanOrEqual(metrics.median);
      expect(metrics.p99).toBeGreaterThanOrEqual(metrics.p95);
    });

    it('should calculate throughput', () => {
      const calculateThroughput = (
        requests: number,
        durationSeconds: number
      ): number => {
        if (durationSeconds === 0) return 0;
        return requests / durationSeconds;
      };

      expect(calculateThroughput(100, 10)).toBe(10); // 10 req/s
      expect(calculateThroughput(50, 5)).toBe(10);
      expect(calculateThroughput(0, 10)).toBe(0);
    });

    it('should track token usage', () => {
      interface TokenMetrics {
        total_prompt_tokens: number;
        total_completion_tokens: number;
        total_tokens: number;
        avg_tokens_per_request: number;
      }

      const aggregateTokens = (usages: Array<{ prompt: number; completion: number }>): TokenMetrics => {
        const total_prompt = usages.reduce((sum, u) => sum + u.prompt, 0);
        const total_completion = usages.reduce((sum, u) => sum + u.completion, 0);
        const total = total_prompt + total_completion;

        return {
          total_prompt_tokens: total_prompt,
          total_completion_tokens: total_completion,
          total_tokens: total,
          avg_tokens_per_request: usages.length > 0 ? total / usages.length : 0
        };
      };

      const usages = [
        { prompt: 10, completion: 20 },
        { prompt: 15, completion: 25 },
        { prompt: 12, completion: 18 }
      ];

      const metrics = aggregateTokens(usages);

      expect(metrics.total_prompt_tokens).toBe(37);
      expect(metrics.total_completion_tokens).toBe(63);
      expect(metrics.total_tokens).toBe(100);
    });
  });

  describe('Cost Tracking', () => {
    it('should calculate API costs', () => {
      interface PricingConfig {
        prompt_tokens_per_1k: number;
        completion_tokens_per_1k: number;
      }

      const calculateCost = (
        promptTokens: number,
        completionTokens: number,
        pricing: PricingConfig
      ): number => {
        const promptCost = (promptTokens / 1000) * pricing.prompt_tokens_per_1k;
        const completionCost = (completionTokens / 1000) * pricing.completion_tokens_per_1k;
        return promptCost + completionCost;
      };

      const gpt4Pricing: PricingConfig = {
        prompt_tokens_per_1k: 0.03,
        completion_tokens_per_1k: 0.06
      };

      const cost = calculateCost(1000, 500, gpt4Pricing);

      expect(cost).toBe(0.06); // (1000/1000 * 0.03) + (500/1000 * 0.06)
      expect(cost).toBeGreaterThan(0);
    });

    it('should track cost per request', () => {
      const costs = [0.01, 0.02, 0.015, 0.03];
      const avgCost = costs.reduce((a, b) => a + b, 0) / costs.length;

      expect(avgCost).toBe(0.01875);
    });

    it('should compare provider costs', () => {
      const providerCosts = {
        'openai-gpt4': 0.03,
        'anthropic-claude': 0.015,
        'google-gemini': 0.001
      };

      const cheapest = Object.entries(providerCosts).reduce((a, b) =>
        a[1] < b[1] ? a : b
      );

      expect(cheapest[0]).toBe('google-gemini');
      expect(cheapest[1]).toBe(0.001);
    });
  });

  describe('Result Aggregation', () => {
    it('should aggregate benchmark results', () => {
      interface BenchmarkResult {
        provider: string;
        model: string;
        success_count: number;
        failure_count: number;
        avg_latency: number;
        total_cost: number;
      }

      const results: BenchmarkResult[] = [
        { provider: 'openai', model: 'gpt-4', success_count: 95, failure_count: 5, avg_latency: 1500, total_cost: 0.50 },
        { provider: 'anthropic', model: 'claude-3', success_count: 98, failure_count: 2, avg_latency: 1200, total_cost: 0.30 }
      ];

      const totalSuccess = results.reduce((sum, r) => sum + r.success_count, 0);
      const totalFailure = results.reduce((sum, r) => sum + r.failure_count, 0);
      const totalCost = results.reduce((sum, r) => sum + r.total_cost, 0);

      expect(totalSuccess).toBe(193);
      expect(totalFailure).toBe(7);
      expect(totalCost).toBe(0.80);
    });

    it('should calculate success rate', () => {
      const calculateSuccessRate = (success: number, total: number): number => {
        if (total === 0) return 0;
        return (success / total) * 100;
      };

      expect(calculateSuccessRate(95, 100)).toBe(95);
      expect(calculateSuccessRate(0, 100)).toBe(0);
      expect(calculateSuccessRate(100, 100)).toBe(100);
    });
  });

  describe('Comparison and Ranking', () => {
    it('should rank models by performance', () => {
      interface ModelScore {
        model: string;
        score: number;
        latency: number;
        cost: number;
      }

      const rankModels = (models: ModelScore[], metric: keyof Pick<ModelScore, 'score' | 'latency' | 'cost'>): ModelScore[] => {
        return [...models].sort((a, b) => {
          if (metric === 'score') {
            return b[metric] - a[metric]; // Higher score is better
          }
          return a[metric] - b[metric]; // Lower latency/cost is better
        });
      };

      const models: ModelScore[] = [
        { model: 'gpt-4', score: 0.92, latency: 1500, cost: 0.03 },
        { model: 'claude-3', score: 0.95, latency: 1200, cost: 0.015 },
        { model: 'gemini-pro', score: 0.88, latency: 800, cost: 0.001 }
      ];

      const byScore = rankModels(models, 'score');
      const byLatency = rankModels(models, 'latency');
      const byCost = rankModels(models, 'cost');

      expect(byScore[0].model).toBe('claude-3');
      expect(byLatency[0].model).toBe('gemini-pro');
      expect(byCost[0].model).toBe('gemini-pro');
    });

    it('should calculate composite scores', () => {
      const calculateCompositeScore = (
        accuracy: number,
        speed: number,
        cost: number,
        weights: { accuracy: number; speed: number; cost: number }
      ): number => {
        return (
          accuracy * weights.accuracy +
          speed * weights.speed +
          cost * weights.cost
        );
      };

      const weights = { accuracy: 0.5, speed: 0.3, cost: 0.2 };
      const score = calculateCompositeScore(0.9, 0.8, 0.7, weights);

      expect(score).toBeCloseTo(0.83, 2);
    });
  });
});

describe('Report Generation', () => {
  describe('Result Formatting', () => {
    it('should format results as JSON', () => {
      const result = {
        benchmark_name: 'Test Benchmark',
        timestamp: new Date().toISOString(),
        results: [
          { model: 'gpt-4', score: 0.92 }
        ]
      };

      const json = JSON.stringify(result, null, 2);

      expect(json).toContain('benchmark_name');
      expect(json).toContain('timestamp');
      expect(() => JSON.parse(json)).not.toThrow();
    });

    it('should format results as CSV', () => {
      const formatCSV = (headers: string[], rows: string[][]): string => {
        const headerLine = headers.join(',');
        const dataLines = rows.map(row => row.join(','));
        return [headerLine, ...dataLines].join('\n');
      };

      const csv = formatCSV(
        ['Model', 'Score', 'Latency'],
        [
          ['gpt-4', '0.92', '1500'],
          ['claude-3', '0.95', '1200']
        ]
      );

      expect(csv).toContain('Model,Score,Latency');
      expect(csv).toContain('gpt-4,0.92,1500');
    });

    it('should generate summary statistics', () => {
      interface Summary {
        total_requests: number;
        successful_requests: number;
        failed_requests: number;
        avg_latency_ms: number;
        total_cost_usd: number;
      }

      const generateSummary = (
        results: Array<{ success: boolean; latency: number; cost: number }>
      ): Summary => {
        const successful = results.filter(r => r.success);

        return {
          total_requests: results.length,
          successful_requests: successful.length,
          failed_requests: results.length - successful.length,
          avg_latency_ms: successful.reduce((sum, r) => sum + r.latency, 0) / successful.length,
          total_cost_usd: results.reduce((sum, r) => sum + r.cost, 0)
        };
      };

      const results = [
        { success: true, latency: 1000, cost: 0.01 },
        { success: true, latency: 1200, cost: 0.015 },
        { success: false, latency: 5000, cost: 0.0 }
      ];

      const summary = generateSummary(results);

      expect(summary.total_requests).toBe(3);
      expect(summary.successful_requests).toBe(2);
      expect(summary.failed_requests).toBe(1);
      expect(summary.total_cost_usd).toBe(0.025);
    });
  });

  describe('Visualization Data', () => {
    it('should prepare data for charts', () => {
      interface ChartData {
        labels: string[];
        datasets: Array<{
          label: string;
          data: number[];
        }>;
      }

      const prepareChartData = (
        results: Array<{ model: string; score: number; latency: number }>
      ): ChartData => {
        return {
          labels: results.map(r => r.model),
          datasets: [
            {
              label: 'Score',
              data: results.map(r => r.score)
            },
            {
              label: 'Latency (ms)',
              data: results.map(r => r.latency)
            }
          ]
        };
      };

      const results = [
        { model: 'gpt-4', score: 0.92, latency: 1500 },
        { model: 'claude-3', score: 0.95, latency: 1200 }
      ];

      const chartData = prepareChartData(results);

      expect(chartData.labels).toHaveLength(2);
      expect(chartData.datasets).toHaveLength(2);
      expect(chartData.datasets[0].data).toEqual([0.92, 0.95]);
    });
  });
});

describe('Dataset Management', () => {
  describe('Dataset Loading', () => {
    it('should validate dataset format', () => {
      const dataset = {
        name: 'Test Dataset',
        version: '1.0',
        prompts: [
          { id: '1', text: 'Hello', category: 'greeting' },
          { id: '2', text: 'How are you?', category: 'question' }
        ]
      };

      expect(dataset.name).toBeTruthy();
      expect(dataset.version).toMatch(/\d+\.\d+/);
      expect(dataset.prompts.length).toBeGreaterThan(0);

      dataset.prompts.forEach(prompt => {
        expect(prompt.id).toBeTruthy();
        expect(prompt.text).toBeTruthy();
        expect(prompt.category).toBeTruthy();
      });
    });

    it('should support dataset categories', () => {
      const categories = ['coding', 'math', 'reasoning', 'creative', 'factual'];

      categories.forEach(category => {
        expect(typeof category).toBe('string');
        expect(category.length).toBeGreaterThan(0);
      });
    });
  });
});
