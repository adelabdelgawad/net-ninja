import { type Component, createSignal, createResource, createEffect, on } from 'solid-js';
import { emailsApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';
import type { Email, EmailCreate, EmailUpdate } from '~/types';
import {
  EmailRecipientList,
  EmailRecipientFormDialog,
  EmailRecipientDeleteDialog,
} from './email';

interface EmailRecipientsContentProps {
  addTrigger?: number;
  onAdd?: () => void;
}

export const EmailRecipientsContent: Component<EmailRecipientsContentProps> = (props) => {
  const [emails, { refetch }] = createResource(() => emailsApi.list());
  const [isAddDialogOpen, setIsAddDialogOpen] = createSignal(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = createSignal(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = createSignal(false);
  const [selectedEmail, setSelectedEmail] = createSignal<Email | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);

  // Form state
  const [formData, setFormData] = createSignal<Partial<EmailCreate>>({
    name: '',
    recipient: '',
    isActive: true,
  });

  const resetForm = () => {
    setFormData({
      name: '',
      recipient: '',
      isActive: true,
    });
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

  const openEditDialog = (email: Email) => {
    setSelectedEmail(email);
    setFormData({
      name: email.name || '',
      recipient: email.recipient,
      isActive: email.isActive,
    });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (email: Email) => {
    setSelectedEmail(email);
    setIsDeleteDialogOpen(true);
  };

  const handleCreate = async () => {
    const data = formData();
    if (!data.recipient) {
      return;
    }
    setIsLoading(true);
    try {
      await emailsApi.create(data as EmailCreate);
      setIsAddDialogOpen(false);
      refetch();
      showToast({ title: 'Email recipient created', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to create email recipient', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdate = async () => {
    const email = selectedEmail();
    if (!email) return;

    const data = formData();
    const updateData: EmailUpdate = {
      name: data.name,
      recipient: data.recipient,
      isActive: data.isActive,
    };

    setIsLoading(true);
    try {
      await emailsApi.update(email.id, updateData);
      setIsEditDialogOpen(false);
      refetch();
      showToast({ title: 'Email recipient updated', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to update email recipient', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async () => {
    const email = selectedEmail();
    if (!email) return;

    setIsLoading(true);
    try {
      await emailsApi.delete(email.id);
      setIsDeleteDialogOpen(false);
      refetch();
      showToast({ title: 'Email recipient deleted', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to delete email recipient', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setIsLoading(false);
    }
  };

  const handleToggleActive = async (email: Email) => {
    try {
      await emailsApi.update(email.id, { isActive: !email.isActive });
      refetch();
      showToast({ title: `Recipient ${!email.isActive ? 'activated' : 'deactivated'}`, variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to toggle recipient status', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  const updateFormField = <K extends keyof EmailCreate>(field: K, value: EmailCreate[K]) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  return (
    <div class="flex flex-col h-full">
      {/* Content header - App Center style */}
      <div class="px-6 pt-6 pb-4 flex-shrink-0">
        <h1 class="text-[20px] font-semibold text-[#eeeeee] mb-1">Email Recipients</h1>
        <p class="text-[13px] text-[#999999] leading-relaxed">
          Manage email addresses that receive notifications and daily reports.
        </p>
      </div>

      {/* Actions bar */}
      <div class="flex items-center gap-3 mb-4 px-6 flex-shrink-0">
        <span class="text-[13px] font-medium text-[#cccccc]">
          Recipients ({emails()?.length ?? 0})
        </span>
        <div class="flex-1" />
        <button
          type="button"
          class="h-[30px] px-4 text-[12px] font-medium rounded-[8px] bg-[#3584e4] text-white hover:bg-[#3987e5] transition-colors"
          onClick={openAddDialog}
        >
          + Add Recipient
        </button>
      </div>

      {/* Email list */}
      <div class="flex-1 overflow-hidden px-6">
        <EmailRecipientList
        emails={emails}
        onAddRecipient={openAddDialog}
        onEdit={openEditDialog}
        onToggleActive={handleToggleActive}
        onDelete={openDeleteDialog}
      />
      </div>

      {/* Dialogs */}
      <EmailRecipientFormDialog
        open={isAddDialogOpen}
        onOpenChange={setIsAddDialogOpen}
        mode="add"
        formData={formData}
        updateFormField={updateFormField}
        onSubmit={handleCreate}
        isLoading={isLoading}
      />
      <EmailRecipientFormDialog
        open={isEditDialogOpen}
        onOpenChange={setIsEditDialogOpen}
        mode="edit"
        formData={formData}
        updateFormField={updateFormField}
        onSubmit={handleUpdate}
        isLoading={isLoading}
      />
      <EmailRecipientDeleteDialog
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        email={selectedEmail}
        onDelete={handleDelete}
        isLoading={isLoading}
      />
    </div>
  );
};
