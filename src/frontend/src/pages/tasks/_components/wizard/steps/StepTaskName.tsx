import { type Component, createSignal, createEffect, Show } from 'solid-js';
import { tasksApi } from '~/api/client';
import { cn } from '~/lib/utils';

export interface StepTaskNameProps {
  value: string;
  onChange: (value: string) => void;
  isValid: boolean;
}

const inputClass = "h-[34px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555] pr-10";

export const StepTaskName: Component<StepTaskNameProps> = (props) => {
  const [checking, setChecking] = createSignal(false);
  const [isAvailable, setIsAvailable] = createSignal<boolean | null>(null);
  const [checkError, setCheckError] = createSignal<string | null>(null);

  let timeoutId: number | undefined;

  // Debounced name availability check
  createEffect(() => {
    const name = props.value.trim();

    if (timeoutId) {
      clearTimeout(timeoutId);
    }

    if (name.length === 0) {
      setIsAvailable(null);
      setCheckError(null);
      return;
    }

    setChecking(true);
    timeoutId = window.setTimeout(async () => {
      try {
        const available = await tasksApi.checkNameAvailable(name);
        setIsAvailable(available);
        setCheckError(null);
      } catch (error) {
        setCheckError('Failed to check name availability');
        setIsAvailable(null);
      } finally {
        setChecking(false);
      }
    }, 500);
  });

  const showValidationIcon = () => {
    if (!props.value.trim()) return null;
    if (checking()) return 'checking';
    if (isAvailable() === true) return 'valid';
    if (isAvailable() === false) return 'invalid';
    return null;
  };

  return (
    <div class="space-y-4">
      <div>
        <h3 class="text-[13px] font-medium text-[#cccccc] mb-1">Task Name</h3>
        <p class="text-[11px] text-[#808080]">
          Enter a unique, descriptive name for your task. This will help you identify it later.
        </p>
      </div>

      <div>
        <label class="text-[11px] text-[#808080] mb-1 block">Name</label>
        <div class="relative">
          <input
            type="text"
            class={cn(
              inputClass,
              isAvailable() === false && 'border-[#c72e0f]',
              isAvailable() === true && 'border-[#388a34]'
            )}
            value={props.value}
            onInput={(e) => props.onChange(e.currentTarget.value)}
            placeholder="e.g., Daily Office Speed Test"
            autofocus
          />
          <div class="absolute right-2 top-1/2 -translate-y-1/2">
            <Show when={showValidationIcon() === 'checking'}>
              <svg class="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
            </Show>
            <Show when={showValidationIcon() === 'valid'}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="#388a34">
                <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
              </svg>
            </Show>
            <Show when={showValidationIcon() === 'invalid'}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="#c72e0f">
                <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/>
              </svg>
            </Show>
          </div>
        </div>
        <Show when={isAvailable() === false}>
          <p class="text-[10px] text-[#c72e0f] mt-1">
            This task name is already taken. Please choose a different name.
          </p>
        </Show>
        <Show when={isAvailable() === true}>
          <p class="text-[10px] text-[#388a34] mt-1">
            This name is available.
          </p>
        </Show>
        <Show when={checkError()}>
          <p class="text-[10px] text-[#c72e0f] mt-1">
            {checkError()}
          </p>
        </Show>
      </div>

      <div class="bg-[#2d2d2d] border border-[#3c3c3c] rounded-[3px] px-3 py-2">
        <h4 class="text-[10px] text-[#cccccc] mb-1.5">Naming Tips:</h4>
        <ul class="text-[10px] text-[#808080] space-y-1">
          <li class="flex gap-2">
            <span class="text-[#007acc]">•</span>
            <span>Use descriptive names that indicate purpose and scope</span>
          </li>
          <li class="flex gap-2">
            <span class="text-[#007acc]">•</span>
            <span>Include location, frequency, or line identifiers if helpful</span>
          </li>
          <li class="flex gap-2">
            <span class="text-[#007acc]">•</span>
            <span>Examples: "Main Office Daily Check", "Backup Line Speed Test"</span>
          </li>
        </ul>
      </div>
    </div>
  );
};
