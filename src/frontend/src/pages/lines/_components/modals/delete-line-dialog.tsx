import { type Component } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '~/components/ui/dialog';
import type { Line } from '~/types';

export interface DeleteLineDialogProps {
  open: boolean;
  line: Line | null;
  onOpenChange: (open: boolean) => void;
  onDelete: () => void;
}

export const DeleteLineDialog: Component<DeleteLineDialogProps> = (props) => {
  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[400px]">
        <DialogHeader>
          <DialogTitle>Delete Line</DialogTitle>
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
            <div class="flex-shrink-0 w-8 h-8 rounded-[3px] bg-[#c72e0f]/15 flex items-center justify-center">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="#c72e0f"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/></svg>
            </div>
            <div>
              <p class="text-[13px] text-[#cccccc] mb-1">
                Are you sure you want to delete <strong class="text-white">"{props.line?.name}"</strong>?
              </p>
              <p class="text-[11px] text-[#808080]">
                This action cannot be undone. All associated data will be permanently removed.
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
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#c72e0f] bg-[#c72e0f] text-white hover:bg-[#d9534f] hover:border-[#d9534f] transition-colors"
            onClick={props.onDelete}
          >
            Delete
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
