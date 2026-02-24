/**
 * Parse a time string in "HH:MM" format into hour and minute components.
 */
export function parseTime(timeStr: string): { hour: string; minute: string } {
  const parts = timeStr.split(':');
  return {
    hour: parts[0] || '00',
    minute: parts[1] || '00',
  };
}

/**
 * Format hour and minute values into "HH:MM" format.
 */
export function formatTime(hour: string, minute: string): string {
  const normalizedHour = normalizeTimeValue(hour, 23);
  const normalizedMinute = normalizeTimeValue(minute, 59);
  return `${normalizedHour}:${normalizedMinute}`;
}

/**
 * Normalize a time value to 2 digits with leading zero.
 * Handles partial input (e.g., "1" → "01", empty → "00").
 */
export function normalizeTimeValue(value: string, max: number): string {
  const trimmed = value.trim();
  if (!trimmed) return '00';

  // If it's already 2 digits, validate and return
  if (trimmed.length === 2) {
    const num = parseInt(trimmed, 10);
    if (!isNaN(num) && num >= 0 && num <= max) {
      return trimmed;
    }
    // If invalid, clamp to max
    return max.toString().padStart(2, '0');
  }

  // Single digit - pad with leading zero
  if (trimmed.length === 1) {
    const num = parseInt(trimmed, 10);
    if (!isNaN(num) && num >= 0) {
      return trimmed.padStart(2, '0');
    }
  }

  // For any other case, try to parse and clamp
  const num = parseInt(trimmed, 10);
  if (isNaN(num)) return '00';
  const clamped = Math.min(Math.max(0, num), max);
  return clamped.toString().padStart(2, '0');
}

/**
 * Check if a string is a valid hour (0-23).
 * Accepts partial input during editing (e.g., "1", "2").
 */
export function isValidHour(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return true; // Allow empty during editing

  // Allow single digits 0-9
  if (trimmed.length === 1) {
    const num = parseInt(trimmed, 10);
    return !isNaN(num) && num >= 0 && num <= 9;
  }

  // Allow two digits 00-23
  if (trimmed.length === 2) {
    const num = parseInt(trimmed, 10);
    return !isNaN(num) && num >= 0 && num <= 23;
  }

  return false;
}

/**
 * Check if a string is a valid minute (0-59).
 * Accepts partial input during editing (e.g., "5", "30").
 */
export function isValidMinute(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return true; // Allow empty during editing

  // Allow single digits 0-9
  if (trimmed.length === 1) {
    const num = parseInt(trimmed, 10);
    return !isNaN(num) && num >= 0 && num <= 9;
  }

  // Allow two digits 00-59
  if (trimmed.length === 2) {
    const num = parseInt(trimmed, 10);
    return !isNaN(num) && num >= 0 && num <= 59;
  }

  return false;
}

/**
 * Clamp a number to valid hour range (0-23).
 * Wraps around: 24 → 0, -1 → 23
 */
export function clampHour(value: number): number {
  if (value >= 24) return 0;
  if (value < 0) return 23;
  return value;
}

/**
 * Clamp a number to valid minute range (0-59).
 * Wraps around: 60 → 0, -1 → 59
 */
export function clampMinute(value: number): number {
  if (value >= 60) return 0;
  if (value < 0) return 59;
  return value;
}
