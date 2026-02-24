import { type Component } from 'solid-js';

interface SmtpServerEmptyProps {
  onAddServer: () => void;
}

export const SmtpServerEmpty: Component<SmtpServerEmptyProps> = (props) => {
  return (
    <div class="rounded-[10px] border border-dashed border-[#3c3c3c] bg-[#1e1e1e] py-10 px-6 text-center">
      <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-[#3584e4]/10">
        <svg width="20" height="20" viewBox="0 0 16 16" fill="#3584e4">
          <path d="M2 4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4zm2-1a1 1 0 0 0-1 1v.217l5 3.125 5-3.125V4a1 1 0 0 0-1-1H4zm9 2.441-4.724 2.953a.5.5 0 0 1-.552 0L3 5.441V12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V5.441z"/>
        </svg>
      </div>
      <p class="mt-3 text-[14px] font-medium text-[#cccccc]">
        No SMTP servers configured
      </p>
      <p class="mt-1 text-[12px] text-[#999999]">
        Add your first server to start sending email notifications
      </p>
      <button
        type="button"
        class="mt-4 h-[32px] px-5 text-[12px] font-medium rounded-[8px] bg-[#3584e4] text-white hover:bg-[#3987e5] transition-colors"
        onClick={props.onAddServer}
      >
        Add Your First Server
      </button>
    </div>
  );
};
