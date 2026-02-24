import { type Component, Show } from 'solid-js';
import type { SmtpConfig } from '~/types';

interface SmtpServerActionsProps {
  config: SmtpConfig;
  onTest: () => void;
  onEdit: () => void;
  onSetDefault: () => void;
  onDelete: () => void;
}

const actionBtnClass = "h-[28px] px-2.5 flex items-center gap-1.5 rounded-[6px] text-[11px] font-medium border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#555555] transition-colors";

export const SmtpServerActions: Component<SmtpServerActionsProps> = (props) => {
  return (
    <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
      {/* Test */}
      <button type="button" class={actionBtnClass} onClick={props.onTest}>
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path d="M15.854.146a.5.5 0 0 1 .11.54l-5.819 14.547a.75.75 0 0 1-1.329.124l-3.178-4.995L.643 7.184a.75.75 0 0 1 .124-1.33L15.314.037a.5.5 0 0 1 .54.11ZM6.636 10.07l2.761 4.338L14.13 2.576 6.636 10.07Zm6.787-8.201L1.591 6.602l4.339 2.76 7.494-7.493Z"/>
        </svg>
        Test
      </button>

      {/* Edit */}
      <button type="button" class={actionBtnClass} onClick={props.onEdit}>
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path d="M13.23 1h-1.46L3.52 9.25l-.16.22L1 13.59 2.41 15l4.12-2.36.22-.16L15 4.23V2.77L13.23 1zM2.41 13.59l1.51-3 1.45 1.45-2.96 1.55zm3.83-2.06L4.47 9.76l6-6 1.77 1.77-6 6z"/>
        </svg>
        Edit
      </button>

      {/* Set Default */}
      <Show when={!props.config.isDefault}>
        <button type="button" class={actionBtnClass} onClick={props.onSetDefault}>
          <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.75.75 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25zm0 2.445L6.615 5.5a.75.75 0 0 1-.564.41l-3.097.45 2.24 2.184a.75.75 0 0 1 .216.664l-.528 3.084 2.769-1.456a.75.75 0 0 1 .698 0l2.77 1.456-.53-3.084a.75.75 0 0 1 .216-.664l2.24-2.183-3.096-.45a.75.75 0 0 1-.564-.41L8 2.694z"/>
          </svg>
        </button>
      </Show>

      {/* Delete */}
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
  );
};
