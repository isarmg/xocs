import assert from 'node:assert/strict';
import { test } from 'node:test';
import { formatUtcTime, parseUtcDate, utcDateTimeInput } from './fixtures/react-baseline/src/utc-time.ts';

test('SQLite timestamps and offset-free date inputs mean UTC', () => {
  assert.equal(parseUtcDate('2026-10-12 23:59:59'), Date.parse('2026-10-12T23:59:59Z'));
  assert.equal(parseUtcDate('2028-02-29'), Date.parse('2028-02-29T00:00:00Z'));
  assert.equal(utcDateTimeInput('2026-10-12T23:59'), '2026-10-12T23:59:00.000Z');
  assert.equal(utcDateTimeInput(''), null);
});

test('displayed timestamps explicitly identify UTC and handle missing data', () => {
  assert.equal(formatUtcTime('2026-10-12 23:59:59'), '2026-10-12 23:59:59 UTC');
  assert.equal(formatUtcTime('2026-10-13T00:00:00Z'), '2026-10-13 00:00:00 UTC');
  assert.equal(formatUtcTime(null), null);
  assert.equal(formatUtcTime('invalid'), 'invalid');
});
