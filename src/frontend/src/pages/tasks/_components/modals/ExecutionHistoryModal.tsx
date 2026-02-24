import { type Component, createResource, For, Show, createSignal } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog';
import { Checkbox } from '~/components/ui/checkbox';
import { cn } from '~/lib/utils';
import { formatDateTime } from '~/lib/date';
import type { Task, TaskExecutionResponse } from '~/types';
import { taskExecutionsApi, taskNotificationApi, emailsApi, smtpConfigsApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';

export interface ExecutionHistoryModalProps {
  open: boolean;
  task: Task | null;
  onOpenChange: (open: boolean) => void;
}

const StatusIcon: Component<{ status: string }> = (props) => {
  switch (props.status) {
    case 'completed':
      return (
        <svg width="16" height="16" viewBox="0 0 16 16" fill="#4ec9b0">
          <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
        </svg>
      );
    case 'failed':
      return (
        <svg width="16" height="16" viewBox="0 0 16 16" fill="#c72e0f">
          <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/>
        </svg>
      );
    case 'running':
      return (
        <svg width="16" height="16" viewBox="0 0 16 16" fill="#dcdcaa">
          <circle cx="8" cy="8" r="3" />
        </svg>
      );
    default:
      return (
        <svg width="16" height="16" viewBox="0 0 16 16" fill="#808080">
          <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
          <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
        </svg>
      );
  }
};

const TriggerIcon: Component<{ trigger: string }> = (props) => {
  switch (props.trigger) {
    case 'manual':
      return (
        <svg width="12" height="12" viewBox="0 0 16 16" fill="#007acc">
          <path d="M10.804 8 5 4.633v6.734L10.804 8zm.792-.696a.802.802 0 0 1 0 1.392l-6.363 3.692C4.713 12.69 4 12.345 4 11.692V4.308c0-.653.713-.998 1.233-.696l6.363 3.692z"/>
        </svg>
      );
    case 'scheduler':
      return (
        <svg width="12" height="12" viewBox="0 0 16 16" fill="#4ec9b0">
          <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
          <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
        </svg>
      );
    default:
      return (
        <svg width="12" height="12" viewBox="0 0 16 16" fill="#808080">
          <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
          <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
        </svg>
      );
  }
};

const formatDuration = (ms: number | null) => {
  if (ms === null) return '-';
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
};

const selectClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] cursor-pointer";

// --- Resend Email Dialog ---

interface ResendEmailDialogProps {
  open: boolean;
  executionId: string;
  executionLabel: string;
  onOpenChange: (open: boolean) => void;
}

const ResendEmailDialog: Component<ResendEmailDialogProps> = (props) => {
  const [sending, setSending] = createSignal(false);
  const [smtpId, setSmtpId] = createSignal<number | undefined>();
  const [subject, setSubject] = createSignal('NetNinja Task Results');
  const [toIds, setToIds] = createSignal<number[]>([]);
  const [ccIds, setCcIds] = createSignal<number[]>([]);

  const [emails] = createResource(() => props.open ? true : null, () => emailsApi.list());
  const [smtpConfigs] = createResource(() => props.open ? true : null, () => smtpConfigsApi.list());

  const activeEmails = () => emails()?.filter(e => e.isActive) ?? [];
  const activeSmtpConfigs = () => smtpConfigs()?.filter(c => c.isActive) ?? [];

  const resetAndClose = () => {
    setSending(false);
    setSmtpId(undefined);
    setSubject('NetNinja Task Results');
    setToIds([]);
    setCcIds([]);
    props.onOpenChange(false);
  };

  const handleSend = async () => {
    const smtp = smtpId();
    if (!smtp || toIds().length === 0) {
      showToast({ title: 'Missing fields', description: 'Please select an SMTP server and at least one TO recipient.', variant: 'error', duration: 4000 });
      return;
    }

    setSending(true);
    try {
      await taskNotificationApi.resend({
        executionId: props.executionId,
        smtpConfigId: smtp,
        emailSubject: subject() || 'NetNinja Task Results',
        toRecipientIds: toIds(),
        ccRecipientIds: ccIds(),
      });
      showToast({ title: 'Email sent', description: 'Notification email sent successfully.', variant: 'success', duration: 3000 });
      resetAndClose();
    } catch (e) {
      showToast({ title: 'Failed to send', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setSending(false);
    }
  };

  const handleToggleTo = (emailId: number) => {
    const current = toIds();
    setToIds(current.includes(emailId) ? current.filter(id => id !== emailId) : [...current, emailId]);
  };

  const handleToggleCc = (emailId: number) => {
    const current = ccIds();
    setCcIds(current.includes(emailId) ? current.filter(id => id !== emailId) : [...current, emailId]);
  };

  return (
    <Dialog open={props.open} onOpenChange={(open) => { if (!open) resetAndClose(); else props.onOpenChange(open); }}>
      <DialogContent class="max-w-[440px]">
        <DialogHeader>
          <DialogTitle class="flex items-center gap-2">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M.05 3.555A2 2 0 0 1 2 2h12a2 2 0 0 1 1.95 1.555L8 8.414.05 3.555ZM0 4.697v7.104l5.803-3.558L0 4.697ZM6.761 8.83l-6.57 4.027A2 2 0 0 0 2 14h12a2 2 0 0 0 1.808-1.144l-6.57-4.027L8 9.586l-1.239-.757Zm3.436-.586L16 11.801V4.697l-5.803 3.546Z"/>
            </svg>
            Send Email
          </DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={resetAndClose}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>

        <div class="px-4 py-3 space-y-3">
          {/* Execution info */}
          <div class="px-2 py-1.5 bg-[#2d2d2d] border border-[#3c3c3c] rounded-[3px] text-[11px] text-[#808080]">
            Execution: <span class="text-[#cccccc]">{props.executionLabel}</span>
          </div>

          {/* SMTP Server */}
          <div>
            <label class="text-[10px] text-[#cccccc] mb-1 block">SMTP Server (Sender)</label>
            <select
              class={selectClass}
              value={smtpId()?.toString() ?? ''}
              onChange={(e) => setSmtpId(e.currentTarget.value ? parseInt(e.currentTarget.value, 10) : undefined)}
            >
              <option value="">Select SMTP server...</option>
              <For each={activeSmtpConfigs()}>
                {(config) => (
                  <option value={config.id.toString()}>
                    {config.name} ({config.senderEmail})
                  </option>
                )}
              </For>
            </select>
          </div>

          {/* Email Subject */}
          <div>
            <label class="text-[10px] text-[#cccccc] mb-1 block">Subject</label>
            <input
              type="text"
              class={selectClass}
              value={subject()}
              onInput={(e) => setSubject(e.currentTarget.value)}
              placeholder="NetNinja Task Results"
            />
          </div>

          {/* To Recipients */}
          <div>
            <label class="text-[10px] text-[#cccccc] mb-1 block">To Recipients</label>
            <div class="space-y-1 max-h-[100px] overflow-y-auto pr-2">
              <Show
                when={!emails.loading && activeEmails().length > 0}
                fallback={<p class="text-[10px] text-[#808080] py-1">{emails.loading ? 'Loading...' : 'No recipients configured'}</p>}
              >
                <For each={activeEmails()}>
                  {(email) => (
                    <label class="flex items-center gap-2 px-2 py-1 rounded hover:bg-[#2d2d2d] cursor-pointer transition-colors">
                      <Checkbox
                        checked={toIds().includes(email.id)}
                        onChange={() => handleToggleTo(email.id)}
                      />
                      <div class="flex-1 min-w-0">
                        <div class="text-[11px] text-[#cccccc] truncate">
                          {email.name || 'Unnamed'}
                        </div>
                        <div class="text-[10px] text-[#808080] truncate">
                          {email.recipient}
                        </div>
                      </div>
                    </label>
                  )}
                </For>
              </Show>
            </div>
          </div>

          {/* CC Recipients */}
          <div>
            <label class="text-[10px] text-[#cccccc] mb-1 block">CC Recipients (optional)</label>
            <div class="space-y-1 max-h-[80px] overflow-y-auto pr-2">
              <Show when={!emails.loading && activeEmails().length > 0}>
                <For each={activeEmails()}>
                  {(email) => (
                    <label class="flex items-center gap-2 px-2 py-1 rounded hover:bg-[#2d2d2d] cursor-pointer transition-colors">
                      <Checkbox
                        checked={ccIds().includes(email.id)}
                        onChange={() => handleToggleCc(email.id)}
                      />
                      <div class="flex-1 min-w-0">
                        <div class="text-[11px] text-[#cccccc] truncate">
                          {email.name || 'Unnamed'}
                        </div>
                        <div class="text-[10px] text-[#808080] truncate">
                          {email.recipient}
                        </div>
                      </div>
                    </label>
                  )}
                </For>
              </Show>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div class="flex justify-end gap-2 border-t border-[#3c3c3c] px-4 py-2.5">
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
            onClick={resetAndClose}
          >
            Cancel
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-1.5"
            onClick={handleSend}
            disabled={sending() || !smtpId() || toIds().length === 0}
          >
            {sending() ? (
              <>
                <svg class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Sending...
              </>
            ) : (
              'Send Email'
            )}
          </button>
        </div>
      </DialogContent>
    </Dialog>
  );
};

// --- Execution Row ---

const ExecutionRow: Component<{ execution: TaskExecutionResponse; onResend?: (executionId: string, label: string) => void }> = (props) => {
  const [expanded, setExpanded] = createSignal(false);

  const executionLabel = () => `${formatDateTime(props.execution.startedAt)} - ${props.execution.status}`;

  return (
    <div class="border border-[#3c3c3c] rounded-[3px] overflow-hidden">
      {/* Execution Header */}
      <button
        type="button"
        onClick={() => setExpanded(!expanded())}
        class={cn(
          'w-full flex items-center justify-between px-3 py-2 text-left transition-colors',
          'hover:bg-[#2d2d2d]/80',
          expanded() && 'bg-[#2d2d2d]'
        )}
      >
        <div class="flex items-center gap-2">
          <StatusIcon status={props.execution.status} />
          <div>
            <div class="flex items-center gap-2">
              <span class="text-[12px] font-medium text-[#cccccc]">
                {formatDateTime(props.execution.startedAt)}
              </span>
              <div class="flex items-center gap-1 text-[11px] text-[#808080]">
                <TriggerIcon trigger={props.execution.triggeredBy} />
                <span>{props.execution.triggeredBy}</span>
              </div>
            </div>
            <div class="text-[11px] text-[#808080] flex items-center gap-2 mt-0.5">
              <Show when={props.execution.resultSummary}>
                <span>
                  {props.execution.resultSummary!.successCount} succeeded,{' '}
                  {props.execution.resultSummary!.failureCount} failed
                </span>
              </Show>
              <Show when={props.execution.durationMs !== null}>
                <span class="text-[#606060]">•</span>
                <span>{formatDuration(props.execution.durationMs)}</span>
              </Show>
            </div>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Show when={props.onResend && props.execution.status !== 'running'}>
            <button
              type="button"
              class="p-1 rounded-[2px] text-[#808080] hover:text-[#cccccc] hover:bg-white/[0.06] transition-colors"
              title="Send email for this execution"
              onClick={(e) => {
                e.stopPropagation();
                props.onResend?.(props.execution.executionId, executionLabel());
              }}
            >
              <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
                <path d="M.05 3.555A2 2 0 0 1 2 2h12a2 2 0 0 1 1.95 1.555L8 8.414.05 3.555ZM0 4.697v7.104l5.803-3.558L0 4.697ZM6.761 8.83l-6.57 4.027A2 2 0 0 0 2 14h12a2 2 0 0 0 1.808-1.144l-6.57-4.027L8 9.586l-1.239-.757Zm3.436-.586L16 11.801V4.697l-5.803 3.546Z"/>
              </svg>
            </button>
          </Show>
          <span
            class={cn(
              'px-2 py-0.5 rounded text-[11px] font-medium',
              props.execution.status === 'completed' && 'bg-[#4ec9b0]/20 text-[#4ec9b0]',
              props.execution.status === 'failed' && 'bg-[#c72e0f]/20 text-[#c72e0f]',
              props.execution.status === 'running' && 'bg-[#dcdcaa]/20 text-[#dcdcaa]'
            )}
          >
            {props.execution.status}
          </span>
          {expanded() ? (
            <svg width="14" height="14" viewBox="0 0 16 16" fill="#808080">
              <path fill-rule="evenodd" d="M1.646 4.646a.5.5 0 0 1 .708 0L8 10.293l5.646-5.647a.5.5 0 0 1 .708.708l-6 6a.5.5 0 0 1-.708 0l-6-6a.5.5 0 0 1 0-.708z"/>
            </svg>
          ) : (
            <svg width="14" height="14" viewBox="0 0 16 16" fill="#808080">
              <path fill-rule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/>
            </svg>
          )}
        </div>
      </button>

      {/* Expanded Details */}
      <Show when={expanded()}>
        <div class="border-t border-[#3c3c3c] bg-[#252525] px-3 py-2">
          {/* Error Message */}
          <Show when={props.execution.errorMessage}>
            <div class="mb-2 px-2 py-1.5 bg-[#c72e0f]/10 border border-[#c72e0f]/30 rounded text-[11px] text-[#c72e0f]">
              {props.execution.errorMessage}
            </div>
          </Show>

          {/* Line Results */}
          <Show when={props.execution.lineResults.length > 0}>
            <div class="space-y-2">
              <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium">
                Line Results
              </div>
              <div class="space-y-1">
                <For each={props.execution.lineResults}>
                  {(result) => (
                    <div
                      class={cn(
                        'flex items-center justify-between px-2 py-1.5 rounded-[2px]',
                        'bg-[#2d2d2d] border border-[#3c3c3c]'
                      )}
                    >
                      <div class="flex items-center gap-2">
                        <StatusIcon status={result.status} />
                        <span class="text-[12px] text-[#cccccc]">{result.lineName}</span>
                        <span class="text-[11px] text-[#808080]">
                          {result.taskType === 'speed_test' ? 'Speed' : 'Quota'}
                        </span>
                      </div>
                      <div class="flex items-center gap-3 text-[11px] text-[#808080]">
                        <Show when={result.durationMs !== null}>
                          <span>{formatDuration(result.durationMs)}</span>
                        </Show>
                        <Show when={result.errorMessage}>
                          <span class="text-[#c72e0f] max-w-[200px] truncate">
                            {result.errorMessage}
                          </span>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </Show>

          <Show when={props.execution.lineResults.length === 0}>
            <div class="text-[11px] text-[#808080] text-center py-2">
              No detailed results available
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

// --- Main Modal ---

export const ExecutionHistoryModal: Component<ExecutionHistoryModalProps> = (props) => {
  const [executions, { refetch }] = createResource(
    () => (props.open && props.task ? props.task.id : null),
    async (taskId) => {
      if (!taskId) return [];
      try {
        return await taskExecutionsApi.getByTaskId(taskId, 20);
      } catch (e) {
        showToast({ title: 'Failed to load execution history', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
        return [];
      }
    }
  );

  // Resend email dialog state
  const [resendOpen, setResendOpen] = createSignal(false);
  const [resendExecId, setResendExecId] = createSignal('');
  const [resendExecLabel, setResendExecLabel] = createSignal('');

  const openResendDialog = (executionId: string, label: string) => {
    setResendExecId(executionId);
    setResendExecLabel(label);
    setResendOpen(true);
  };

  return (
    <>
      <Dialog open={props.open} onOpenChange={props.onOpenChange}>
        <DialogContent class="max-w-[600px] max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              Execution History
              <Show when={props.task}>
                <span class="font-normal text-[#808080] ml-2">
                  for {props.task!.name}
                </span>
              </Show>
            </DialogTitle>
            <button
              type="button"
              class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
              onClick={() => props.onOpenChange(false)}
            >
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
            </button>
          </DialogHeader>

          <div class="px-4 py-3">
            <Show when={executions.loading}>
              <div class="text-center py-8 text-[12px] text-[#808080]">
                Loading execution history...
              </div>
            </Show>

            <Show when={!executions.loading && executions() && executions()!.length > 0}>
              <div class="space-y-2">
                <For each={executions()}>
                  {(execution) => (
                    <ExecutionRow
                      execution={execution}
                      onResend={openResendDialog}
                    />
                  )}
                </For>
              </div>
            </Show>

            <Show when={!executions.loading && executions()?.length === 0}>
              <div class="text-center py-8">
                <svg width="48" height="48" viewBox="0 0 16 16" fill="#3c3c3c" class="mx-auto mb-3">
                  <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
                  <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
                </svg>
                <p class="text-[12px] text-[#808080]">No executions yet</p>
                <p class="text-[11px] text-[#606060] mt-1">
                  Run the task to see execution history here
                </p>
              </div>
            </Show>
          </div>

          <div class="flex items-center justify-between border-t border-[#3c3c3c] px-4 py-2.5">
            <button
              type="button"
              class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
              onClick={() => refetch()}
            >
              Refresh
            </button>
            <button
              type="button"
              class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
              onClick={() => props.onOpenChange(false)}
            >
              Close
            </button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Resend Email Dialog */}
      <ResendEmailDialog
        open={resendOpen()}
        executionId={resendExecId()}
        executionLabel={resendExecLabel()}
        onOpenChange={setResendOpen}
      />
    </>
  );
};
