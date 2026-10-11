// Calendar months retain the original day, clamped at the end of shorter months.
export function timeTogether(since: number, now: number) {
  if (!Number.isFinite(since) || !Number.isFinite(now) || since > now) return null;
  const start = new Date(since), end = new Date(now);
  if (!Number.isFinite(start.getTime()) || !Number.isFinite(end.getTime())) return null;
  const anniversary = (months: number) => {
    if (months === 0) return new Date(start);
    const date = new Date(start);
    date.setDate(1);
    date.setMonth(start.getMonth() + months);
    date.setDate(Math.min(start.getDate(), new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate()));
    return date;
  };
  let months = (end.getFullYear() - start.getFullYear()) * 12 + end.getMonth() - start.getMonth();
  if (anniversary(months).getTime() > now) months--;
  let cursor = anniversary(months), days = 0;
  for (;;) {
    const next = new Date(cursor); next.setDate(next.getDate() + 1);
    if (next.getTime() > now) break;
    cursor = next; days++;
  }
  const seconds = Math.floor((now - cursor.getTime()) / 1000);
  return { years: Math.floor(months / 12), months: months % 12, days, hours: Math.floor(seconds / 3600), minutes: Math.floor(seconds / 60) % 60, seconds: seconds % 60 };
}
