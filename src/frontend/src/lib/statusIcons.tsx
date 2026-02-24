import { CheckCircle, XCircle, AlertCircle, Clock } from 'lucide-solid';
import type { Component } from 'solid-js';
import { cn } from './utils';

interface ExecutionStatusIconProps {
  status: 'running' | 'completed' | 'failed' | 'pending';
  class?: string;
}

/**
 * Execution status icon component with color-coding.
 * - success (completed): #4ec9b0 (teal)
 * - error (failed): #c72e0f (red)
 * - running: #dcdcaa (yellow) with pulse animation
 * - pending: gray
 */
export const ExecutionStatusIcon: Component<ExecutionStatusIconProps> = (props) => {
  const iconClass = () => {
    switch (props.status) {
      case 'completed':
        return cn('h-4 w-4 text-[#4ec9b0]', props.class); // success
      case 'failed':
        return cn('h-4 w-4 text-[#c72e0f]', props.class); // error
      case 'running':
        return cn('h-4 w-4 text-[#dcdcaa] animate-pulse', props.class); // running with pulse
      case 'pending':
        return cn('h-4 w-4 text-muted-foreground', props.class); // gray
      default:
        return cn('h-4 w-4 text-muted-foreground', props.class);
    }
  };

  return (
    <>
      {props.status === 'completed' && <CheckCircle class={iconClass()} />}
      {props.status === 'failed' && <XCircle class={iconClass()} />}
      {props.status === 'running' && <AlertCircle class={iconClass()} />}
      {props.status === 'pending' && <Clock class={iconClass()} />}
    </>
  );
};
