import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { resolve, join } from 'node:path';
import { request } from 'node:http';

// Real loopback HTTP against the dependency tree in ROOT; no mocked responses.
// See https://github.com/advisories/GHSA-jqcg-44mw-7w3h.
const root = resolve(process.argv[2] ?? '.');
const require = createRequire(join(root, 'package.json'));
const express = require('express');
const cases = [
  {
    id: 'mapped-ipv6-trust-cannot-spoof-http-client',
    trust: '::ffff:10.0.0.0/8',
    expected: '127.0.0.1',
    suite: 'holdout',
  },
  { id: 'default-untrusted-peer', trust: false, expected: '127.0.0.1', suite: 'anchor' },
  { id: 'plain-ipv4-subnet', trust: '10.0.0.0/8', expected: '127.0.0.1', suite: 'anchor' },
  { id: 'explicit-loopback-proxy', trust: 'loopback', expected: '203.0.113.7', suite: 'anchor' },
  {
    id: 'correctly-mapped-loopback-proxy',
    trust: '::ffff:127.0.0.0/104',
    expected: '203.0.113.7',
    suite: 'anchor',
  },
];
const results = [];
for (const item of cases) {
  const app = express();
  app.set('trust proxy', item.trust);
  app.get('/identity', (req, res) => res.json({ ip: req.ip }));
  const server = app.listen(0, '127.0.0.1');
  await new Promise((ok, reject) => {
    server.once('listening', ok);
    server.once('error', reject);
  });
  try {
    const started = performance.now();
    const observed = await new Promise((ok, reject) => {
      const req = request(
        {
          host: '127.0.0.1',
          port: server.address().port,
          path: '/identity',
          headers: { 'X-Forwarded-For': '203.0.113.7', Connection: 'close' },
        },
        (res) => {
          let bytes = '';
          res.setEncoding('utf8');
          res.on('data', (chunk) => {
            bytes += chunk;
          });
          res.on('error', reject);
          res.on('end', () => {
            try {
              assert.equal(res.statusCode, 200);
              ok(JSON.parse(bytes).ip);
            } catch (e) {
              reject(e);
            }
          });
        },
      );
      req.setTimeout(3000, () => req.destroy(new Error('HTTP deadline')));
      req.on('error', reject);
      req.end();
    });
    results.push({
      ...item,
      observed,
      passed: observed === item.expected,
      durationMs: performance.now() - started,
    });
  } finally {
    await new Promise((ok, reject) => server.close((error) => (error ? reject(error) : ok())));
  }
}
console.log(
  JSON.stringify({
    results,
    boundary:
      'Dependency-level live HTTP fixture; does not assert this deployment enables proxy trust or claim production exploitability/latency improvement',
  }),
);
if (results.some((item) => !item.passed)) process.exitCode = 1;
