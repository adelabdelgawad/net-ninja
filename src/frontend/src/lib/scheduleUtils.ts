import type { Schedule } from '../types/task';

/**
 * Compute the next run time for a given schedule.
 * Checks the next 7 days for matching day-of-week + times.
 * @param schedule - Schedule object with days (0-6 for Sun-Sat) and times (HH:MM)
 * @returns Date object for next run time, or null if no schedule
 */
export function getNextRunTime(schedule: Schedule | null): Date | null {
  if (!schedule || !schedule.days || schedule.days.length === 0 || !schedule.times || schedule.times.length === 0) {
    return null;
  }

  const now = new Date();
  const candidates: Date[] = [];

  // Check next 7 days
  for (let dayOffset = 0; dayOffset < 7; dayOffset++) {
    const checkDate = new Date(now);
    checkDate.setDate(now.getDate() + dayOffset);
    const dayOfWeek = checkDate.getDay(); // 0-6 for Sunday-Saturday

    // Skip if this day is not in the schedule
    if (!schedule.days.includes(dayOfWeek)) {
      continue;
    }

    // Check each time for this day
    for (const timeStr of schedule.times) {
      const [hoursStr, minutesStr] = timeStr.split(':');
      const hours = parseInt(hoursStr, 10);
      const minutes = parseInt(minutesStr, 10);

      if (isNaN(hours) || isNaN(minutes)) {
        continue;
      }

      const candidate = new Date(checkDate);
      candidate.setHours(hours, minutes, 0, 0);

      // Only include future times
      if (candidate > now) {
        candidates.push(candidate);
      }
    }
  }

  if (candidates.length === 0) {
    return null;
  }

  // Return earliest candidate
  return candidates.sort((a, b) => a.getTime() - b.getTime())[0];
}

/**
 * Format next run time as human-readable string.
 * Shows "in Xm/Xh" for near-term, date for distant.
 * @param date - Next run time
 * @returns Formatted string like "in 5m", "in 2h", or "Jan 24, 3:00 PM"
 */
export function formatNextRun(date: Date | null): string {
  if (!date) {
    return '--';
  }

  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffMinutes = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);

  // Within 1 hour: show minutes
  if (diffMinutes < 60) {
    return `in ${diffMinutes}m`;
  }

  // Within 24 hours: show hours
  if (diffHours < 24) {
    return `in ${diffHours}h`;
  }

  // Beyond 24 hours: show date
  const options: Intl.DateTimeFormatOptions = {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  };
  return date.toLocaleString(undefined, options);
}
