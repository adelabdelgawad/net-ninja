import type { Component } from 'solid-js';
import { Play, Calendar } from 'lucide-solid';
import type { TaskExecutionResponse } from '~/types';
import { ExecutionStatusIcon } from '~/lib/statusIcons';
import { cn } from '~/lib/utils';

interface RecentExecutionProps {
  execution: TaskExecutionResponse;
}

/**
 * RecentExecution component - compact row displaying task execution info.
 * Shows status icon, task name, trigger badge, and timestamp.
 */
export const RecentExecution: Component<RecentExecutionProps> = (props) => {
  const formatTimestamp = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);

    if (diffMinutes < 1) {
      return 'just now';
    } else if (diffMinutes < 60) {
      return `${diffMinutes}m ago`;
    } else if (diffHours < 24) {
      return `${diffHours}h ago`;
    } else {
      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
  };

  return (
    <div class="h-8 px-3 py-2 flex items-center justify-between border-b border-border/50 hover:bg-muted/30 transition-colors">
      {/* Left: Status icon + task name */}
      <div class="flex items-center gap-2 flex-1 min-w-0">
        <ExecutionStatusIcon status={props.execution.status} class="flex-shrink-0" />
        <span class="text-xs font-medium truncate">
          {props.execution.taskName}
        </span>
      </div>

      {/* Right: Trigger badge + timestamp */}
      <div class="flex items-center gap-2 flex-shrink-0">
        <div class={cn(
          'flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium',
          props.execution.triggeredBy === 'manual'
            ? 'bg-blue-500/10 text-blue-500'
            : 'bg-purple-500/10 text-purple-500'
        )}>
          {props.execution.triggeredBy === 'manual' ? (
            <Play class="h-2.5 w-2.5" />
          ) : (
            <Calendar class="h-2.5 w-2.5" />
          )}
          <span class="uppercase tracking-wide">
            {props.execution.triggeredBy === 'manual' ? 'Manual' : 'Scheduled'}
          </span>
        </div>
        <span class="text-xs text-muted-foreground font-mono">
          {formatTimestamp(props.execution.startedAt)}
        </span>
      </div>
    </div>
  );
};
