import type { Component } from 'solid-js';
import { For } from 'solid-js';
import type { Task } from '~/types';
import { getNextRunTime, formatNextRun } from '~/lib/scheduleUtils';
import { cn } from '~/lib/utils';

interface UpcomingTaskProps {
  task: Task;
}

/**
 * UpcomingTask component - compact row displaying scheduled task info.
 * Shows task name, task type badges, and next run time.
 */
export const UpcomingTask: Component<UpcomingTaskProps> = (props) => {
  const nextRun = () => getNextRunTime(props.task.schedule);
  const nextRunFormatted = () => formatNextRun(nextRun());

  const taskTypeBadge = (taskType: 'speed_test' | 'quota_check') => {
    const config = taskType === 'speed_test'
      ? { label: 'Speed', color: 'bg-cyan-500/10 text-cyan-500' }
      : { label: 'Quota', color: 'bg-amber-500/10 text-amber-500' };

    return (
      <span class={cn(
        'px-1.5 py-0.5 rounded text-[10px] font-medium uppercase tracking-wide',
        config.color
      )}>
        {config.label}
      </span>
    );
  };

  return (
    <div class="h-8 px-3 py-2 flex items-center justify-between border-b border-border/50 hover:bg-muted/30 transition-colors">
      {/* Left: Task name + task type badges */}
      <div class="flex items-center gap-2 flex-1 min-w-0">
        <span class="text-xs font-medium truncate">
          {props.task.name}
        </span>
        <div class="flex items-center gap-1 flex-shrink-0">
          <For each={props.task.taskTypes}>
            {(taskType) => taskTypeBadge(taskType)}
          </For>
        </div>
      </div>

      {/* Right: Next run time */}
      <div class="flex-shrink-0">
        <span class="text-xs text-muted-foreground font-mono">
          {nextRunFormatted()}
        </span>
      </div>
    </div>
  );
};
