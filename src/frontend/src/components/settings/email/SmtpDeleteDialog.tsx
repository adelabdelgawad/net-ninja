import { type Component, type Accessor, type Setter } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '~/components/ui/dialog';
import type { SmtpConfig } from '~/types';

interface SmtpDeleteDialogProps {
  open: Accessor<boolean>;
  onOpenChange: Setter<boolean>;
  config: Accessor<SmtpConfig | null>;
  onDelete: () => void;
  isLoading: Accessor<boolean>;
}

export const SmtpDeleteDialog: Component<SmtpDeleteDialogProps> = (props) => {
  return (
    <Dialog open={props.open()} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[400px]">
        <DialogHeader>
          <DialogTitle>Delete SMTP Server</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <DialogDescription class="px-4 py-2 text-[11px] text-[#808080]">
          Are you sure you want to delete <strong class="text-[#cccccc]">"{props.config()?.name}"</strong>? This action cannot be undone.
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
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#c72e0f] bg-[#c72e0f] text-white hover:bg-[#d9534f] hover:border-[#d9534f] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={props.onDelete}
            disabled={props.isLoading()}
          >
            {props.isLoading() ? (
              <>
                <svg class="inline-block w-3 h-3 mr-1.5 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Deleting...
              </>
            ) : (
              <>
                <svg class="inline-block w-3 h-3 mr-1.5" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"/>
                  <path fill-rule="evenodd" d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1zM4.118 4 4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/>
                </svg>
                Delete
              </>
            )}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
