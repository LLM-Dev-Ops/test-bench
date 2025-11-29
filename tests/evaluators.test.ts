/**
 * Unit tests for LLM evaluation metrics
 */

import { describe, it, expect } from 'vitest';

describe('Evaluation Metrics', () => {
  describe('Perplexity Evaluator', () => {
    it('should calculate perplexity score', () => {
      // Mock perplexity calculation
      const calculatePerplexity = (logProbs: number[]): number => {
        if (logProbs.length === 0) return 0;
        const avgLogProb = logProbs.reduce((a, b) => a + b, 0) / logProbs.length;
        return Math.exp(-avgLogProb);
      };

      const logProbs = [-0.5, -0.3, -0.7, -0.4];
      const perplexity = calculatePerplexity(logProbs);

      expect(perplexity).toBeGreaterThan(0);
      expect(perplexity).toBeLessThan(10); // Low perplexity is better
    });

    it('should handle empty input', () => {
      const calculatePerplexity = (logProbs: number[]): number => {
        if (logProbs.length === 0) return 0;
        const avgLogProb = logProbs.reduce((a, b) => a + b, 0) / logProbs.length;
        return Math.exp(-avgLogProb);
      };

      const perplexity = calculatePerplexity([]);
      expect(perplexity).toBe(0);
    });

    it('should return higher perplexity for worse predictions', () => {
      const calculatePerplexity = (logProbs: number[]): number => {
        if (logProbs.length === 0) return 0;
        const avgLogProb = logProbs.reduce((a, b) => a + b, 0) / logProbs.length;
        return Math.exp(-avgLogProb);
      };

      const goodPredictions = [-0.1, -0.2, -0.1];
      const poorPredictions = [-2.0, -3.0, -2.5];

      const goodPerplexity = calculatePerplexity(goodPredictions);
      const poorPerplexity = calculatePerplexity(poorPredictions);

      expect(poorPerplexity).toBeGreaterThan(goodPerplexity);
    });
  });

  describe('Coherence Evaluator', () => {
    it('should detect coherence violations', () => {
      interface CoherenceViolation {
        type: string;
        position: number;
        severity: 'low' | 'medium' | 'high';
      }

      const detectViolations = (text: string): CoherenceViolation[] => {
        const violations: CoherenceViolation[] = [];

        // Check for abrupt topic changes (simplified)
        if (text.includes('however') && text.includes('but')) {
          violations.push({
            type: 'contradictory_markers',
            position: text.indexOf('but'),
            severity: 'medium'
          });
        }

        return violations;
      };

      const coherentText = 'The sky is blue. It is a beautiful day.';
      const incoherentText = 'The sky is blue. However, I like pizza. But wait, what?';

      expect(detectViolations(coherentText).length).toBe(0);
      expect(detectViolations(incoherentText).length).toBeGreaterThan(0);
    });

    it('should calculate coherence score', () => {
      const calculateCoherence = (violations: number, textLength: number): number => {
        if (textLength === 0) return 0;
        return Math.max(0, 1 - (violations / (textLength / 100)));
      };

      const score1 = calculateCoherence(0, 100); // No violations
      const score2 = calculateCoherence(5, 100); // Some violations

      expect(score1).toBe(1.0);
      expect(score2).toBeLessThan(1.0);
      expect(score2).toBeGreaterThanOrEqual(0);
    });

    it('should identify discourse markers', () => {
      const discourseMarkers = [
        'however',
        'therefore',
        'moreover',
        'furthermore',
        'consequently',
        'meanwhile',
        'nevertheless'
      ];

      const text = 'The system works well. However, there are some issues.';
      const hasMarker = discourseMarkers.some(marker => text.toLowerCase().includes(marker));

      expect(hasMarker).toBe(true);
    });
  });

  describe('Faithfulness Evaluator', () => {
    it('should detect hallucinations', () => {
      const detectHallucination = (
        response: string,
        context: string
      ): { hasHallucination: boolean; confidence: number } => {
        // Simplified hallucination detection
        const responseFacts = response.toLowerCase().split('.').filter(s => s.trim());
        const contextLower = context.toLowerCase();

        let unsupportedFacts = 0;
        responseFacts.forEach(fact => {
          if (fact.length > 10 && !contextLower.includes(fact.trim().substring(0, 20))) {
            unsupportedFacts++;
          }
        });

        const confidence = unsupportedFacts / Math.max(responseFacts.length, 1);
        return {
          hasHallucination: unsupportedFacts > 0,
          confidence
        };
      };

      const context = 'The sky is blue during the day.';
      const faithfulResponse = 'The sky appears blue.';
      const hallucinatedResponse = 'The sky is green and made of cheese.';

      const result1 = detectHallucination(faithfulResponse, context);
      const result2 = detectHallucination(hallucinatedResponse, context);

      expect(result1.confidence).toBeLessThan(result2.confidence);
    });

    it('should calculate faithfulness score', () => {
      const calculateFaithfulness = (
        supportedClaims: number,
        totalClaims: number
      ): number => {
        if (totalClaims === 0) return 1.0;
        return supportedClaims / totalClaims;
      };

      expect(calculateFaithfulness(10, 10)).toBe(1.0);
      expect(calculateFaithfulness(5, 10)).toBe(0.5);
      expect(calculateFaithfulness(0, 10)).toBe(0.0);
    });

    it('should verify source attribution', () => {
      interface Attribution {
        claim: string;
        source: string | null;
      }

      const checkAttribution = (text: string): Attribution[] => {
        const attributions: Attribution[] = [];
        const sentences = text.split('.').filter(s => s.trim());

        sentences.forEach(sentence => {
          const hasAttribution = sentence.includes('according to') ||
            sentence.includes('states that') ||
            sentence.includes('reported that');

          attributions.push({
            claim: sentence.trim(),
            source: hasAttribution ? 'cited' : null
          });
        });

        return attributions;
      };

      const textWithAttribution = 'According to the report, sales increased.';
      const textWithoutAttribution = 'Sales increased.';

      const attr1 = checkAttribution(textWithAttribution);
      const attr2 = checkAttribution(textWithoutAttribution);

      expect(attr1[0].source).toBeTruthy();
      expect(attr2[0].source).toBeNull();
    });
  });

  describe('Relevance Evaluator', () => {
    it('should calculate semantic similarity', () => {
      // Simplified similarity using word overlap
      const calculateSimilarity = (text1: string, text2: string): number => {
        const words1 = new Set(text1.toLowerCase().split(/\s+/));
        const words2 = new Set(text2.toLowerCase().split(/\s+/));

        const intersection = new Set([...words1].filter(w => words2.has(w)));
        const union = new Set([...words1, ...words2]);

        return intersection.size / union.size;
      };

      const prompt = 'What is the weather today?';
      const relevant = 'The weather today is sunny and warm.';
      const irrelevant = 'I like to eat pizza for dinner.';

      const relevanceScore = calculateSimilarity(prompt, relevant);
      const irrelevanceScore = calculateSimilarity(prompt, irrelevant);

      expect(relevanceScore).toBeGreaterThan(irrelevanceScore);
    });

    it('should detect off-topic responses', () => {
      const isOnTopic = (prompt: string, response: string, threshold = 0.1): boolean => {
        const words1 = new Set(prompt.toLowerCase().split(/\s+/));
        const words2 = new Set(response.toLowerCase().split(/\s+/));
        const intersection = new Set([...words1].filter(w => words2.has(w)));
        const union = new Set([...words1, ...words2]);
        const similarity = intersection.size / union.size;

        return similarity >= threshold;
      };

      const prompt = 'Explain machine learning.';
      const onTopic = 'Machine learning is a subset of AI that uses algorithms.';
      const offTopic = 'The recipe for chocolate cake requires flour and eggs.';

      expect(isOnTopic(prompt, onTopic)).toBe(true);
      expect(isOnTopic(prompt, offTopic)).toBe(false);
    });

    it('should measure answer completeness', () => {
      const measureCompleteness = (
        prompt: string,
        response: string
      ): { complete: boolean; coverage: number } => {
        // Extract question words
        const questionWords = ['what', 'why', 'how', 'when', 'where', 'who'];
        const promptLower = prompt.toLowerCase();

        const questionType = questionWords.find(w => promptLower.includes(w));
        const responseLength = response.split(/\s+/).length;

        // Simple heuristic: responses should have minimum length based on question
        const minLength = questionType === 'why' || questionType === 'how' ? 20 : 5;
        const complete = responseLength >= minLength;
        const coverage = Math.min(1, responseLength / minLength);

        return { complete, coverage };
      };

      const prompt = 'Why is the sky blue?';
      const shortResponse = 'Because of light.';
      const completeResponse = 'The sky is blue because of Rayleigh scattering. When sunlight enters the atmosphere, shorter blue wavelengths scatter more than other colors.';

      const result1 = measureCompleteness(prompt, shortResponse);
      const result2 = measureCompleteness(prompt, completeResponse);

      expect(result2.coverage).toBeGreaterThan(result1.coverage);
    });
  });

  describe('LLM-as-Judge', () => {
    it('should structure evaluation prompts', () => {
      const createEvaluationPrompt = (
        task: string,
        response: string,
        criteria: string[]
      ): string => {
        return `
Evaluate the following response for the task: ${task}

Response: ${response}

Criteria:
${criteria.map(c => `- ${c}`).join('\n')}

Rate each criterion on a scale of 1-5 and provide overall feedback.
`.trim();
      };

      const prompt = createEvaluationPrompt(
        'Summarize the article',
        'The article discusses climate change.',
        ['Accuracy', 'Completeness', 'Clarity']
      );

      expect(prompt).toContain('Evaluate');
      expect(prompt).toContain('Accuracy');
      expect(prompt).toContain('scale of 1-5');
    });

    it('should parse judge responses', () => {
      interface JudgeResult {
        scores: Record<string, number>;
        feedback: string;
      }

      const parseJudgeResponse = (response: string): JudgeResult => {
        const scores: Record<string, number> = {};
        const lines = response.split('\n');

        lines.forEach(line => {
          const match = line.match(/(\w+):\s*(\d+)/);
          if (match) {
            scores[match[1].toLowerCase()] = parseInt(match[2]);
          }
        });

        const feedbackIndex = response.toLowerCase().indexOf('feedback:');
        const feedback = feedbackIndex >= 0
          ? response.substring(feedbackIndex + 9).trim()
          : '';

        return { scores, feedback };
      };

      const judgeResponse = `
Accuracy: 4
Completeness: 3
Clarity: 5

Feedback: Good overall but could be more detailed.
`;

      const result = parseJudgeResponse(judgeResponse);

      expect(result.scores.accuracy).toBe(4);
      expect(result.scores.completeness).toBe(3);
      expect(result.scores.clarity).toBe(5);
      expect(result.feedback).toContain('Good overall');
    });

    it('should aggregate judge scores', () => {
      const aggregateScores = (scores: Record<string, number>): number => {
        const values = Object.values(scores);
        if (values.length === 0) return 0;

        const sum = values.reduce((a, b) => a + b, 0);
        return sum / values.length;
      };

      const scores = {
        accuracy: 4,
        completeness: 3,
        clarity: 5,
        relevance: 4
      };

      const average = aggregateScores(scores);

      expect(average).toBe(4);
      expect(average).toBeGreaterThanOrEqual(1);
      expect(average).toBeLessThanOrEqual(5);
    });
  });
});

describe('Text Analysis Utilities', () => {
  describe('Token Counting', () => {
    it('should estimate token count', () => {
      const estimateTokens = (text: string): number => {
        // Rough approximation: 1 token ≈ 4 characters
        return Math.ceil(text.length / 4);
      };

      expect(estimateTokens('Hello, world!')).toBeGreaterThan(0);
      expect(estimateTokens('a'.repeat(100))).toBe(25);
      expect(estimateTokens('')).toBe(0);
    });
  });

  describe('Readability Metrics', () => {
    it('should calculate sentence complexity', () => {
      const calculateComplexity = (text: string): number => {
        const sentences = text.split(/[.!?]+/).filter(s => s.trim());
        const words = text.split(/\s+/).filter(w => w.trim());

        if (sentences.length === 0) return 0;

        const avgWordsPerSentence = words.length / sentences.length;
        return avgWordsPerSentence;
      };

      const simple = 'I like cats. They are nice.';
      const complex = 'The implementation of sophisticated algorithms requires comprehensive understanding of computational complexity theory.';

      const simpleComplexity = calculateComplexity(simple);
      const complexComplexity = calculateComplexity(complex);

      expect(complexComplexity).toBeGreaterThan(simpleComplexity);
    });
  });

  describe('Sentiment Analysis', () => {
    it('should detect sentiment indicators', () => {
      const detectSentiment = (text: string): 'positive' | 'negative' | 'neutral' => {
        const positive = ['good', 'great', 'excellent', 'amazing', 'wonderful'];
        const negative = ['bad', 'terrible', 'awful', 'horrible', 'poor'];

        const textLower = text.toLowerCase();
        const hasPositive = positive.some(word => textLower.includes(word));
        const hasNegative = negative.some(word => textLower.includes(word));

        if (hasPositive && !hasNegative) return 'positive';
        if (hasNegative && !hasPositive) return 'negative';
        return 'neutral';
      };

      expect(detectSentiment('This is a great product!')).toBe('positive');
      expect(detectSentiment('This is a terrible experience.')).toBe('negative');
      expect(detectSentiment('The sky is blue.')).toBe('neutral');
    });
  });
});
