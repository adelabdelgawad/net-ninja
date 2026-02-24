import { type Component, Show, createResource, For } from 'solid-js';
import { cn } from '~/lib/utils';
import { emailsApi, smtpConfigsApi } from '~/api/client';
import { Checkbox } from '~/components/ui/checkbox';

const inputClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const selectClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] cursor-pointer";
const labelClass = "text-[11px] text-[#808080] mb-1 block";

export interface NotificationConfig {
  isEnabled: boolean;
  smtpConfigId?: number;
  emailSubject?: string;
  toRecipientIds: number[];
  ccRecipientIds: number[];
}

export interface StepNotificationsProps {
  value: NotificationConfig;
  onChange: (value: NotificationConfig) => void;
}

export const StepNotifications: Component<StepNotificationsProps> = (props) => {
  const [emails] = createResource(() => emailsApi.list());
  const [smtpConfigs] = createResource(() => smtpConfigsApi.list());

  // Filter active emails only
  const activeEmails = () => emails()?.filter(e => e.isActive) ?? [];

  const handleToggleEnabled = () => {
    props.onChange({
      ...props.value,
      isEnabled: !props.value.isEnabled,
    });
  };

  const handleSmtpChange = (e: Event) => {
    const value = (e.currentTarget as HTMLSelectElement).value;
    props.onChange({
      ...props.value,
      smtpConfigId: value ? parseInt(value, 10) : undefined,
    });
  };

  const handleSubjectChange = (e: Event) => {
    const value = (e.currentTarget as HTMLInputElement).value;
    props.onChange({
      ...props.value,
      emailSubject: value || undefined,
    });
  };

  const handleToRecipientToggle = (emailId: number) => {
    const current = props.value.toRecipientIds;
    const newIds = current.includes(emailId)
      ? current.filter(id => id !== emailId)
      : [...current, emailId];
    props.onChange({
      ...props.value,
      toRecipientIds: newIds,
    });
  };

  const handleCcRecipientToggle = (emailId: number) => {
    const current = props.value.ccRecipientIds;
    const newIds = current.includes(emailId)
      ? current.filter(id => id !== emailId)
      : [...current, emailId];
    props.onChange({
      ...props.value,
      ccRecipientIds: newIds,
    });
  };

  return (
    <div class="space-y-4">
      <div>
        <h3 class="text-[13px] font-medium text-[#cccccc] mb-1">Email Notifications</h3>
        <p class="text-[11px] text-[#808080]">
          Configure email notifications to be sent after task execution.
        </p>
      </div>

      {/* Enable/Disable Toggle */}
      <button
        type="button"
        onClick={handleToggleEnabled}
        class={cn(
          'group relative flex items-start gap-3 px-4 py-3 rounded-[3px] border-2 transition-all cursor-pointer text-left w-full',
          'bg-[#2d2d2d] hover:bg-[#2d2d2d]/80',
          props.value.isEnabled
            ? 'border-[#007acc] ring-2 ring-[#007acc]/30'
            : 'border-[#3c3c3c] hover:border-[#007acc]/50'
        )}
      >
        <div
          class={cn(
            'w-10 h-10 rounded-[3px] flex items-center justify-center transition-colors shrink-0',
            props.value.isEnabled
              ? 'bg-[#007acc] text-white'
              : 'bg-[#3c3c3c] text-[#808080] group-hover:bg-[#007acc]/20 group-hover:text-[#007acc]'
          )}
        >
          {props.value.isEnabled ? (
            <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm.93-9.412-1 4.705c-.07.34.029.533.304.533.194 0 .487-.07.686-.246l-.088.416c-.287.346-.92.598-1.465.598-.703 0-1.002-.422-.808-1.319l.738-3.468c.064-.293.006-.399-.287-.47l-.451-.081.082-.381 2.29-.287zM8 5.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2z"/>
            </svg>
          ) : (
            <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"/>
              <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
            </svg>
          )}
        </div>

        <div class="flex-1">
          <h4 class="text-[12px] font-medium text-[#cccccc] mb-0.5">
            {props.value.isEnabled ? 'Notifications Enabled' : 'Notifications Disabled'}
          </h4>
          <p class="text-[11px] text-[#808080] leading-relaxed">
            {props.value.isEnabled
              ? 'Email notifications will be sent after each task execution'
              : 'Click to enable email notifications for this task'}
          </p>
        </div>

        {props.value.isEnabled && (
          <div class="absolute top-3 right-3">
            <div class="w-5 h-5 rounded-full bg-[#007acc] flex items-center justify-center">
              <svg class="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
              </svg>
            </div>
          </div>
        )}
      </button>

      {/* Configuration (shown when enabled) */}
      <Show when={props.value.isEnabled}>
        <div class="space-y-4 px-4 py-3 bg-[#252525] border border-[#3c3c3c] rounded-[3px]">
          <h4 class="text-[11px] font-medium text-[#cccccc] flex items-center gap-2">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M.05 3.555A2 2 0 0 1 2 2h12a2 2 0 0 1 1.95 1.555L8 8.414.05 3.555ZM0 4.697v7.104l5.803-3.558L0 4.697ZM6.761 8.83l-6.57 4.027A2 2 0 0 0 2 14h12a2 2 0 0 0 1.808-1.144l-6.57-4.027L8 9.586l-1.239-.757Zm3.436-.586L16 11.801V4.697l-5.803 3.546Z"/>
            </svg>
            Notification Configuration
          </h4>

          {/* SMTP Server Selection */}
          <div>
            <label class={labelClass}>SMTP Server</label>
            <select
              class={selectClass}
              value={props.value.smtpConfigId?.toString() ?? ''}
              onChange={handleSmtpChange}
            >
              <option value="">Select SMTP server...</option>
              <For each={smtpConfigs()?.filter(c => c.isActive) ?? []}>
                {(config) => (
                  <option value={config.id.toString()}>{config.name}</option>
                )}
              </For>
            </select>
            <Show when={!smtpConfigs.loading && smtpConfigs()?.length === 0}>
              <p class="text-[10px] text-[#c72e0f] mt-1">
                No SMTP servers configured. Please add one in Email Settings.
              </p>
            </Show>
          </div>

          {/* Email Subject */}
          <div>
            <label class={labelClass}>Email Subject (optional)</label>
            <input
              type="text"
              class={inputClass}
              value={props.value.emailSubject || ''}
              onInput={handleSubjectChange}
              placeholder="NetNinja Task Results"
            />
          </div>

          {/* To Recipients */}
          <div>
            <label class={labelClass}>To Recipients</label>
            <div class="space-y-1 max-h-[120px] overflow-y-auto pr-2">
              <Show
                when={!emails.loading && activeEmails().length > 0}
                fallback={
                  <p class="text-[11px] text-[#808080] py-1">
                    {emails.loading ? 'Loading recipients...' : 'No active email recipients found.'}
                  </p>
                }
              >
                <For each={activeEmails()}>
                  {(email) => (
                    <label class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-[#2d2d2d] cursor-pointer transition-colors">
                      <Checkbox
                        checked={props.value.toRecipientIds.includes(email.id)}
                        onChange={() => handleToRecipientToggle(email.id)}
                      />
                      <div class="flex-1 min-w-0">
                        <div class="text-[11px] font-medium text-[#cccccc] truncate">
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
            <label class={labelClass}>CC Recipients (optional)</label>
            <div class="space-y-1 max-h-[100px] overflow-y-auto pr-2">
              <Show when={!emails.loading && activeEmails().length > 0}>
                <For each={activeEmails()}>
                  {(email) => (
                    <label class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-[#2d2d2d] cursor-pointer transition-colors">
                      <Checkbox
                        checked={props.value.ccRecipientIds.includes(email.id)}
                        onChange={() => handleCcRecipientToggle(email.id)}
                      />
                      <div class="flex-1 min-w-0">
                        <div class="text-[11px] font-medium text-[#cccccc] truncate">
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
  );
};
