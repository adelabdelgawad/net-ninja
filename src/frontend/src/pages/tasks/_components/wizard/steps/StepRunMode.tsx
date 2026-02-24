import { type Component, Show, createSignal, For } from 'solid-js';
import { cn } from '~/lib/utils';
import { Play, Calendar, Clock, Plus, X } from 'lucide-solid';
import type { Schedule } from '~/types';
import { TimeInput } from '~/components/ui/time-input';

export interface StepRunModeProps {
  runMode: 'one_time' | 'scheduled' | '';
  schedule: Schedule | null;
  onRunModeChange: (value: 'one_time' | 'scheduled') => void;
  onScheduleChange: (schedule: Schedule | null) => void;
}

const DAYS_OF_WEEK = [
  { value: 1, label: 'Mon' },
  { value: 2, label: 'Tue' },
  { value: 3, label: 'Wed' },
  { value: 4, label: 'Thu' },
  { value: 5, label: 'Fri' },
  { value: 6, label: 'Sat' },
  { value: 0, label: 'Sun' },
];

export const StepRunMode: Component<StepRunModeProps> = (props) => {
  const [times, setTimes] = createSignal<string[]>(['09:00']);
  const [selectedDays, setSelectedDays] = createSignal<number[]>([]);

  const handleRunModeChange = (mode: 'one_time' | 'scheduled') => {
    props.onRunModeChange(mode);
    if (mode === 'one_time') {
      props.onScheduleChange(null);
    } else {
      // Initialize with default schedule
      updateSchedule(selectedDays(), times());
    }
  };

  const toggleDay = (day: number) => {
    const current = selectedDays();
    const newDays = current.includes(day)
      ? current.filter(d => d !== day)
      : [...current, day].sort((a, b) => {
          // Sort with Monday first, Sunday last
          const orderA = a === 0 ? 7 : a;
          const orderB = b === 0 ? 7 : b;
          return orderA - orderB;
        });
    setSelectedDays(newDays);
    updateSchedule(newDays, times());
  };

  const handleTimeChange = (index: number, newTime: string) => {
    const currentTimes = [...times()];
    currentTimes[index] = newTime;
    setTimes(currentTimes);
    updateSchedule(selectedDays(), currentTimes);
  };

  const addTime = () => {
    const currentTimes = times();
    // Add a new time slot 1 hour after the last one, or default to 10:00
    const lastTime = currentTimes[currentTimes.length - 1];
    const [hours, minutes] = lastTime.split(':').map(Number);
    const nextHour = (hours + 1) % 24;
    const newTime = `${nextHour.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
    const newTimes = [...currentTimes, newTime];
    setTimes(newTimes);
    updateSchedule(selectedDays(), newTimes);
  };

  const removeTime = (index: number) => {
    const currentTimes = times();
    if (currentTimes.length <= 1) return; // Keep at least one time
    const newTimes = currentTimes.filter((_, i) => i !== index);
    setTimes(newTimes);
    updateSchedule(selectedDays(), newTimes);
  };

  const updateSchedule = (days: number[], timeValues: string[]) => {
    if (props.runMode === 'scheduled') {
      props.onScheduleChange({
        days,
        times: timeValues,
      });
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h3 class="text-lg font-medium text-[#cccccc] mb-2">Run Mode</h3>
        <p class="text-sm text-[#808080]">
          Choose when and how often this task should run.
        </p>
      </div>

      {/* Run Mode Selection */}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <button
          type="button"
          onClick={() => handleRunModeChange('one_time')}
          class={cn(
            'group relative flex flex-col items-start gap-3 p-5 rounded-lg border-2 transition-all cursor-pointer text-left',
            'bg-[#2d2d2d] hover:bg-[#2d2d2d]/80',
            props.runMode === 'one_time'
              ? 'border-[#007acc] ring-2 ring-[#007acc]/30'
              : 'border-[#3c3c3c] hover:border-[#007acc]/50'
          )}
        >
          <div
            class={cn(
              'w-12 h-12 rounded-lg flex items-center justify-center transition-colors',
              props.runMode === 'one_time'
                ? 'bg-[#007acc] text-white'
                : 'bg-[#3c3c3c] text-[#808080] group-hover:bg-[#007acc]/20 group-hover:text-[#007acc]'
            )}
          >
            <Play class="w-6 h-6" />
          </div>

          <div class="flex-1">
            <h4 class="text-base font-medium text-[#cccccc] mb-1">
              One Time
            </h4>
            <p class="text-sm text-[#808080] leading-relaxed">
              Run this task once, manually triggered when needed.
            </p>
          </div>

          {props.runMode === 'one_time' && (
            <div class="absolute top-3 right-3">
              <div class="w-5 h-5 rounded-full bg-[#007acc] flex items-center justify-center">
                <svg class="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                </svg>
              </div>
            </div>
          )}
        </button>

        <button
          type="button"
          onClick={() => handleRunModeChange('scheduled')}
          class={cn(
            'group relative flex flex-col items-start gap-3 p-5 rounded-lg border-2 transition-all cursor-pointer text-left',
            'bg-[#2d2d2d] hover:bg-[#2d2d2d]/80',
            props.runMode === 'scheduled'
              ? 'border-[#007acc] ring-2 ring-[#007acc]/30'
              : 'border-[#3c3c3c] hover:border-[#007acc]/50'
          )}
        >
          <div
            class={cn(
              'w-12 h-12 rounded-lg flex items-center justify-center transition-colors',
              props.runMode === 'scheduled'
                ? 'bg-[#007acc] text-white'
                : 'bg-[#3c3c3c] text-[#808080] group-hover:bg-[#007acc]/20 group-hover:text-[#007acc]'
            )}
          >
            <Calendar class="w-6 h-6" />
          </div>

          <div class="flex-1">
            <h4 class="text-base font-medium text-[#cccccc] mb-1">
              Scheduled
            </h4>
            <p class="text-sm text-[#808080] leading-relaxed">
              Run this task automatically on a recurring schedule.
            </p>
          </div>

          {props.runMode === 'scheduled' && (
            <div class="absolute top-3 right-3">
              <div class="w-5 h-5 rounded-full bg-[#007acc] flex items-center justify-center">
                <svg class="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                </svg>
              </div>
            </div>
          )}
        </button>
      </div>

      {/* Schedule Configuration (shown when scheduled mode is selected) */}
      <Show when={props.runMode === 'scheduled'}>
        <div class="space-y-4 p-5 bg-[#252525] border border-[#3c3c3c] rounded-lg">
          <h4 class="text-sm font-medium text-[#cccccc] flex items-center gap-2">
            <Calendar class="w-4 h-4" />
            Schedule Configuration
          </h4>

          {/* Day Selection */}
          <div>
            <label class="text-xs font-medium text-[#cccccc] mb-2 block">
              Days of Week
            </label>
            <div class="flex flex-wrap gap-2">
              <For each={DAYS_OF_WEEK}>
                {(day) => (
                  <button
                    type="button"
                    onClick={() => toggleDay(day.value)}
                    class={cn(
                      'px-3 py-2 rounded-md text-xs font-medium transition-colors border',
                      selectedDays().includes(day.value)
                        ? 'bg-[#007acc] border-[#007acc] text-white'
                        : 'bg-[#2d2d2d] border-[#3c3c3c] text-[#cccccc] hover:border-[#007acc]/50'
                    )}
                  >
                    {day.label}
                  </button>
                )}
              </For>
            </div>
            <Show when={selectedDays().length === 0}>
              <p class="text-xs text-[#c72e0f] mt-2">
                Please select at least one day.
              </p>
            </Show>
          </div>

          {/* Time Selection */}
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-xs font-medium text-[#cccccc] flex items-center gap-2">
                <Clock class="w-4 h-4" />
                Times
              </label>
              <button
                type="button"
                onClick={addTime}
                class={cn(
                  'flex items-center gap-1 px-2 py-1 text-xs font-medium rounded transition-colors',
                  'bg-[#007acc]/20 text-[#007acc] hover:bg-[#007acc]/30'
                )}
              >
                <Plus class="w-3 h-3" />
                Add Time
              </button>
            </div>
            <div class="space-y-2">
              <For each={times()}>
                {(timeValue, index) => (
                  <div class="flex items-center gap-2">
                    <TimeInput
                      value={timeValue}
                      onChange={(newValue) => handleTimeChange(index(), newValue)}
                      class="flex-1"
                    />
                    <Show when={times().length > 1}>
                      <button
                        type="button"
                        onClick={() => removeTime(index())}
                        class={cn(
                          'p-2 rounded-md transition-colors',
                          'bg-[#3c3c3c] text-[#808080] hover:bg-[#c72e0f]/20 hover:text-[#c72e0f]'
                        )}
                        title="Remove time"
                      >
                        <X class="w-4 h-4" />
                      </button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* Schedule Preview */}
          <Show when={selectedDays().length > 0 && times().length > 0}>
            <div class="bg-[#2d2d2d] border border-[#3c3c3c] rounded-md p-3">
              <p class="text-xs text-[#cccccc] mb-1">
                <strong>Schedule Preview:</strong>
              </p>
              <p class="text-xs text-[#808080]">
                Runs every{' '}
                {selectedDays()
                  .map(d => DAYS_OF_WEEK.find(day => day.value === d)?.label)
                  .join(', ')}{' '}
                at {times().sort().join(', ')}
              </p>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};
