import { type Component } from 'solid-js';
import type { Email } from '~/types';
import { Switch } from '@kobalte/core';

interface EmailRecipientRowProps {
  email: Email;
  onEdit: () => void;
  onToggleActive: () => void;
  onDelete: () => void;
}

export const EmailRecipientRow: Component<EmailRecipientRowProps> = (props) => {
  return (
    <div class="group flex items-center gap-3 px-4 py-3 hover:bg-[#2d2d2d] transition-colors rounded-[8px]">
      {/* User icon — green: active, grey: inactive */}
      <div
        class="flex items-center justify-center h-9 w-9 rounded-[10px] shrink-0"
        classList={{
          'bg-[#26a269]/15': props.email.isActive,
          'bg-[#808080]/10': !props.email.isActive,
        }}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill={props.email.isActive ? '#57e389' : '#808080'}>
          <path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm2-3a2 2 0 1 1-4 0 2 2 0 0 1 4 0zm4 8c0 1-1 1-1 1H3s-1 0-1-1 1-4 6-4 6 3 6 4zm-1-.004c-.001-.246-.154-.986-.832-1.664C11.516 10.68 10.289 10 8 10c-2.29 0-3.516.68-4.168 1.332-.678.678-.83 1.418-.832 1.664h10z"/>
        </svg>
      </div>

      {/* Content */}
      <div class="flex-1 min-w-0">
        <span class="text-[13px] font-medium text-[#eeeeee] truncate block">
          {props.email.name || 'Unnamed'}
        </span>
        <span class="text-[11px] text-[#999999] truncate block mt-0.5">
          {props.email.recipient}
        </span>
      </div>

      {/* Active toggle */}
      <div class="flex items-center gap-2 shrink-0">
        <span class="text-[11px] text-[#999999]">
          {props.email.isActive ? 'Active' : 'Inactive'}
        </span>
        <Switch.Root
          checked={props.email.isActive}
          onChange={props.onToggleActive}
          class="relative inline-flex h-[18px] w-[32px] items-center rounded-full bg-[#3c3c3c] transition-colors data-[checked]:bg-[#3584e4] cursor-pointer"
        >
          <Switch.Thumb class="h-[14px] w-[14px] transform rounded-full bg-white transition-transform data-[checked]:translate-x-[16px] data-[unchecked]:translate-x-[2px]" />
        </Switch.Root>
      </div>

      {/* Actions */}
      <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
        <button
          type="button"
          class="h-[28px] px-2.5 flex items-center gap-1.5 rounded-[6px] text-[11px] font-medium border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#555555] transition-colors"
          onClick={props.onEdit}
        >
          <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
            <path d="M13.23 1h-1.46L3.52 9.25l-.16.22L1 13.59 2.41 15l4.12-2.36.22-.16L15 4.23V2.77L13.23 1zM2.41 13.59l1.51-3 1.45 1.45-2.96 1.55zm3.83-2.06L4.47 9.76l6-6 1.77 1.77-6 6z"/>
          </svg>
          Edit
        </button>
        <button
          type="button"
          class="h-[28px] px-2.5 flex items-center rounded-[6px] text-[11px] font-medium border border-[#3c3c3c] bg-[#2d2d2d] text-[#808080] hover:bg-[#c72e0f]/15 hover:text-[#c72e0f] hover:border-[#c72e0f]/30 transition-colors"
          onClick={props.onDelete}
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"/>
            <path fill-rule="evenodd" d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1zM4.118 4 4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/>
          </svg>
        </button>
      </div>
    </div>
  );
};
