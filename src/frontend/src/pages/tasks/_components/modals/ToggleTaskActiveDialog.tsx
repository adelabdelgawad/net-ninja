import { type Component, createSignal } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '~/components/ui/dialog';
import type { Task } from '~/types';

interface ToggleTaskActiveDialogProps {
  open: boolean;
  task: Task | null;
  onOpenChange: (open: boolean) => void;
  onConfirm: (task: Task, isActive: boolean) => Promise<void>;
}

export const ToggleTaskActiveDialog: Component<ToggleTaskActiveDialogProps> = (props) => {
  const [toggling, setToggling] = createSignal(false);
  const willActivate = () => !(props.task?.isActive ?? true);

  const handleConfirm = async () => {
    const task = props.task;
    if (!task) return;
    setToggling(true);
    try {
      await props.onConfirm(task, !task.isActive);
      props.onOpenChange(false);
    } catch {
      // error handled by parent
    } finally {
      setToggling(false);
    }
  };

  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[400px]">
        <DialogHeader>
          <DialogTitle>{willActivate() ? 'Activate' : 'Deactivate'} Task</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>

        <div class="px-4 py-4">
          <div class="flex items-start gap-3">
            <div class={`flex-shrink-0 w-8 h-8 rounded-[3px] flex items-center justify-center ${willActivate() ? 'bg-[#26a269]/15' : 'bg-[#e5a50a]/15'}`}>
              {willActivate() ? (
                <svg width="16" height="16" viewBox="0 0 16 16" fill="#26a269"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm3.354 4.146l-.708-.708L7 9.086 5.354 7.44l-.708.708 2.354 2.354 4.354-4.356z"/></svg>
              ) : (
                <svg width="16" height="16" viewBox="0 0 16 16" fill="#e5a50a"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/></svg>
              )}
            </div>
            <div>
              <p class="text-[13px] text-[#cccccc] mb-1">
                Are you sure you want to {willActivate() ? 'activate' : 'deactivate'}{' '}
                <strong class="text-white">"{props.task?.name}"</strong>?
              </p>
              <p class="text-[11px] text-[#808080]">
                {willActivate()
                  ? 'This task will resume its scheduled executions.'
                  : 'This task will be paused and will not run on schedule.'}
              </p>
            </div>
          </div>
        </div>

        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            Cancel
          </button>
          <button
            type="button"
            class={`h-[26px] px-3 text-[12px] rounded-[3px] border text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
              willActivate()
                ? 'border-[#26a269] bg-[#26a269] hover:bg-[#2eb878] hover:border-[#2eb878]'
                : 'border-[#e5a50a] bg-[#e5a50a] hover:bg-[#f0b820] hover:border-[#f0b820]'
            }`}
            disabled={toggling()}
            onClick={handleConfirm}
          >
            {toggling() ? 'Saving...' : willActivate() ? 'Activate' : 'Deactivate'}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
