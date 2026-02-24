import { type Component, createSignal, createResource, Show, For } from 'solid-js';
import { smtpConfigsApi, emailsApi } from '~/api/client';
import { SmtpServersContent } from './SmtpServersContent';
import { EmailRecipientsContent } from './EmailRecipientsContent';

interface EmailSettingsProps {
  addTrigger?: number;
  onAdd?: () => void;
}

type TabId = 'smtp' | 'recipients';

interface NavItem {
  id: TabId;
  label: string;
  iconPath: string;
}

const navItems: NavItem[] = [
  {
    id: 'smtp',
    label: 'SMTP Servers',
    iconPath: 'M2 4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4zm2-1a1 1 0 0 0-1 1v.217l5 3.125 5-3.125V4a1 1 0 0 0-1-1H4zm9 2.441-4.724 2.953a.5.5 0 0 1-.552 0L3 5.441V12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V5.441z',
  },
  {
    id: 'recipients',
    label: 'Recipients',
    iconPath: 'M7 14s-1 0-1-1 1-4 5-4 5 3 5 4-1 1-1 1H7zm4-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM5.216 14A2.238 2.238 0 0 1 5 13c0-1.355.68-2.75 1.936-3.72A6.325 6.325 0 0 0 5 9c-4 0-5 3-5 4s1 1 1 1h4.216zM4.5 8a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z',
  },
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
      case 'ready': return { label: 'Email: Ready', color: '#26a269', dot: '#26a269' };
      case 'no-default': return { label: 'No default server', color: '#e5a50a', dot: '#e5a50a' };
      case 'no-recipients': return { label: 'No active recipients', color: '#e5a50a', dot: '#e5a50a' };
      case 'not-configured': return { label: 'Not configured', color: '#c72e0f', dot: '#c72e0f' };
    }
  };

  // Refetch data when switching tabs back to keep status fresh
  const handleTabChange = (id: TabId) => {
    setActiveTab(id);
    refetchSmtp();
    refetchEmails();
  };

  return (
    <div class="flex flex-1 min-h-0 h-full">
      {/* Sidebar */}
      <div class="w-[180px] shrink-0 bg-[#1e1e1e] border-r border-[#2a2a2a] flex flex-col">
        {/* Sidebar header */}
        <div class="px-4 pt-4 pb-3">
          <h2 class="text-[13px] font-semibold text-[#cccccc]">Email</h2>
        </div>

        {/* Nav items */}
        <nav class="flex-1 px-2 space-y-0.5">
          <For each={navItems}>
            {(item) => (
              <button
                type="button"
                class={`w-full flex items-center gap-2.5 px-3 py-[7px] rounded-[8px] text-[12px] font-medium transition-colors cursor-pointer ${
                  activeTab() === item.id
                    ? 'bg-[#2d2d2d] text-[#ffffff]'
                    : 'text-[#999999] hover:bg-[#252526] hover:text-[#cccccc]'
                }`}
                onClick={() => handleTabChange(item.id)}
              >
                <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" class="shrink-0 opacity-80">
                  <path d={item.iconPath} />
                </svg>
                {item.label}
              </button>
            )}
          </For>
        </nav>

        {/* Email status indicator */}
        <div class="px-3 py-3 border-t border-[#2a2a2a]">
          <div class="flex items-center gap-2">
            <div
              class="h-2 w-2 rounded-full shrink-0"
              style={{ "background-color": statusConfig().dot }}
            />
            <span
              class="text-[11px] font-medium truncate"
              style={{ color: statusConfig().color }}
            >
              {statusConfig().label}
            </span>
          </div>
        </div>
      </div>

      {/* Content area */}
      <div class="flex-1 min-h-0 overflow-hidden bg-[#252526]">
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
