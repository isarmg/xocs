import assert from 'node:assert/strict';
import { test } from 'node:test';
import { timeTogether } from '../src/love-time.ts';

const date = (...parts) => new Date(...parts).getTime();
test('the home timer counts calendar years and months at leap days and month ends', () => {
  for (const [since, now, years, months, days] of [
    [date(2026, 0, 31), date(2026, 1, 28), 0, 1, 0],
    [date(2026, 0, 31), date(2026, 2, 1), 0, 1, 1],
    [date(2024, 1, 29), date(2025, 1, 28), 1, 0, 0],
    [date(2016, 1, 1), date(2026, 9, 11), 10, 8, 10],
  ]) assert.deepEqual(timeTogether(since, now), { years, months, days, hours: 0, minutes: 0, seconds: 0 });
  assert.deepEqual(timeTogether(date(2026, 9, 1), date(2026, 9, 8, 12, 34, 56)), { years: 0, months: 0, days: 7, hours: 12, minutes: 34, seconds: 56 });
});
test('the home timer waits for its start date and handles missing or invalid dates', () => {
  const now = date(2026, 9, 11);
  for (const since of [NaN, Infinity, 1e50, now + 1]) assert.equal(timeTogether(since, now), null);
  assert.equal(timeTogether(now, NaN), null);
});
