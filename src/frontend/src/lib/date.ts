/**
 * Format a date/time for display in device local timezone
 */
export function formatDateTime(
  dateStr: string | null | undefined,
  options: Intl.DateTimeFormatOptions = {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }
): string {
  if (!dateStr) return '-';
  try {
    return new Date(dateStr).toLocaleString(undefined, options);
  } catch {
    return dateStr;
  }
}

export function formatShortDateTime(dateStr: string | null | undefined): string {
  return formatDateTime(dateStr, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
