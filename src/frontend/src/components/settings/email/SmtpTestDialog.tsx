import { type Component, type Accessor, type Setter, Show } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '~/components/ui/dialog';
import type { SmtpConfig } from '~/types';

export type TestStatus = 'idle' | 'testing' | 'success' | 'error';

interface SmtpTestDialogProps {
  open: Accessor<boolean>;
  onOpenChange: Setter<boolean>;
  config: Accessor<SmtpConfig | null>;
  testRecipient: Accessor<string>;
  setTestRecipient: Setter<string>;
  testStatus: Accessor<TestStatus>;
  testMessage: Accessor<string>;
  onTest: () => void;
}

const inputClass = "h-[28px] w-full px-2 bg-sidebar border border-border text-foreground text-[12px] rounded-none focus:outline-none focus:border-primary placeholder:text-muted-foreground";
const labelClass = "text-[11px] text-muted-foreground mb-1 block";

export const SmtpTestDialog: Component<SmtpTestDialogProps> = (props) => {
  return (
    <Dialog open={props.open()} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[400px]">
        <DialogHeader>
          <DialogTitle>Test SMTP Connection</DialogTitle>
          <button
            type="button"
            class="text-muted-foreground hover:text-foreground p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <DialogDescription class="px-4 py-2 text-[11px] text-muted-foreground">
          Send a test email using <strong class="text-foreground">"{props.config()?.name}"</strong>
        </DialogDescription>

        <div class="px-4 py-3 space-y-3">
          <div>
            <label class={labelClass}>Test Recipient</label>
            <input
              type="email"
              class={inputClass}
              value={props.testRecipient()}
              onInput={(e) => props.setTestRecipient(e.currentTarget.value)}
              placeholder="test@example.com"
            />
          </div>

          <Show when={props.testStatus() === 'success'}>
            <div class="flex items-center gap-2 px-3 py-2 rounded-[3px] bg-[#388a34]/15 border border-[#388a34]/30 text-[11px] text-[#4ec9b0]">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
              </svg>
              {props.testMessage()}
            </div>
          </Show>

          <Show when={props.testStatus() === 'error'}>
            <div class="flex items-start gap-2 px-3 py-2 rounded-[3px] bg-[#ef4444]/15 border border-[#ef4444]/30 text-[11px] text-[#ef4444]">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor" class="shrink-0 mt-0.5">
                <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/>
              </svg>
              {props.testMessage()}
            </div>
          </Show>
        </div>

        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-border bg-accent text-foreground hover:bg-accent hover:border-border transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            Close
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-primary bg-primary text-white hover:bg-[#2563eb] hover:border-[#2563eb] disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-1.5"
            onClick={props.onTest}
            disabled={props.testStatus() === 'testing' || !props.testRecipient()}
          >
            {props.testStatus() === 'testing' ? (
              <>
                <svg class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Testing...
              </>
            ) : (
              <>
                <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M15.854.146a.5.5 0 0 1 .11.54l-5.819 14.547a.75.75 0 0 1-1.329.124l-3.178-4.995L.643 7.184a.75.75 0 0 1 .124-1.33L15.314.037a.5.5 0 0 1 .54.11ZM6.636 10.07l2.761 4.338L14.13 2.576 6.636 10.07Zm6.787-8.201L1.591 6.602l4.339 2.76 7.494-7.493Z"/>
                </svg>
                Send Test Email
              </>
            )}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
