import { type Component } from 'solid-js';

interface SmtpServerEmptyProps {
  onAddServer: () => void;
}

export const SmtpServerEmpty: Component<SmtpServerEmptyProps> = (props) => {
  return (
    <div class="rounded-[10px] border border-dashed border-border bg-sidebar py-10 px-6 text-center">
      <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
        <svg width="20" height="20" viewBox="0 0 16 16" fill="#3b82f6">
          <path d="M2 4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4zm2-1a1 1 0 0 0-1 1v.217l5 3.125 5-3.125V4a1 1 0 0 0-1-1H4zm9 2.441-4.724 2.953a.5.5 0 0 1-.552 0L3 5.441V12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V5.441z"/>
        </svg>
      </div>
      <p class="mt-3 text-[14px] font-medium text-foreground">
        No SMTP servers configured
      </p>
      <p class="mt-1 text-[12px] text-muted-foreground">
        Add your first server to start sending email notifications
      </p>
      <button
        type="button"
        class="mt-4 h-[32px] px-5 text-[12px] font-medium rounded-[8px] bg-primary text-white hover:bg-[#2563eb] transition-colors"
        onClick={props.onAddServer}
      >
        Add Your First Server
      </button>
    </div>
  );
};
