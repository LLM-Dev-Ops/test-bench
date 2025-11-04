/**
 * Global test setup file for Vitest
 * This file runs before all tests
 */

import { beforeAll, afterAll, afterEach } from 'vitest';

beforeAll(() => {
  // Setup code that runs once before all tests
  console.log('🧪 Starting test suite...');
});

afterAll(() => {
  // Cleanup code that runs once after all tests
  console.log('✅ Test suite completed!');
});

afterEach(() => {
  // Cleanup after each test
  // Clear any mocks, reset state, etc.
});

// Extend expect with custom matchers if needed
// expect.extend({
//   toBeValidLLMResponse(received) {
//     // Custom matcher implementation
//   },
// });
