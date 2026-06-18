import { type Component, createSignal, createResource, Show, For } from 'solid-js';
import { smtpConfigsApi, emailsApi } from '~/api/client';
import { SmtpServersContent } from './SmtpServersContent';
import { EmailRecipientsContent } from './EmailRecipientsContent';

interface EmailSettingsProps {
  addTrigger?: number;
  onAdd?: () => void;
}

type TabId = 'smtp' | 'recipients';

interface TabItem {
  id: TabId;
  label: string;
}

const tabs: TabItem[] = [
  { id: 'smtp', label: 'SMTP Servers' },
  { id: 'recipients', label: 'Recipients' },
];

type EmailStatus = 'ready' | 'no-default' | 'no-recipients' | 'not-configured';

export const EmailSettings: Component<EmailSettingsProps> = (props) => {
  const [activeTab, setActiveTab] = createSignal<TabId>('smtp');
  const [smtpConfigs, { refetch: refetchSmtp }] = createResource(() => smtpConfigsApi.list());
  const [emails, { refetch: refetchEmails }] = createResource(() => emailsApi.list());

  const emailStatus = (): EmailStatus => {
    const configs = smtpConfigs();
    const recipients = emails();
    if (!configs || configs.length === 0) return 'not-configured';
    if (!configs.some(c => c.isDefault)) return 'no-default';
    if (!recipients || !recipients.some(r => r.isActive)) return 'no-recipients';
    return 'ready';
  };

  const statusConfig = () => {
    const s = emailStatus();
    switch (s) {
      case 'ready': return { label: 'Email: Ready', class: 'text-success', dot: 'bg-success' };
      case 'no-default': return { label: 'No default server', class: 'text-warning', dot: 'bg-warning' };
      case 'no-recipients': return { label: 'No active recipients', class: 'text-warning', dot: 'bg-warning' };
      case 'not-configured': return { label: 'Not configured', class: 'text-destructive', dot: 'bg-destructive' };
    }
  };

  // Refetch data when switching tabs back to keep status fresh
  const handleTabChange = (id: TabId) => {
    setActiveTab(id);
    refetchSmtp();
    refetchEmails();
  };

  return (
    <div class="flex flex-1 min-h-0 h-full flex-col bg-background">
      {/* Top tab navigation (mockup .tab-nav style) */}
      <nav class="flex items-center gap-1 px-6 bg-card border-b border-border shrink-0 overflow-x-auto">
        <For each={tabs}>
          {(tab) => (
            <button
              type="button"
              onClick={() => handleTabChange(tab.id)}
              class={`relative px-5 py-3 text-[14px] font-medium whitespace-nowrap border-b-2 transition-colors cursor-pointer ${
                activeTab() === tab.id
                  ? 'text-primary border-primary'
                  : 'text-muted-foreground border-transparent hover:text-foreground'
              }`}
            >
              {tab.label}
            </button>
          )}
        </For>

        <div class="flex-1" />

        {/* Email status pill */}
        <div class="flex items-center gap-2 pr-1">
          <span class={`h-2 w-2 rounded-full shrink-0 ${statusConfig().dot}`} />
          <span class={`text-[12px] font-medium truncate ${statusConfig().class}`}>
            {statusConfig().label}
          </span>
        </div>
      </nav>

      {/* Content area */}
      <div class="flex-1 min-h-0 overflow-hidden bg-background">
        <Show when={activeTab() === 'smtp'}>
          <SmtpServersContent addTrigger={props.addTrigger} onAdd={props.onAdd} />
        </Show>
        <Show when={activeTab() === 'recipients'}>
          <EmailRecipientsContent addTrigger={props.addTrigger} onAdd={props.onAdd} />
        </Show>
      </div>
    </div>
  );
};
