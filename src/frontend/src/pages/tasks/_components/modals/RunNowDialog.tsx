import { type Component, type Accessor, createSignal, createResource, Show, For } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '~/components/ui/dialog';
import { Checkbox } from '~/components/ui/checkbox';
import type { Task, RuntimeNotificationConfig } from '~/types';
import { taskNotificationApi, emailsApi, smtpConfigsApi } from '~/api/client';
import { cn } from '~/lib/utils';

interface RunNowDialogProps {
  open: Accessor<boolean>;
  onOpenChange: (open: boolean) => void;
  task: Accessor<Task | null>;
  onExecute: (task: Task, notificationOverride?: RuntimeNotificationConfig) => Promise<void>;
}

const selectClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] cursor-pointer";

type NotificationMode = 'saved' | 'override';

export const RunNowDialog: Component<RunNowDialogProps> = (props) => {
  const [mode, setMode] = createSignal<NotificationMode>('saved');
  const [executing, setExecuting] = createSignal(false);

  // Load notification config for the task
  const [notificationConfig] = createResource(
    () => props.task()?.id,
    (taskId) => taskNotificationApi.get(taskId)
  );

  // Load emails and SMTP configs for override form
  const [emails] = createResource(() => emailsApi.list());
  const [smtpConfigs] = createResource(() => smtpConfigsApi.list());

  // Override form state
  const [overrideEnabled, setOverrideEnabled] = createSignal(true);
  const [overrideSmtpId, setOverrideSmtpId] = createSignal<number | undefined>();
  const [overrideSubject, setOverrideSubject] = createSignal<string>('');
  const [overrideToIds, setOverrideToIds] = createSignal<number[]>([]);
  const [overrideCcIds, setOverrideCcIds] = createSignal<number[]>([]);

  const activeEmails = () => emails()?.filter(e => e.isActive) ?? [];

  const handleExecute = async () => {
    const task = props.task();
    if (!task) return;

    setExecuting(true);
    try {
      let override: RuntimeNotificationConfig | undefined;

      if (mode() === 'override') {
        override = {
          isEnabled: overrideEnabled(),
          smtpConfigId: overrideSmtpId(),
          emailSubject: overrideSubject() || undefined,
          toRecipientIds: overrideToIds(),
          ccRecipientIds: overrideCcIds(),
        };
      }

      // Start execution (don't await - let it run in background)
      props.onExecute(task, override).catch((error) => {
        console.error('Failed to execute task:', error);
      });

      // Close dialog immediately after starting the task
      props.onOpenChange(false);
      resetOverride();
    } catch (error) {
      console.error('Failed to execute task:', error);
    } finally {
      setExecuting(false);
    }
  };

  const resetOverride = () => {
    setMode('saved');
    setOverrideEnabled(true);
    setOverrideSmtpId(undefined);
    setOverrideSubject('');
    setOverrideToIds([]);
    setOverrideCcIds([]);
  };

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      resetOverride();
    }
    props.onOpenChange(open);
  };

  const handleToggleToRecipient = (emailId: number) => {
    const current = overrideToIds();
    const newIds = current.includes(emailId)
      ? current.filter(id => id !== emailId)
      : [...current, emailId];
    setOverrideToIds(newIds);
  };

  const handleToggleCcRecipient = (emailId: number) => {
    const current = overrideCcIds();
    const newIds = current.includes(emailId)
      ? current.filter(id => id !== emailId)
      : [...current, emailId];
    setOverrideCcIds(newIds);
  };

  const savedConfigSummary = () => {
    const config = notificationConfig();
    if (!config?.isEnabled) return 'Notifications disabled';

    const toCount = config.toRecipientIds.length;
    const smtpName = smtpConfigs()?.find(s => s.id === config.smtpConfigId)?.name || 'Unknown SMTP';

    return `Enabled: To ${toCount} recipient${toCount !== 1 ? 's' : ''} via "${smtpName}"`;
  };

  return (
    <Dialog open={props.open()} onOpenChange={handleOpenChange}>
      <DialogContent class="max-w-[520px]">
        <DialogHeader>
          <DialogTitle>Run Task: {props.task()?.name}</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <DialogDescription class="px-4 py-2 text-[11px] text-[#808080]">
          Execute this task immediately with optional notification override.
        </DialogDescription>

        <div class="px-4 py-3 space-y-4">
          <div>
            <h4 class="text-[11px] text-[#cccccc] mb-2">Notification Settings</h4>

            <div class="space-y-2">
              {/* Use saved settings */}
              <button
                type="button"
                onClick={() => setMode('saved')}
                class={cn(
                  "flex items-start gap-3 px-3 py-2 rounded-[3px] border transition-colors cursor-pointer w-full text-left",
                  mode() === 'saved' ? 'border-[#007acc] bg-[#007acc]/10' : 'border-[#3c3c3c] hover:border-[#007acc]/50'
                )}
              >
                <div class={cn(
                  "w-4 h-4 rounded-full border-2 mt-0.5 shrink-0 flex items-center justify-center",
                  mode() === 'saved' ? 'border-[#007acc] bg-[#007acc]' : 'border-[#3c3c3c]'
                )}>
                  {mode() === 'saved' && (
                    <div class="w-2 h-2 rounded-full bg-white"></div>
                  )}
                </div>
                <div class="flex-1">
                  <div class="text-[11px] text-[#cccccc]">
                    Use saved notification settings
                  </div>
                  <p class="text-[10px] text-[#808080] mt-1">
                    {savedConfigSummary()}
                  </p>
                </div>
              </button>

              {/* Override */}
              <button
                type="button"
                onClick={() => setMode('override')}
                class={cn(
                  "flex items-start gap-3 px-3 py-2 rounded-[3px] border transition-colors cursor-pointer w-full text-left",
                  mode() === 'override' ? 'border-[#007acc] bg-[#007acc]/10' : 'border-[#3c3c3c] hover:border-[#007acc]/50'
                )}
              >
                <div class={cn(
                  "w-4 h-4 rounded-full border-2 mt-0.5 shrink-0 flex items-center justify-center",
                  mode() === 'override' ? 'border-[#007acc] bg-[#007acc]' : 'border-[#3c3c3c]'
                )}>
                  {mode() === 'override' && (
                    <div class="w-2 h-2 rounded-full bg-white"></div>
                  )}
                </div>
                <div class="flex-1">
                  <div class="text-[11px] text-[#cccccc]">
                    Override for this run
                  </div>
                  <p class="text-[10px] text-[#808080] mt-1">
                    Configure custom notification settings for this execution only
                  </p>
                </div>
              </button>
            </div>
          </div>

          {/* Override form */}
          <Show when={mode() === 'override'}>
            <div class="space-y-3 px-3 py-3 bg-[#252525] border border-[#3c3c3c] rounded-[3px]">
              <h5 class="text-[10px] text-[#cccccc] flex items-center gap-2">
                <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M.05 3.555A2 2 0 0 1 2 2h12a2 2 0 0 1 1.95 1.555L8 8.414.05 3.555ZM0 4.697v7.104l5.803-3.558L0 4.697ZM6.761 8.83l-6.57 4.027A2 2 0 0 0 2 14h12a2 2 0 0 0 1.808-1.144l-6.57-4.027L8 9.586l-1.239-.757Zm3.436-.586L16 11.801V4.697l-5.803 3.546Z"/>
                </svg>
                Override Configuration
              </h5>

              {/* SMTP Server */}
              <div>
                <label class="text-[10px] text-[#cccccc] mb-1 block">
                  SMTP Server
                </label>
                <select
                  class={selectClass}
                  value={overrideSmtpId()?.toString() ?? ''}
                  onChange={(e) => setOverrideSmtpId(e.currentTarget.value ? parseInt(e.currentTarget.value, 10) : undefined)}
                >
                  <option value="">Select SMTP server...</option>
                  <For each={smtpConfigs()?.filter(c => c.isActive) ?? []}>
                    {(config) => (
                      <option value={config.id.toString()}>{config.name}</option>
                    )}
                  </For>
                </select>
              </div>

              {/* Email Subject */}
              <div>
                <label class="text-[10px] text-[#cccccc] mb-1 block">
                  Email Subject (optional)
                </label>
                <input
                  type="text"
                  class={selectClass}
                  value={overrideSubject()}
                  onInput={(e) => setOverrideSubject(e.currentTarget.value)}
                  placeholder="NetNinja Task Results"
                />
              </div>

              {/* To Recipients */}
              <div>
                <label class="text-[10px] text-[#cccccc] mb-1 block">
                  To Recipients
                </label>
                <div class="space-y-1 max-h-[100px] overflow-y-auto pr-2">
                  <Show
                    when={!emails.loading && activeEmails().length > 0}
                    fallback={<p class="text-[10px] text-[#808080] py-1">{emails.loading ? 'Loading...' : 'No recipients'}</p>}
                  >
                    <For each={activeEmails()}>
                      {(email) => (
                        <label class="flex items-center gap-2 px-2 py-1 rounded hover:bg-[#2d2d2d] cursor-pointer transition-colors">
                          <Checkbox
                            checked={overrideToIds().includes(email.id)}
                            onChange={() => handleToggleToRecipient(email.id)}
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
                <label class="text-[10px] text-[#cccccc] mb-1 block">
                  CC Recipients (optional)
                </label>
                <div class="space-y-1 max-h-[80px] overflow-y-auto pr-2">
                  <Show when={!emails.loading && activeEmails().length > 0}>
                    <For each={activeEmails()}>
                      {(email) => (
                        <label class="flex items-center gap-2 px-2 py-1 rounded hover:bg-[#2d2d2d] cursor-pointer transition-colors">
                          <Checkbox
                            checked={overrideCcIds().includes(email.id)}
                            onChange={() => handleToggleCcRecipient(email.id)}
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
          </Show>
        </div>

        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={() => handleOpenChange(false)}
            disabled={executing()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-1.5"
            onClick={handleExecute}
            disabled={executing()}
          >
            {executing() ? (
              <>
                <svg class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Running...
              </>
            ) : (
              'Run Task'
            )}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
