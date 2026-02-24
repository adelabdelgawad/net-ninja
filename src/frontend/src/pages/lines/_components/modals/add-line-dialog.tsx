import type { Component } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '~/components/ui/dialog';
import type { LineCreate } from '~/types';
import {
  BasicInfoSection,
  NetworkSection,
  CredentialsSection,
} from './sections';

export interface AddLineDialogProps {
  open: boolean;
  formData: LineCreate;
  isEditing: boolean;
  saving: boolean;
  onOpenChange: (open: boolean) => void;
  onUpdateField: (field: keyof LineCreate, value: string) => void;
  onSave: () => void;
}

export const AddLineDialog: Component<AddLineDialogProps> = (props) => {
  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[520px]">
        <DialogHeader>
          <DialogTitle>{props.isEditing ? 'Edit Line' : 'Add Line'}</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>

        <div class="px-4 py-3 space-y-4">
          <BasicInfoSection
            formData={props.formData}
            onUpdateField={props.onUpdateField}
          />
          <div class="h-px bg-[#3c3c3c]" />
          <NetworkSection
            formData={props.formData}
            onUpdateField={props.onUpdateField}
          />
          <div class="h-px bg-[#3c3c3c]" />
          <CredentialsSection
            formData={props.formData}
            onUpdateField={props.onUpdateField}
          />
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
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={props.onSave}
            disabled={props.saving}
          >
            {props.saving ? 'Saving...' : props.isEditing ? 'Save Changes' : 'Add Line'}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
