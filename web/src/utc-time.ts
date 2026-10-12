// SQLite timestamps and date inputs without an offset always refer to UTC.
export function parseUtcDate(value: string | null | undefined): number {
  if (!value) return NaN;
  let normalized = value.trim().replace(' ', 'T');
  if (/^\d{4}-\d{2}-\d{2}$/.test(normalized)) normalized += 'T00:00:00';
  if (!/(?:Z|[+-]\d{2}:?\d{2})$/i.test(normalized)) normalized += 'Z';
  return Date.parse(normalized);
}

export function formatUtcTime(value: string | null | undefined): string | null {
  if (!value) return null;
  const timestamp = parseUtcDate(value);
  if (!Number.isFinite(timestamp)) return value;
  return new Date(timestamp).toISOString().slice(0, 19).replace('T', ' ') + ' UTC';
}

export function utcDateTimeInput(value: string): string | null {
  return value ? new Date(parseUtcDate(value)).toISOString() : null;
}
