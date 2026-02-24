import { type Component, createSignal, splitProps, onMount, onCleanup } from 'solid-js';
import { cn } from '~/lib/utils';
import {
  parseTime,
  formatTime,
  normalizeTimeValue,
  clampHour,
  clampMinute,
} from '~/lib/time';

export interface TimeInputProps {
  value: string; // "HH:MM" format
  onChange: (value: string) => void;
  disabled?: boolean;
  class?: string;
}

/**
 * Custom time input component with 24-hour format and robust keyboard handling.
 *
 * Features:
 * - Separate hour and minute inputs with visual separation
 * - Arrow keys to increment/decrement values
 * - Automatic normalization on blur (e.g., "1" → "01")
 * - Partial input handling during editing
 * - Wraps at boundaries (23 → 00 for hours, 59 → 00 for minutes)
 */
export const TimeInput: Component<TimeInputProps> = (props) => {
  const [local, rest] = splitProps(props, ['value', 'onChange', 'disabled', 'class']);

  // Parse the current value into hour and minute
  const parsed = () => parseTime(local.value);

  // Raw input state for editing (allows partial values like "1")
  const [hourInput, setHourInput] = createSignal(parsed().hour);
  const [minuteInput, setMinuteInput] = createSignal(parsed().minute);

  // Focus state to track which field is being edited
  const [hourFocused, setHourFocused] = createSignal(false);
  const [minuteFocused, setMinuteFocused] = createSignal(false);

  // Refs for the input elements
  let hourRef: HTMLInputElement | undefined;
  let minuteRef: HTMLInputElement | undefined;

  // Update input state when value prop changes externally
  const updateInputsFromValue = () => {
    const p = parseTime(local.value);
    // Only update if not focused (don't interrupt user editing)
    if (!hourFocused()) setHourInput(p.hour);
    if (!minuteFocused()) setMinuteInput(p.minute);
  };

  // Normalize and commit changes
  const normalizeAndCommit = () => {
    const normalizedHour = normalizeTimeValue(hourInput(), 23);
    const normalizedMinute = normalizeTimeValue(minuteInput(), 59);
    const newTime = formatTime(normalizedHour, normalizedMinute);

    setHourInput(normalizedHour);
    setMinuteInput(normalizedMinute);
    local.onChange(newTime);
  };

  // Handle hour input changes
  const handleHourChange = (value: string) => {
    // Allow only numeric input
    const numericValue = value.replace(/[^0-9]/g, '');
    // Limit to 2 characters
    const truncated = numericValue.slice(0, 2);

    // Validate: allow partial during editing, but prevent obviously invalid
    if (truncated.length === 2) {
      const num = parseInt(truncated, 10);
      if (num > 23) return; // Reject values > 23
    }

    setHourInput(truncated);

    // Auto-advance to minute field after 2 digits
    if (truncated.length === 2 && minuteRef) {
      minuteRef.focus();
      minuteRef.select();
    }
  };

  // Handle minute input changes
  const handleMinuteChange = (value: string) => {
    // Allow only numeric input
    const numericValue = value.replace(/[^0-9]/g, '');
    // Limit to 2 characters
    const truncated = numericValue.slice(0, 2);

    // Validate: allow partial during editing, but prevent obviously invalid
    if (truncated.length === 2) {
      const num = parseInt(truncated, 10);
      if (num > 59) return; // Reject values > 59
    }

    setMinuteInput(truncated);
  };

  // Handle keyboard navigation and modification
  const handleHourKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        const currentHour = parseInt(hourInput() || '0', 10);
        const newHour = clampHour(currentHour + 1);
        setHourInput(newHour.toString().padStart(2, '0'));
        normalizeAndCommit();
        break;
      case 'ArrowDown':
        e.preventDefault();
        const currentHourDown = parseInt(hourInput() || '0', 10);
        const newHourDown = clampHour(currentHourDown - 1);
        setHourInput(newHourDown.toString().padStart(2, '0'));
        normalizeAndCommit();
        break;
      case 'ArrowRight':
        // Move to minute field at the end
        if (hourInput().length >= 2 && minuteRef) {
          e.preventDefault();
          minuteRef.focus();
          minuteRef.select();
        }
        break;
      case 'Tab':
        // Normalize before moving focus
        normalizeAndCommit();
        break;
      case ':':
        // Allow colon to move to minute field
        if (minuteRef) {
          e.preventDefault();
          minuteRef.focus();
          minuteRef.select();
        }
        break;
    }
  };

  const handleMinuteKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        const currentMinute = parseInt(minuteInput() || '0', 10);
        const newMinute = clampMinute(currentMinute + 1);
        setMinuteInput(newMinute.toString().padStart(2, '0'));
        normalizeAndCommit();
        break;
      case 'ArrowDown':
        e.preventDefault();
        const currentMinuteDown = parseInt(minuteInput() || '0', 10);
        const newMinuteDown = clampMinute(currentMinuteDown - 1);
        setMinuteInput(newMinuteDown.toString().padStart(2, '0'));
        normalizeAndCommit();
        break;
      case 'ArrowLeft':
        // Move to hour field at the start
        if (minuteInput().length === 0 && hourRef) {
          e.preventDefault();
          hourRef.focus();
          hourRef.select();
        }
        break;
      case 'Tab':
        // Normalize before moving focus
        normalizeAndCommit();
        break;
    }
  };

  // Handle focus events
  const handleHourFocus = () => {
    setHourFocused(true);
    // Select all text for easy replacement
    if (hourRef) hourRef.select();
  };

  const handleHourBlur = () => {
    setHourFocused(false);
    normalizeAndCommit();
  };

  const handleMinuteFocus = () => {
    setMinuteFocused(true);
    // Select all text for easy replacement
    if (minuteRef) minuteRef.select();
  };

  const handleMinuteBlur = () => {
    setMinuteFocused(false);
    normalizeAndCommit();
  };

  // Watch for external value changes
  const createInterval = () => setInterval(updateInputsFromValue, 100);

  onMount(() => {
    const interval = createInterval();
    onCleanup(() => clearInterval(interval));
  });

  return (
    <div
      class={cn(
        'inline-flex items-center gap-1 px-3 py-2 rounded-md text-sm border transition-colors',
        'bg-[#2d2d2d] border-[#3c3c3c] text-[#cccccc]',
        'focus-within:outline-none focus-within:ring-2 focus-within:ring-[#007acc] focus-within:border-[#007acc]',
        local.disabled && 'opacity-50 cursor-not-allowed',
        local.class
      )}
      {...rest}
    >
      {/* Hour Input */}
      <input
        ref={hourRef}
        type="text"
        inputMode="numeric"
        value={hourInput()}
        onInput={(e) => handleHourChange(e.currentTarget.value)}
        onKeyDown={handleHourKeyDown}
        onFocus={handleHourFocus}
        onBlur={handleHourBlur}
        disabled={local.disabled}
        placeholder="00"
        maxLength={2}
        class={cn(
          'w-10 bg-transparent text-center text-[#cccccc] placeholder-[#808080]',
          'focus:outline-none',
          local.disabled && 'cursor-not-allowed'
        )}
        style="font-variant-numeric: tabular-nums;"
      />

      {/* Colon Separator */}
      <span class="text-[#cccccc] select-none" aria-hidden="true">
        :
      </span>

      {/* Minute Input */}
      <input
        ref={minuteRef}
        type="text"
        inputMode="numeric"
        value={minuteInput()}
        onInput={(e) => handleMinuteChange(e.currentTarget.value)}
        onKeyDown={handleMinuteKeyDown}
        onFocus={handleMinuteFocus}
        onBlur={handleMinuteBlur}
        disabled={local.disabled}
        placeholder="00"
        maxLength={2}
        class={cn(
          'w-10 bg-transparent text-center text-[#cccccc] placeholder-[#808080]',
          'focus:outline-none',
          local.disabled && 'cursor-not-allowed'
        )}
        style="font-variant-numeric: tabular-nums;"
      />
    </div>
  );
};
