import { type Component, createSignal, createResource, createEffect, on, Show } from 'solid-js';
import { smtpConfigsApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';
import type { SmtpConfig, SmtpConfigCreate, SmtpConfigUpdate } from '~/types';
import { applyVendorDefaults } from '~/utils/smtp-vendors';
import {
  SmtpFormDialog,
  SmtpDeleteDialog,
  SmtpTestDialog,
  SmtpServerList,
  type TestStatus,
} from './email';

interface SmtpServersContentProps {
  addTrigger?: number;
  onAdd?: () => void;
}

export const SmtpServersContent: Component<SmtpServersContentProps> = (props) => {
  const [configs, { refetch }] = createResource(() => smtpConfigsApi.list());
  const [isAddDialogOpen, setIsAddDialogOpen] = createSignal(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = createSignal(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = createSignal(false);
  const [isTestDialogOpen, setIsTestDialogOpen] = createSignal(false);
  const [selectedConfig, setSelectedConfig] = createSignal<SmtpConfig | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);
  const [testStatus, setTestStatus] = createSignal<TestStatus>('idle');
  const [testMessage, setTestMessage] = createSignal('');
  const [testRecipient, setTestRecipient] = createSignal('');

  // Form state
  const [formData, setFormData] = createSignal<Partial<SmtpConfigCreate>>({
    name: '',
    host: '',
    port: 587,
    vendor: 'gmail',
    username: '',
    password: '',
    senderEmail: '',
    senderName: 'NetBOT',
    useTls: true,
  });

  const resetForm = () => {
    const defaults = applyVendorDefaults('gmail', {
      name: '',
      username: '',
      password: '',
      senderEmail: '',
      senderName: 'NetBOT',
    });
    setFormData(defaults);
  };

  const openAddDialog = () => {
    resetForm();
    setIsAddDialogOpen(true);
  };

  // React to addTrigger changes from parent (PageHeader button)
  createEffect(on(
    () => props.addTrigger,
    (trigger) => {
      if (trigger && trigger > 0) {
        openAddDialog();
      }
    },
    { defer: true }
  ));

  const openEditDialog = (config: SmtpConfig) => {
    setSelectedConfig(config);
    setFormData({
      name: config.name,
      host: config.host,
      port: config.port,
      vendor: config.vendor,
      username: config.username,
      password: '',
      senderEmail: config.senderEmail,
      senderName: config.senderName,
      useTls: config.useTls,
    });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (config: SmtpConfig) => {
    setSelectedConfig(config);
    setIsDeleteDialogOpen(true);
  };

  const openTestDialog = (config: SmtpConfig) => {
    setSelectedConfig(config);
    setTestRecipient('');
    setTestStatus('idle');
    setTestMessage('');
    setIsTestDialogOpen(true);
  };

  const handleCreate = async () => {
    const data = formData();

    if (data.vendor === 'exchange' && !data.host) {
      showToast({
        title: 'Host required',
        description: 'Exchange requires explicit host configuration',
        variant: 'error',
      });
      return;
    }

    // Required fields
    if (!data.name || !data.username || !data.password || !data.senderEmail) {
      return;
    }

    // Host validation (Exchange already checked above, Gmail/Outlook365 have host auto-filled)
    if (!data.host) {
      showToast({
        title: 'Host required',
        description: 'SMTP host is required',
        variant: 'error',
      });
      return;
    }
    setIsLoading(true);
    try {
      await smtpConfigsApi.create(data as SmtpConfigCreate);
      setIsAddDialogOpen(false);
      refetch();
      showToast({ title: 'SMTP server created', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to create SMTP server', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdate = async () => {
    const config = selectedConfig();
    if (!config) return;

    const data = formData();
    const updateData: SmtpConfigUpdate = {
      name: data.name,
      host: data.host,
      port: data.port,
      vendor: data.vendor,
      username: data.username,
      senderEmail: data.senderEmail,
      senderName: data.senderName,
      useTls: data.useTls,
    };

    // Only include password if it was changed
    if (data.password) {
      updateData.password = data.password;
    }

    setIsLoading(true);
    try {
      await smtpConfigsApi.update(config.id, updateData);
      setIsEditDialogOpen(false);
      refetch();
      showToast({ title: 'SMTP server updated', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to update SMTP server', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async () => {
    const config = selectedConfig();
    if (!config) return;

    setIsLoading(true);
    try {
      await smtpConfigsApi.delete(config.id);
      setIsDeleteDialogOpen(false);
      refetch();
      showToast({ title: 'SMTP server deleted', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to delete SMTP server', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setIsLoading(false);
    }
  };

  const handleSetDefault = async (config: SmtpConfig) => {
    try {
      await smtpConfigsApi.setDefault(config.id);
      refetch();
      showToast({ title: 'Default SMTP server updated', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to set default SMTP server', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  const handleTest = async () => {
    const config = selectedConfig();
    if (!config || !testRecipient()) return;

    setTestStatus('testing');
    try {
      const result = await smtpConfigsApi.testExisting(config.id, testRecipient());
      setTestStatus(result.success ? 'success' : 'error');
      setTestMessage(result.message);
      showToast({ title: result.success ? 'Test email sent' : 'Test failed', description: result.message, variant: result.success ? 'success' : 'error', duration: result.success ? 3000 : 5000 });
    } catch (e) {
      setTestStatus('error');
      setTestMessage(`Failed to test: ${e}`);
      showToast({ title: 'Test failed', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  const updateFormField = <K extends keyof SmtpConfigCreate>(field: K, value: SmtpConfigCreate[K]) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  return (
    <div class="flex flex-col h-full">
      {/* Content header - App Center style */}
      <div class="px-6 pt-6 pb-4 flex-shrink-0">
        <h1 class="text-[20px] font-semibold text-[#eeeeee] mb-1">SMTP Servers</h1>
        <p class="text-[13px] text-[#999999] leading-relaxed">
          Configure outgoing mail servers for sending notifications and reports.
        </p>
      </div>

      {/* Actions bar */}
      <div class="flex items-center gap-3 mb-4 px-6 flex-shrink-0">
        <span class="text-[13px] font-medium text-[#cccccc]">
          Configured servers ({configs()?.length ?? 0})
        </span>
        <div class="flex-1" />
        <button
          type="button"
          class="h-[30px] px-4 text-[12px] font-medium rounded-[8px] bg-[#3584e4] text-white hover:bg-[#3987e5] transition-colors"
          onClick={openAddDialog}
        >
          + Add Server
        </button>
      </div>

      {/* No default warning */}
      <Show when={configs() && configs()!.length > 0 && !configs()!.some(c => c.isDefault)}>
        <div class="mx-6 mb-3 flex items-center gap-2.5 px-3.5 py-2.5 rounded-[8px] bg-[#e5a50a]/10 border border-[#e5a50a]/20 flex-shrink-0">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="#e5a50a" class="shrink-0">
            <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.713c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/>
          </svg>
          <span class="text-[12px] text-[#e5a50a]">
            No default server selected. Set a favourite to enable email notifications.
          </span>
        </div>
      </Show>

      {/* Server list */}
      <div class="flex-1 overflow-hidden px-6">
        <SmtpServerList
        configs={configs}
        onAddServer={openAddDialog}
        onTest={openTestDialog}
        onEdit={openEditDialog}
        onSetDefault={handleSetDefault}
        onDelete={openDeleteDialog}
      />
      </div>

      {/* Dialogs */}
      <SmtpFormDialog
        open={isAddDialogOpen}
        onOpenChange={setIsAddDialogOpen}
        mode="add"
        formData={formData}
        updateFormField={updateFormField}
        onSubmit={handleCreate}
        isLoading={isLoading}
      />
      <SmtpFormDialog
        open={isEditDialogOpen}
        onOpenChange={setIsEditDialogOpen}
        mode="edit"
        formData={formData}
        updateFormField={updateFormField}
        onSubmit={handleUpdate}
        isLoading={isLoading}
      />
      <SmtpDeleteDialog
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        config={selectedConfig}
        onDelete={handleDelete}
        isLoading={isLoading}
      />
      <SmtpTestDialog
        open={isTestDialogOpen}
        onOpenChange={setIsTestDialogOpen}
        config={selectedConfig}
        testRecipient={testRecipient}
        setTestRecipient={setTestRecipient}
        testStatus={testStatus}
        testMessage={testMessage}
        onTest={handleTest}
      />
    </div>
  );
};
