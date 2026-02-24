import { type Component, createResource, For, Show } from 'solid-js';
import { linesApi } from '~/api/client';
import { Check } from 'lucide-solid';
import { cn } from '~/lib/utils';

export interface StepLineSelectionProps {
  selectedLineIds: number[];
  onChange: (lineIds: number[]) => void;
}

export const StepLineSelection: Component<StepLineSelectionProps> = (props) => {
  const [lines] = createResource(async () => {
    try {
      return await linesApi.list();
    } catch (e) {
      console.error('[StepLineSelection] Failed to load lines:', e);
      return [];
    }
  });

  const handleToggle = (lineId: number) => {
    const currentIds = props.selectedLineIds;
    if (currentIds.includes(lineId)) {
      props.onChange(currentIds.filter(id => id !== lineId));
    } else {
      props.onChange([...currentIds, lineId]);
    }
  };

  const handleSelectAll = () => {
    const allLineIds = lines()?.map(l => l.id) || [];
    props.onChange(allLineIds);
  };

  const handleDeselectAll = () => {
    props.onChange([]);
  };

  const isAllSelected = () => {
    const lineList = lines() || [];
    return lineList.length > 0 && props.selectedLineIds.length === lineList.length;
  };

  const isSelected = (lineId: number) => props.selectedLineIds.includes(lineId);

  return (
    <div class="space-y-6">
      <div>
        <h3 class="text-lg font-medium text-[#cccccc] mb-2">Select Lines</h3>
        <p class="text-sm text-[#808080]">
          Choose which internet lines should be included in this task. You can select one or multiple lines.
        </p>
      </div>

      <Show when={!lines.loading && lines()}>
        <div class="flex items-center justify-between mb-3">
          <div class="text-xs text-[#808080]">
            {props.selectedLineIds.length} of {lines()?.length || 0} selected
          </div>
          <div class="flex gap-2">
            <button
              type="button"
              class="text-xs text-[#007acc] hover:underline disabled:text-[#3c3c3c] disabled:cursor-not-allowed"
              onClick={handleSelectAll}
              disabled={isAllSelected()}
            >
              Select All
            </button>
            <span class="text-xs text-[#3c3c3c]">|</span>
            <button
              type="button"
              class="text-xs text-[#007acc] hover:underline disabled:text-[#3c3c3c] disabled:cursor-not-allowed"
              onClick={handleDeselectAll}
              disabled={props.selectedLineIds.length === 0}
            >
              Deselect All
            </button>
          </div>
        </div>
      </Show>

      <div class="space-y-2 max-h-[350px] overflow-y-auto">
        <Show when={lines.loading}>
          <div class="text-center py-8 text-sm text-[#808080]">
            Loading lines...
          </div>
        </Show>

        <Show when={!lines.loading && lines()?.length === 0}>
          <div class="text-center py-8">
            <p class="text-sm text-[#808080] mb-2">No lines configured.</p>
            <p class="text-xs text-[#666666]">
              You need to create at least one line before creating a task.
            </p>
          </div>
        </Show>

        <Show when={!lines.loading && lines() && lines()!.length > 0}>
          <For each={lines()}>
            {(line) => (
              <button
                type="button"
                onClick={() => handleToggle(line.id)}
                onKeyPress={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    handleToggle(line.id);
                  }
                }}
                class={cn(
                  'group relative w-full flex items-start gap-3 p-3 rounded-md border transition-all cursor-pointer text-left',
                  'bg-[#2d2d2d] border-[#3c3c3c]',
                  // Hover state
                  'hover:border-[#007acc]/50 hover:bg-[#2d2d2d]/80 hover:shadow-sm',
                  // Focus state
                  'focus:outline-none focus:ring-2 focus:ring-[#007acc]/50 focus:ring-offset-2 focus:ring-offset-[#1e1e1e]',
                  // Active/pressed state
                  'active:scale-[0.99]',
                  // Selected state - call isSelected() directly for reactivity
                  isSelected(line.id) && 'border-[#007acc] bg-[#007acc]/10 shadow-sm'
                )}
                aria-pressed={isSelected(line.id)}
                role="checkbox"
                aria-checked={isSelected(line.id)}
              >
                <div
                  class={cn(
                    'w-5 h-5 flex items-center justify-center rounded border-2 transition-colors shrink-0 mt-0.5',
                    'border-[#3c3c3c] bg-[#1e1e1e]',
                    'group-hover:border-[#007acc]/50',
                    isSelected(line.id)
                      ? 'border-[#007acc] bg-[#007acc]'
                      : 'border-[#3c3c3c] bg-[#1e1e1e]'
                  )}
                >
                  <Show when={isSelected(line.id)}>
                    <Check class="w-3.5 h-3.5 text-white" />
                  </Show>
                </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-1">
                      <span class="text-sm font-medium text-[#cccccc]">
                        {line.name}
                      </span>
                      <span class="text-xs text-[#808080]">
                        ({line.lineNumber})
                      </span>
                    </div>
                    <div class="flex items-center gap-3 text-xs text-[#808080]">
                      <Show when={line.isp}>
                        <span>{line.isp}</span>
                      </Show>
                      <Show when={line.ipAddress}>
                        <span class="text-[#666666]">•</span>
                        <span>{line.ipAddress}</span>
                      </Show>
                    </div>
                    <Show when={line.description}>
                      <p class="text-xs text-[#666666] mt-1 line-clamp-1">
                        {line.description}
                      </p>
                    </Show>
                  </div>
                </button>
            )}
          </For>
        </Show>
      </div>

      <Show when={props.selectedLineIds.length === 0}>
        <div class="bg-[#5a5a00]/20 border border-[#5a5a00] rounded-md p-3">
          <p class="text-xs text-[#cccccc]">
            Please select at least one line to continue.
          </p>
        </div>
      </Show>
    </div>
  );
};
