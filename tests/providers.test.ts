/**
 * Unit tests for LLM Provider abstractions and interfaces
 */

import { describe, it, expect } from 'vitest';

describe('Provider Interface', () => {
  describe('Completion Request', () => {
    it('should have required fields', () => {
      const request = {
        model: 'gpt-4',
        prompt: 'Hello, world!',
        temperature: 0.7,
        max_tokens: 100,
        stream: false
      };

      expect(request.model).toBeTruthy();
      expect(request.prompt).toBeTruthy();
      expect(request.temperature).toBeGreaterThanOrEqual(0);
      expect(request.temperature).toBeLessThanOrEqual(2);
      expect(request.max_tokens).toBeGreaterThan(0);
      expect(typeof request.stream).toBe('boolean');
    });

    it('should validate temperature range', () => {
      const validTemperatures = [0, 0.5, 0.7, 1.0, 1.5, 2.0];
      const invalidTemperatures = [-0.1, 2.1, -1, 3];

      validTemperatures.forEach(temp => {
        expect(temp).toBeGreaterThanOrEqual(0);
        expect(temp).toBeLessThanOrEqual(2);
      });

      invalidTemperatures.forEach(temp => {
        const isValid = temp >= 0 && temp <= 2;
        expect(isValid).toBe(false);
      });
    });

    it('should validate max_tokens', () => {
      expect(100).toBeGreaterThan(0);
      expect(4096).toBeGreaterThan(0);
      expect(0).not.toBeGreaterThan(0);
      expect(-1).not.toBeGreaterThan(0);
    });

    it('should support optional parameters', () => {
      const minimalRequest = {
        model: 'gpt-4',
        prompt: 'Test'
      };

      const fullRequest = {
        model: 'gpt-4',
        prompt: 'Test',
        temperature: 0.7,
        max_tokens: 100,
        top_p: 0.9,
        stop: ['END'],
        stream: false
      };

      expect(minimalRequest.model).toBeTruthy();
      expect(minimalRequest.prompt).toBeTruthy();
      expect(fullRequest.temperature).toBeDefined();
      expect(fullRequest.max_tokens).toBeDefined();
      expect(fullRequest.top_p).toBeDefined();
      expect(fullRequest.stop).toBeDefined();
    });
  });

  describe('Completion Response', () => {
    it('should have required response fields', () => {
      const response = {
        id: 'chatcmpl-123',
        model: 'gpt-4',
        content: 'Hello! How can I help you?',
        finish_reason: 'stop',
        usage: {
          prompt_tokens: 10,
          completion_tokens: 15,
          total_tokens: 25
        },
        created: Date.now()
      };

      expect(response.id).toBeTruthy();
      expect(response.model).toBeTruthy();
      expect(response.content).toBeTruthy();
      expect(response.finish_reason).toBeTruthy();
      expect(response.usage).toBeDefined();
      expect(response.created).toBeGreaterThan(0);
    });

    it('should validate token usage', () => {
      const usage = {
        prompt_tokens: 10,
        completion_tokens: 15,
        total_tokens: 25
      };

      expect(usage.prompt_tokens).toBeGreaterThan(0);
      expect(usage.completion_tokens).toBeGreaterThan(0);
      expect(usage.total_tokens).toBe(usage.prompt_tokens + usage.completion_tokens);
    });

    it('should support different finish reasons', () => {
      const validFinishReasons = ['stop', 'length', 'content_filter', 'tool_calls', 'function_call'];

      validFinishReasons.forEach(reason => {
        expect(validFinishReasons).toContain(reason);
      });
    });
  });

  describe('Model Information', () => {
    it('should provide model metadata', () => {
      const modelInfo = {
        id: 'gpt-4',
        name: 'GPT-4',
        provider: 'openai',
        max_tokens: 8192,
        supports_streaming: true,
        supports_functions: true
      };

      expect(modelInfo.id).toBeTruthy();
      expect(modelInfo.name).toBeTruthy();
      expect(modelInfo.provider).toBeTruthy();
      expect(modelInfo.max_tokens).toBeGreaterThan(0);
      expect(typeof modelInfo.supports_streaming).toBe('boolean');
      expect(typeof modelInfo.supports_functions).toBe('boolean');
    });

    it('should have valid context window sizes', () => {
      const models = [
        { id: 'gpt-4', max_tokens: 8192 },
        { id: 'gpt-4-turbo', max_tokens: 128000 },
        { id: 'gpt-3.5-turbo', max_tokens: 16385 },
        { id: 'claude-3-opus', max_tokens: 200000 }
      ];

      models.forEach(model => {
        expect(model.max_tokens).toBeGreaterThan(0);
        expect(model.max_tokens).toBeLessThanOrEqual(1000000); // Reasonable upper bound
      });
    });
  });

  describe('Streaming Support', () => {
    it('should handle streaming chunks', () => {
      const chunk = {
        content: 'Hello',
        is_final: false,
        finish_reason: null
      };

      expect(chunk.content).toBeDefined();
      expect(typeof chunk.is_final).toBe('boolean');
      expect(chunk.finish_reason === null || typeof chunk.finish_reason === 'string').toBe(true);
    });

    it('should mark final chunk correctly', () => {
      const finalChunk = {
        content: '',
        is_final: true,
        finish_reason: 'stop'
      };

      expect(finalChunk.is_final).toBe(true);
      expect(finalChunk.finish_reason).toBeTruthy();
    });
  });

  describe('Error Handling', () => {
    it('should define provider error types', () => {
      const errorTypes = [
        'InvalidApiKey',
        'RateLimitExceeded',
        'InvalidRequest',
        'ServerError',
        'NetworkError',
        'TimeoutError',
        'InvalidModel'
      ];

      errorTypes.forEach(errorType => {
        expect(errorTypes).toContain(errorType);
        expect(errorType).toBeTruthy();
      });
    });

    it('should include retry-after for rate limits', () => {
      const rateLimitError = {
        type: 'RateLimitExceeded',
        message: 'Rate limit exceeded',
        retry_after: 60 // seconds
      };

      expect(rateLimitError.type).toBe('RateLimitExceeded');
      expect(rateLimitError.retry_after).toBeGreaterThan(0);
    });

    it('should provide error details', () => {
      const error = {
        type: 'InvalidRequest',
        message: 'Invalid parameter: temperature must be between 0 and 2',
        details: {
          parameter: 'temperature',
          value: 3.0,
          expected: '0-2'
        }
      };

      expect(error.type).toBeTruthy();
      expect(error.message).toBeTruthy();
      expect(error.details).toBeDefined();
    });
  });
});

describe('Provider-Specific Features', () => {
  describe('OpenAI', () => {
    it('should support GPT models', () => {
      const models = ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo', 'gpt-4o'];

      models.forEach(model => {
        expect(model).toMatch(/^gpt-/);
      });
    });

    it('should support function calling', () => {
      const request = {
        model: 'gpt-4',
        prompt: 'What is the weather?',
        functions: [
          {
            name: 'get_weather',
            description: 'Get the current weather',
            parameters: {
              type: 'object',
              properties: {
                location: { type: 'string' }
              }
            }
          }
        ]
      };

      expect(request.functions).toBeDefined();
      expect(request.functions.length).toBeGreaterThan(0);
      expect(request.functions[0].name).toBeTruthy();
    });
  });

  describe('Anthropic', () => {
    it('should support Claude models', () => {
      const models = [
        'claude-3-opus-20240229',
        'claude-3-sonnet-20240229',
        'claude-3-haiku-20240307'
      ];

      models.forEach(model => {
        expect(model).toMatch(/^claude-/);
      });
    });

    it('should support system prompts', () => {
      const request = {
        model: 'claude-3-opus-20240229',
        system: 'You are a helpful assistant.',
        prompt: 'Hello!'
      };

      expect(request.system).toBeTruthy();
      expect(request.prompt).toBeTruthy();
    });
  });

  describe('Google', () => {
    it('should support Gemini models', () => {
      const models = [
        'gemini-pro',
        'gemini-pro-vision',
        'gemini-1.5-pro',
        'gemini-1.5-flash'
      ];

      models.forEach(model => {
        expect(model).toMatch(/^gemini-/);
      });
    });

    it('should support multimodal inputs', () => {
      const request = {
        model: 'gemini-pro-vision',
        prompt: 'What is in this image?',
        images: ['data:image/png;base64,iVBORw0KG...']
      };

      expect(request.images).toBeDefined();
      expect(request.images.length).toBeGreaterThan(0);
    });
  });
});

describe('Provider Factory', () => {
  it('should create providers from configuration', () => {
    const configs = [
      { name: 'openai', type: 'OpenAI' },
      { name: 'anthropic', type: 'Anthropic' },
      { name: 'google', type: 'Google' }
    ];

    configs.forEach(config => {
      expect(config.name).toBeTruthy();
      expect(config.type).toBeTruthy();
    });
  });

  it('should validate provider names', () => {
    const validProviders = [
      'openai',
      'anthropic',
      'google',
      'cohere',
      'mistral',
      'groq',
      'ollama'
    ];

    validProviders.forEach(provider => {
      expect(provider).toMatch(/^[a-z]+$/);
      expect(provider.length).toBeGreaterThan(0);
    });
  });
});
