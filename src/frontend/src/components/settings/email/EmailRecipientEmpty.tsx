import { type Component } from 'solid-js';

interface EmailRecipientEmptyProps {
  onAddRecipient: () => void;
}

export const EmailRecipientEmpty: Component<EmailRecipientEmptyProps> = (props) => {
  return (
    <div class="rounded-[10px] border border-dashed border-[#3c3c3c] bg-[#1e1e1e] py-10 px-6 text-center">
      <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-[#26a269]/10">
        <svg width="20" height="20" viewBox="0 0 16 16" fill="#57e389">
          <path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm2-3a2 2 0 1 1-4 0 2 2 0 0 1 4 0zm4 8c0 1-1 1-1 1H3s-1 0-1-1 1-4 6-4 6 3 6 4zm-1-.004c-.001-.246-.154-.986-.832-1.664C11.516 10.68 10.289 10 8 10c-2.29 0-3.516.68-4.168 1.332-.678.678-.83 1.418-.832 1.664h10z"/>
        </svg>
      </div>
      <p class="mt-3 text-[14px] font-medium text-[#cccccc]">
        No email recipients
      </p>
      <p class="mt-1 text-[12px] text-[#999999]">
        Add email addresses to receive notifications and reports
      </p>
      <button
        type="button"
        class="mt-4 h-[32px] px-5 text-[12px] font-medium rounded-[8px] bg-[#3584e4] text-white hover:bg-[#3987e5] transition-colors"
        onClick={props.onAddRecipient}
      >
        Add Recipient
      </button>
    </div>
  );
};
