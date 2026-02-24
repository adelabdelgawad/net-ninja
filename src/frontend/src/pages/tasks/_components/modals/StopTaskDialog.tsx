import { type Component, type Accessor } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '~/components/ui/dialog';
import type { Task } from '~/types';

interface StopTaskDialogProps {
  open: Accessor<boolean>;
  onOpenChange: (open: boolean) => void;
  task: Accessor<Task | null>;
  onConfirm: (task: Task) => Promise<void>;
}

export const StopTaskDialog: Component<StopTaskDialogProps> = (props) => {
  const handleConfirm = async () => {
    const task = props.task();
    if (!task) return;

    try {
      await props.onConfirm(task);
      props.onOpenChange(false);
    } catch (error) {
      console.error('Failed to stop task:', error);
    }
  };

  return (
    <Dialog open={props.open()} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[440px]">
        <DialogHeader>
          <DialogTitle>Stop Running Task?</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <DialogDescription class="px-4 py-4 text-[11px] text-[#cccccc]">
          This will terminate open browsers and cancel the running speedtest for task <span class="font-semibold text-[#569cd6]">"{props.task()?.name}"</span>.
        </DialogDescription>

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
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#f14c4c] bg-[#f14c4c] text-white hover:bg-[#d43535] hover:border-[#d43535] transition-colors"
            onClick={handleConfirm}
          >
            Stop
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
