import { type Component, type Resource, Show, For } from 'solid-js';
import type { Email } from '~/types';
import { EmailRecipientRow } from './EmailRecipientRow';
import { EmailRecipientEmpty } from './EmailRecipientEmpty';

interface EmailRecipientListProps {
  emails: Resource<Email[] | undefined>;
  onAddRecipient: () => void;
  onEdit: (email: Email) => void;
  onToggleActive: (email: Email) => void;
  onDelete: (email: Email) => void;
}

export const EmailRecipientList: Component<EmailRecipientListProps> = (props) => {
  return (
    <div>
      {/* Loading state */}
      <Show when={props.emails.loading}>
        <div class="flex items-center justify-center py-8 text-[#999999]">
          <svg class="w-4 h-4 animate-spin mr-2" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          <span class="text-[13px]">Loading email recipients...</span>
        </div>
      </Show>

      {/* Error state */}
      <Show when={props.emails.error}>
        <div class="rounded-[8px] bg-[#c72e0f]/10 px-4 py-3 text-[13px] text-[#f48771]">
          Failed to load email recipients
        </div>
      </Show>

      {/* Empty state */}
      <Show when={!props.emails.loading && !props.emails.error && props.emails()?.length === 0}>
        <EmailRecipientEmpty onAddRecipient={props.onAddRecipient} />
      </Show>

      {/* Email list */}
      <Show when={!props.emails.loading && !props.emails.error && (props.emails()?.length ?? 0) > 0}>
        <div class="rounded-[10px] bg-[#1e1e1e] border border-[#2a2a2a] overflow-hidden">
          <For each={props.emails()}>
            {(email, index) => (
              <>
                <Show when={index() > 0}>
                  <div class="h-px bg-[#2a2a2a] mx-4" />
                </Show>
                <EmailRecipientRow
                  email={email}
                  onEdit={() => props.onEdit(email)}
                  onToggleActive={() => props.onToggleActive(email)}
                  onDelete={() => props.onDelete(email)}
                />
              </>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
