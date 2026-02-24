import { type Component, type Accessor, type Setter } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '~/components/ui/dialog';
import type { EmailCreate } from '~/types';

interface EmailRecipientFormDialogProps {
  open: Accessor<boolean>;
  onOpenChange: Setter<boolean>;
  mode: 'add' | 'edit';
  formData: Accessor<Partial<EmailCreate>>;
  updateFormField: <K extends keyof EmailCreate>(field: K, value: EmailCreate[K]) => void;
  onSubmit: () => void;
  isLoading: Accessor<boolean>;
}

const inputClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const labelClass = "text-[11px] text-[#808080] mb-1 block";

export const EmailRecipientFormDialog: Component<EmailRecipientFormDialogProps> = (props) => {
  const isAdd = () => props.mode === 'add';

  return (
    <Dialog open={props.open()} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[520px]">
        <DialogHeader>
          <DialogTitle>{isAdd() ? 'Add Email Recipient' : 'Edit Email Recipient'}</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <DialogDescription class="px-4 py-2 text-[11px] text-[#808080]">
          {isAdd()
            ? 'Add a new email address for notifications'
            : 'Update the email recipient information'}
        </DialogDescription>

        <div class="px-4 py-3 space-y-4">
          <div>
            <label class={labelClass}>Name</label>
            <input
              type="text"
              class={inputClass}
              value={props.formData().name || ''}
              onInput={(e) => props.updateFormField('name', e.currentTarget.value)}
              placeholder={isAdd() ? 'e.g., John Doe' : ''}
            />
          </div>
          <div>
            <label class={labelClass}>Email Address</label>
            <input
              type="email"
              class={inputClass}
              value={props.formData().recipient || ''}
              onInput={(e) => props.updateFormField('recipient', e.currentTarget.value)}
              placeholder={isAdd() ? 'john.doe@example.com' : ''}
            />
          </div>
        </div>

        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
            onClick={() => props.onOpenChange(false)}
            disabled={props.isLoading()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={props.onSubmit}
            disabled={props.isLoading()}
          >
            {props.isLoading() ? (
              <>
                <svg class="inline-block w-3 h-3 mr-1.5 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Saving...
              </>
            ) : (
              isAdd() ? 'Add Recipient' : 'Save Changes'
            )}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
