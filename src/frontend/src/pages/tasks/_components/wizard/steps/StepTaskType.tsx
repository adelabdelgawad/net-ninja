import { type Component, For, Show } from 'solid-js';
import { cn } from '~/lib/utils';
import { Zap, BarChart3, Eye } from 'lucide-solid';

export interface StepTaskTypeProps {
  value: ('speed_test' | 'quota_check')[];
  onChange: (value: ('speed_test' | 'quota_check')[]) => void;
  showBrowser: boolean;
  onShowBrowserChange: (value: boolean) => void;
}

interface TaskTypeOption {
  value: 'speed_test' | 'quota_check';
  title: string;
  description: string;
  icon: Component<{ class?: string }>;
}

const taskTypeOptions: TaskTypeOption[] = [
  {
    value: 'speed_test',
    title: 'Speed Test',
    description: 'Test download and upload speeds for selected lines. Measures bandwidth performance.',
    icon: Zap,
  },
  {
    value: 'quota_check',
    title: 'Quota Check',
    description: 'Check data quota usage for selected lines. Monitors remaining data allowance.',
    icon: BarChart3,
  },
];

export const StepTaskType: Component<StepTaskTypeProps> = (props) => {
  const isSelected = (optionValue: 'speed_test' | 'quota_check') => {
    return props.value.includes(optionValue);
  };

  const toggleSelection = (optionValue: 'speed_test' | 'quota_check') => {
    const isSelected = props.value.includes(optionValue);
    if (isSelected) {
      // Remove if already selected (but prevent deselecting the last one)
      if (props.value.length > 1) {
        props.onChange(props.value.filter(v => v !== optionValue));
      }
    } else {
      // Add to selection
      props.onChange([...props.value, optionValue]);
    }
  };

  const selectedCount = () => props.value.length;

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-medium text-[#cccccc] mb-2">Task Type</h3>
          <p class="text-sm text-[#808080]">
            Choose what type of test or check this task should perform.
          </p>
        </div>
        <div class="text-sm text-[#cccccc]">
          {selectedCount()} of {taskTypeOptions.length} selected
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <For each={taskTypeOptions}>
          {(option) => (
            <button
              type="button"
              onClick={() => toggleSelection(option.value)}
              class={cn(
                'group relative flex flex-col items-start gap-3 p-5 rounded-lg border-2 transition-all cursor-pointer text-left',
                'bg-[#2d2d2d] hover:bg-[#2d2d2d]/80',
                isSelected(option.value)
                  ? 'border-[#007acc] ring-2 ring-[#007acc]/30'
                  : 'border-[#3c3c3c] hover:border-[#007acc]/50'
              )}
            >
              <div
                class={cn(
                  'w-12 h-12 rounded-lg flex items-center justify-center transition-colors',
                  isSelected(option.value)
                    ? 'bg-[#007acc] text-white'
                    : 'bg-[#3c3c3c] text-[#808080] group-hover:bg-[#007acc]/20 group-hover:text-[#007acc]'
                )}
              >
                <option.icon class="w-6 h-6" />
              </div>

              <div class="flex-1">
                <h4 class="text-base font-medium text-[#cccccc] mb-1">
                  {option.title}
                </h4>
                <p class="text-sm text-[#808080] leading-relaxed">
                  {option.description}
                </p>
              </div>

              {/* Checkbox indicator */}
              <div class="absolute top-3 right-3">
                <div
                  class={cn(
                    'w-5 h-5 rounded border-2 flex items-center justify-center transition-colors',
                    isSelected(option.value)
                      ? 'bg-[#007acc] border-[#007acc]'
                      : 'bg-transparent border-[#3c3c3c]'
                  )}
                >
                  {isSelected(option.value) && (
                    <svg class="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                    </svg>
                  )}
                </div>
              </div>
            </button>
          )}
        </For>
      </div>

      <div class="bg-[#2d2d2d] border border-[#3c3c3c] rounded-md p-4">
        <h4 class="text-xs font-medium text-[#cccccc] mb-2">About Task Types:</h4>
        <ul class="text-xs text-[#808080] space-y-1.5">
          <li class="flex gap-2">
            <span class="text-[#007acc] shrink-0">•</span>
            <span><strong class="text-[#cccccc]">Speed Test:</strong> Measures network performance including download speed, upload speed, and latency.</span>
          </li>
          <li class="flex gap-2">
            <span class="text-[#007acc] shrink-0">•</span>
            <span><strong class="text-[#cccccc]">Quota Check:</strong> Scrapes ISP portal data to track data usage and remaining quota.</span>
          </li>
          <li class="flex gap-2">
            <span class="text-[#007acc] shrink-0">•</span>
            <span><strong class="text-[#cccccc]">Multiple Types:</strong> You can select both types to run both tests in a single task.</span>
          </li>
        </ul>
      </div>

      {/* Browser visibility option (only for quota check) */}
      <Show when={props.value.includes('quota_check')}>
        <div class="bg-[#2d2d2d] border border-[#3c3c3c] rounded-md p-4">
          <div class="flex items-start gap-3">
            <button
              type="button"
              onClick={() => props.onShowBrowserChange(!props.showBrowser)}
              class={cn(
                'flex-shrink-0 w-5 h-5 mt-0.5 rounded border-2 flex items-center justify-center transition-all',
                props.showBrowser
                  ? 'bg-[#007acc] border-[#007acc]'
                  : 'bg-transparent border-[#3c3c3c] hover:border-[#007acc]'
              )}
            >
              {props.showBrowser && (
                <svg class="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                </svg>
              )}
            </button>
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <Eye class="w-4 h-4 text-[#cccccc]" />
                <h4 class="text-sm font-medium text-[#cccccc]">Show Browser Window</h4>
              </div>
              <p class="text-xs text-[#808080] leading-relaxed">
                Display the browser window during quota check. By default, the browser runs invisibly in the background. Enable this option to see the browser in action (useful for troubleshooting).
              </p>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
