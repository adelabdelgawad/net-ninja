import { type Component, createResource, For, Show, createEffect, createSignal } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '~/components/ui/dialog';
import { Checkbox } from '~/components/ui/checkbox';
import { TimeInput } from '~/components/ui/time-input';
import { cn } from '~/lib/utils';
import type { Task, CreateTaskRequest, EmailCreate } from '~/types';
import { linesApi, tasksApi, emailsApi, smtpConfigsApi, taskNotificationApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';

export interface EditTaskModalProps {
  open: boolean;
  task: Task | null;
  onOpenChange: (open: boolean) => void;
  onSave: () => void | Promise<unknown>;
}

const DAYS_OF_WEEK = [
  { value: 1, label: 'Mon' },
  { value: 2, label: 'Tue' },
  { value: 3, label: 'Wed' },
  { value: 4, label: 'Thu' },
  { value: 5, label: 'Fri' },
  { value: 6, label: 'Sat' },
  { value: 0, label: 'Sun' },
];

const inputClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const primaryInputClass = "h-[34px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const selectClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] cursor-pointer";
const labelClass = "text-[11px] text-[#808080] mb-1 block";
const sectionHeaderClass = "text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-2";

interface EmailRecipient {
  emailId: number;
  type: 'to' | 'cc';
}

export const EditTaskModal: Component<EditTaskModalProps> = (props) => {
  const [lines] = createResource(async () => {
    try {
      return await linesApi.list();
    } catch (e) {
      showToast({ title: 'Failed to load lines', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      return [];
    }
  });

  const [emails, { refetch: refetchEmails }] = createResource(async () => {
    try {
      return await emailsApi.list();
    } catch (e) {
      showToast({ title: 'Failed to load email recipients', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      return [];
    }
  });

  const [smtpConfigs] = createResource(async () => {
    try {
      return await smtpConfigsApi.list();
    } catch (e) {
      showToast({ title: 'Failed to load SMTP configs', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      return [];
    }
  });

  // Form state
  const [taskName, setTaskName] = createSignal('');
  const [selectedLineIds, setSelectedLineIds] = createSignal<number[]>([]);
  const [taskTypes, setTaskTypes] = createSignal<('speed_test' | 'quota_check')[]>([]);
  const [runMode, setRunMode] = createSignal<'one_time' | 'scheduled'>('one_time');
  const [selectedDays, setSelectedDays] = createSignal<number[]>([]);
  const [times, setTimes] = createSignal<string[]>(['09:00']);
  const [saving, setSaving] = createSignal(false);
  const [nameError, setNameError] = createSignal<string | null>(null);

  // Notification state
  const [smtpConfigId, setSmtpConfigId] = createSignal<number | undefined>(undefined);
  const [emailSubject, setEmailSubject] = createSignal('');
  const [recipients, setRecipients] = createSignal<EmailRecipient[]>([]);

  // Inline recipient creation state
  const [showNewRecipientForm, setShowNewRecipientForm] = createSignal(false);
  const [newRecipientName, setNewRecipientName] = createSignal('');
  const [newRecipientEmail, setNewRecipientEmail] = createSignal('');
  const [creatingRecipient, setCreatingRecipient] = createSignal(false);

  // Initialize form when task changes
  createEffect(() => {
    const task = props.task;
    if (task) {
      setTaskName(task.name);
      setSelectedLineIds([...task.lineIds]);
      setTaskTypes([...task.taskTypes]);
      setRunMode(task.runMode);
      if (task.schedule) {
        setSelectedDays([...task.schedule.days]);
        setTimes([...task.schedule.times]);
      } else {
        setSelectedDays([]);
        setTimes(['09:00']);
      }
      setNameError(null);

      // Load notification config
      taskNotificationApi.get(task.id)
        .then((config) => {
          if (config) {
            setSmtpConfigId(config.smtpConfigId ?? undefined);
            setEmailSubject(config.emailSubject ?? '');

            // Transform TO/CC arrays into unified recipients list
            const toRecipients: EmailRecipient[] = config.toRecipientIds.map(id => ({
              emailId: id,
              type: 'to' as const,
            }));
            const ccRecipients: EmailRecipient[] = config.ccRecipientIds.map(id => ({
              emailId: id,
              type: 'cc' as const,
            }));
            setRecipients([...toRecipients, ...ccRecipients]);
          } else {
            // Reset notification state if no config exists
            setSmtpConfigId(undefined);
            setEmailSubject('');
            setRecipients([]);
          }
        })
        .catch((error) => {
          console.error('[EditTaskModal] Failed to load notification config:', error);
        });
    }
  });

  // Form validation
  const isValid = () => {
    if (taskName().trim().length === 0) return false;
    if (selectedLineIds().length === 0) return false;
    if (taskTypes().length === 0) return false;
    if (runMode() === 'scheduled') {
      if (selectedDays().length === 0 || times().length === 0) return false;
    }
    return true;
  };

  // Handlers
  const handleLineToggle = (lineId: number) => {
    const currentIds = selectedLineIds();
    if (currentIds.includes(lineId)) {
      setSelectedLineIds(currentIds.filter((id: number) => id !== lineId));
    } else {
      setSelectedLineIds([...currentIds, lineId]);
    }
  };

  const handleTaskTypeToggle = (type: 'speed_test' | 'quota_check') => {
    const currentTypes = taskTypes();
    if (currentTypes.includes(type)) {
      if (currentTypes.length > 1) {
        setTaskTypes(currentTypes.filter((t: 'speed_test' | 'quota_check') => t !== type));
      }
    } else {
      setTaskTypes([...currentTypes, type]);
    }
  };

  const handleDayToggle = (day: number) => {
    const current = selectedDays();
    const newDays = current.includes(day)
      ? current.filter((d: number) => d !== day)
      : [...current, day].sort((a: number, b: number) => {
          const orderA = a === 0 ? 7 : a;
          const orderB = b === 0 ? 7 : b;
          return orderA - orderB;
        });
    setSelectedDays(newDays);
  };

  const handleTimeChange = (index: number, newTime: string) => {
    const currentTimes = [...times()];
    currentTimes[index] = newTime;
    setTimes(currentTimes);
  };

  const addTime = () => {
    const currentTimes = times();
    const lastTime = currentTimes[currentTimes.length - 1];
    const [hours, minutes] = lastTime.split(':').map(Number);
    const nextHour = (hours + 1) % 24;
    const newTime = `${nextHour.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
    setTimes([...currentTimes, newTime]);
  };

  const removeTime = (index: number) => {
    const currentTimes = times();
    if (currentTimes.length <= 1) return;
    setTimes(currentTimes.filter((_: string, i: number) => i !== index));
  };

  const handleRunModeChange = (mode: 'one_time' | 'scheduled') => {
    setRunMode(mode);
    if (mode === 'one_time') {
      setSelectedDays([]);
      setTimes(['09:00']);
    }
  };

  // Notification handlers
  const handleSmtpChange = (value: string | null) => {
    setSmtpConfigId(value ? parseInt(value, 10) : undefined);
  };

  const handleAddRecipient = (emailId: string | null) => {
    if (!emailId) return;
    const id = parseInt(emailId, 10);
    // Check if already added
    if (recipients().some(r => r.emailId === id)) return;
    // Add with default type 'to'
    setRecipients([...recipients(), { emailId: id, type: 'to' }]);
  };

  const handleRecipientTypeChange = (emailId: number, newType: string | null) => {
    if (!newType || (newType !== 'to' && newType !== 'cc')) return;
    setRecipients(
      recipients().map(r =>
        r.emailId === emailId ? { ...r, type: newType as 'to' | 'cc' } : r
      )
    );
  };

  const handleRemoveRecipient = (emailId: number) => {
    setRecipients(recipients().filter(r => r.emailId !== emailId));
  };

  // Get available emails for adding (not already in the list)
  const availableEmails = () => {
    const activeEmails = emails()?.filter(e => e.isActive) ?? [];
    const addedIds = recipients().map(r => r.emailId);
    return activeEmails.filter(e => !addedIds.includes(e.id));
  };

  const handleCreateRecipient = async () => {
    const email = newRecipientEmail().trim();
    if (!email) return;

    setCreatingRecipient(true);
    try {
      const created = await emailsApi.create({
        name: newRecipientName().trim() || undefined,
        recipient: email,
        isActive: true,
      } as EmailCreate);
      await refetchEmails();
      // Auto-add to recipients list as 'to'
      setRecipients([...recipients(), { emailId: created.id, type: 'to' }]);
      // Reset form
      setNewRecipientName('');
      setNewRecipientEmail('');
      setShowNewRecipientForm(false);
      showToast({ title: 'Recipient created and added', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to create recipient', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setCreatingRecipient(false);
    }
  };

  const handleSave = async () => {
    if (!props.task || !isValid()) return;

    setSaving(true);
    try {
      // Check name availability if changed
      if (taskName() !== props.task!.name) {
        const available = await tasksApi.checkNameAvailable(taskName().trim());
        if (!available) {
          setNameError('This task name is already taken');
          setSaving(false);
          return;
        }
      }

      const request: CreateTaskRequest = {
        name: taskName().trim(),
        lineIds: selectedLineIds(),
        taskTypes: taskTypes(),
        runMode: runMode(),
        schedule: runMode() === 'scheduled'
          ? { days: selectedDays(), times: times() }
          : undefined,
      };

      await tasksApi.update(props.task.id, request);

      // Save notification config
      // Transform recipients list back to TO/CC arrays
      const recipientsList = recipients();
      const toRecipientIds = recipientsList
        .filter(r => r.type === 'to')
        .map(r => r.emailId);
      const ccRecipientIds = recipientsList
        .filter(r => r.type === 'cc')
        .map(r => r.emailId);

      await taskNotificationApi.upsert(props.task.id, {
        isEnabled: recipientsList.length > 0, // Enable if there are any recipients
        smtpConfigId: smtpConfigId(),
        emailSubject: emailSubject() || undefined,
        toRecipientIds,
        ccRecipientIds,
      });

      showToast({ title: 'Task updated', variant: 'success', duration: 3000 });
      await props.onSave();
      props.onOpenChange(false);
    } catch (error) {
      showToast({ title: 'Failed to update task', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      setNameError('Failed to save task. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  const formatDayLabel = (days: number[]) => {
    return days.map(d => DAYS_OF_WEEK.find(day => day.value === d)?.label).join(', ');
  };

  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[700px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Edit Task</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>

        <div class="px-4 py-3 space-y-4">
          {/* Task Name */}
          <div>
            <label class={labelClass}>Task Name</label>
            <input
              type="text"
              class={cn(primaryInputClass, nameError() && 'border-[#c72e0f]')}
              value={taskName()}
              onInput={(e) => {
                setTaskName(e.currentTarget.value);
                setNameError(null);
              }}
              placeholder="e.g., Daily Office Speed Test"
            />
            <Show when={nameError()}>
              <p class="text-[10px] text-[#c72e0f] mt-1">
                {nameError()}
              </p>
            </Show>
          </div>

          <div class="h-px bg-[#3c3c3c]" />

          {/* Select Lines */}
          <div>
            <div class={sectionHeaderClass}>Select Lines</div>
            <p class="text-[10px] text-[#808080] mb-2">Choose which internet lines should be included in this task.</p>

            <Show when={!lines.loading && lines()}>
              <div class="flex items-center gap-2 text-[10px] text-[#808080] mb-2">
                <span>{selectedLineIds().length} of {lines()?.length || 0} selected</span>
              </div>
            </Show>

            <div class="space-y-1 max-h-[150px] overflow-y-auto">
              <Show when={lines.loading}>
                <div class="text-center py-3 text-[11px] text-[#808080]">
                  Loading lines...
                </div>
              </Show>

              <Show when={!lines.loading && lines() && lines()!.length > 0}>
                <For each={lines()}>
                  {(line) => {
                    const isSelected = () => selectedLineIds().includes(line.id);
                    return (
                      <div
                        onClick={() => handleLineToggle(line.id)}
                        class={cn(
                          'group flex items-start gap-2 px-2 py-1.5 rounded-[2px] border cursor-pointer transition-all',
                          'bg-[#2d2d2d] border-[#3c3c3c]',
                          'hover:border-[#007acc]/50 hover:bg-[#2d2d2d]/80',
                          isSelected() && 'border-[#007acc] bg-[#007acc]/10'
                        )}
                      >
                        <Checkbox
                          checked={isSelected()}
                          onChange={() => handleLineToggle(line.id)}
                          class="pointer-events-none"
                        />
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center gap-2">
                            <span class="text-[11px] text-[#cccccc]">
                              {line.name}
                            </span>
                            <span class="text-[10px] text-[#808080]">
                              ({line.lineNumber})
                            </span>
                          </div>
                          <div class="text-[10px] text-[#808080]">
                            {line.isp} • {line.ipAddress}
                          </div>
                        </div>
                      </div>
                    );
                  }}
                </For>
              </Show>

              <Show when={!lines.loading && lines()?.length === 0}>
                <div class="text-center py-3 text-[11px] text-[#808080]">
                  No lines configured.
                </div>
              </Show>
            </div>
          </div>

          <div class="h-px bg-[#3c3c3c]" />

          {/* Task Types */}
          <div>
            <div class={sectionHeaderClass}>Task Types</div>
            <p class="text-[10px] text-[#808080] mb-2">Choose what type of test or check this task should perform.</p>

            <div class="grid grid-cols-2 gap-2">
              <button
                type="button"
                onClick={() => handleTaskTypeToggle('speed_test')}
                class={cn(
                  'group relative flex items-center gap-2 px-3 py-2 rounded-[3px] border-2 transition-all cursor-pointer text-left',
                  'bg-[#2d2d2d]',
                  taskTypes().includes('speed_test')
                    ? 'border-[#007acc]'
                    : 'border-[#3c3c3c] hover:border-[#007acc]/50'
                )}
              >
                <div
                  class={cn(
                    'w-8 h-8 rounded-[2px] flex items-center justify-center transition-colors',
                    taskTypes().includes('speed_test')
                      ? 'bg-[#007acc] text-white'
                      : 'bg-[#3c3c3c] text-[#808080]'
                  )}
                >
                  <svg class="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M11.251.068a.5.5 0 0 1 .227.58l-6.5 11.5a.5.5 0 0 1-.883-.46l-3.5-6.5a.5.5 0 1 1 .884-.532l3.043 5.658L10.592.468a.5.5 0 0 1 .66-.4z"/>
                  </svg>
                </div>
                <div class="flex-1">
                  <div class="text-[11px] text-[#cccccc]">Speed Test</div>
                  <div class="text-[10px] text-[#808080]">Test network speeds</div>
                </div>
                <Show when={taskTypes().includes('speed_test')}>
                  <div class="absolute top-2 right-2 w-4 h-4 rounded bg-[#007acc] flex items-center justify-center">
                    <svg class="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                    </svg>
                  </div>
                </Show>
              </button>

              <button
                type="button"
                onClick={() => handleTaskTypeToggle('quota_check')}
                class={cn(
                  'group relative flex items-center gap-2 px-3 py-2 rounded-[3px] border-2 transition-all cursor-pointer text-left',
                  'bg-[#2d2d2d]',
                  taskTypes().includes('quota_check')
                    ? 'border-[#007acc]'
                    : 'border-[#3c3c3c] hover:border-[#007acc]/50'
                )}
              >
                <div
                  class={cn(
                    'w-8 h-8 rounded-[2px] flex items-center justify-center transition-colors',
                    taskTypes().includes('quota_check')
                      ? 'bg-[#007acc] text-white'
                      : 'bg-[#3c3c3c] text-[#808080]'
                  )}
                >
                  <svg class="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M4 11a1 1 0 1 1 2 0v1a1 1 0 1 1-2 0v-1zm6-4a1 1 0 1 1 2 0v5a1 1 0 1 1-2 0V7zM7 9a1 1 0 0 0 1-1V3a1 1 0 0 0-2 0v6a1 1 0 0 0 1 1zm-5 0a1 1 0 0 0 1-1V3a1 1 0 0 0-2 0v6a1 1 0 0 0 1 1zm9.016 7A5.96 5.96 0 0 1 11 3.118 7 6 0 0 0 11 16z"/>
                  </svg>
                </div>
                <div class="flex-1">
                  <div class="text-[11px] text-[#cccccc]">Quota Check</div>
                  <div class="text-[10px] text-[#808080]">Check data usage</div>
                </div>
                <Show when={taskTypes().includes('quota_check')}>
                  <div class="absolute top-2 right-2 w-4 h-4 rounded bg-[#007acc] flex items-center justify-center">
                    <svg class="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                    </svg>
                  </div>
                </Show>
              </button>
            </div>
          </div>

          <div class="h-px bg-[#3c3c3c]" />

          {/* Run Mode */}
          <div>
            <div class={sectionHeaderClass}>Run Mode</div>
            <p class="text-[10px] text-[#808080] mb-2">Choose when and how often this task should run.</p>

            <div class="grid grid-cols-2 gap-2">
              <button
                type="button"
                onClick={() => handleRunModeChange('one_time')}
                class={cn(
                  'group relative flex items-center gap-2 px-3 py-2 rounded-[3px] border-2 transition-all cursor-pointer text-left',
                  'bg-[#2d2d2d]',
                  runMode() === 'one_time'
                    ? 'border-[#007acc]'
                    : 'border-[#3c3c3c] hover:border-[#007acc]/50'
                )}
              >
                <div
                  class={cn(
                    'w-8 h-8 rounded-[2px] flex items-center justify-center transition-colors',
                    runMode() === 'one_time'
                      ? 'bg-[#007acc] text-white'
                      : 'bg-[#3c3c3c] text-[#808080]'
                  )}
                >
                  <svg class="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M10.804 8 5 4.633v6.734L10.804 8zm.792-.696a.802.802 0 0 1 0 1.392l-6.363 3.692C4.713 12.69 4 12.345 4 11.692V4.308c0-.653.713-.998 1.233-.696l6.363 3.692z"/>
                  </svg>
                </div>
                <div class="flex-1">
                  <div class="text-[11px] text-[#cccccc]">One Time</div>
                  <div class="text-[10px] text-[#808080]">Run manually</div>
                </div>
                <Show when={runMode() === 'one_time'}>
                  <div class="absolute top-2 right-2 w-4 h-4 rounded bg-[#007acc] flex items-center justify-center">
                    <svg class="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                    </svg>
                  </div>
                </Show>
              </button>

              <button
                type="button"
                onClick={() => handleRunModeChange('scheduled')}
                class={cn(
                  'group relative flex items-center gap-2 px-3 py-2 rounded-[3px] border-2 transition-all cursor-pointer text-left',
                  'bg-[#2d2d2d]',
                  runMode() === 'scheduled'
                    ? 'border-[#007acc]'
                    : 'border-[#3c3c3c] hover:border-[#007acc]/50'
                )}
              >
                <div
                  class={cn(
                    'w-8 h-8 rounded-[2px] flex items-center justify-center transition-colors',
                    runMode() === 'scheduled'
                      ? 'bg-[#007acc] text-white'
                      : 'bg-[#3c3c3c] text-[#808080]'
                  )}
                >
                  <svg class="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
                    <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
                  </svg>
                </div>
                <div class="flex-1">
                  <div class="text-[11px] text-[#cccccc]">Scheduled</div>
                  <div class="text-[10px] text-[#808080]">Automatic</div>
                </div>
                <Show when={runMode() === 'scheduled'}>
                  <div class="absolute top-2 right-2 w-4 h-4 rounded bg-[#007acc] flex items-center justify-center">
                    <svg class="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                    </svg>
                  </div>
                </Show>
              </button>
            </div>

            {/* Schedule Configuration */}
            <Show when={runMode() === 'scheduled'}>
              <div class="space-y-3 px-3 py-3 bg-[#252525] border border-[#3c3c3c] rounded-[3px]">
                {/* Days */}
                <div>
                  <label class={labelClass}>Days of Week</label>
                  <div class="flex flex-wrap gap-1">
                    <For each={DAYS_OF_WEEK}>
                      {(day) => (
                        <button
                          type="button"
                          onClick={() => handleDayToggle(day.value)}
                          class={cn(
                            'px-2 py-1 rounded-[2px] text-[10px] font-medium transition-colors border',
                            selectedDays().includes(day.value)
                              ? 'bg-[#007acc] border-[#007acc] text-white'
                              : 'bg-[#2d2d2d] border-[#3c3c3c] text-[#cccccc] hover:border-[#007acc]/50'
                          )}
                        >
                          {day.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>

                {/* Times */}
                <div>
                  <div class="flex items-center justify-between mb-1">
                    <label class={labelClass}>Times</label>
                    <button
                      type="button"
                      onClick={addTime}
                      class="text-[10px] text-[#007acc] hover:underline"
                    >
                      + Add Time
                    </button>
                  </div>
                  <div class="space-y-1">
                    <For each={times()}>
                      {(timeValue, index) => (
                        <div class="flex items-center gap-2">
                          <TimeInput
                            value={timeValue}
                            onChange={(newValue) => handleTimeChange(index(), newValue)}
                            class="flex-1"
                          />
                          <Show when={times().length > 1}>
                            <button
                              type="button"
                              onClick={() => removeTime(index())}
                              class="px-2 py-1 rounded-[2px] text-[10px] bg-[#3c3c3c] text-[#808080] hover:bg-[#c72e0f]/20 hover:text-[#c72e0f] transition-colors"
                            >
                              Remove
                            </button>
                          </Show>
                        </div>
                      )}
                    </For>
                  </div>
                </div>

                {/* Preview */}
                <Show when={selectedDays().length > 0 && times().length > 0}>
                  <div class="bg-[#2d2d2d] border border-[#3c3c3c] rounded-[3px] px-3 py-2">
                    <p class="text-[10px] text-[#808080]">
                      Runs every <span class="text-[#cccccc] font-medium">{formatDayLabel(selectedDays())}</span> at <span class="text-[#cccccc] font-medium">{times().sort().join(', ')}</span>
                    </p>
                  </div>
                </Show>
              </div>
            </Show>
          </div>

          <div class="h-px bg-[#3c3c3c]" />

          {/* Notifications */}
          <div>
            <div class={sectionHeaderClass}>Notifications</div>
            <p class="text-[10px] text-[#808080] mb-2">Configure email notifications for task results.</p>

            <div class="space-y-3 px-3 py-3 bg-[#252525] border border-[#3c3c3c] rounded-[3px]">
              {/* SMTP Server Selection */}
              <div>
                <label class={labelClass}>SMTP Server</label>
                <select
                  class={selectClass}
                  value={smtpConfigId()?.toString() ?? ''}
                  onChange={(e) => handleSmtpChange(e.currentTarget.value || null)}
                >
                  <option value="">Default SMTP Server</option>
                  <For each={smtpConfigs()?.filter(c => c.isActive) ?? []}>
                    {(config) => (
                      <option value={config.id.toString()}>{config.name}</option>
                    )}
                  </For>
                </select>
                <Show when={!smtpConfigs.loading && smtpConfigs()?.filter(c => c.isActive).length === 0}>
                  <p class="text-[10px] text-[#808080] mt-1">
                    No SMTP servers configured. Default server will be used.
                  </p>
                </Show>
              </div>

              {/* Email Subject */}
              <div>
                <label class={labelClass}>Email Subject (optional)</label>
                <input
                  type="text"
                  class={inputClass}
                  value={emailSubject()}
                  onInput={(e) => setEmailSubject(e.currentTarget.value)}
                  placeholder="NetNinja Task Results"
                />
              </div>

              {/* Email Recipients */}
              <div>
                <label class={labelClass}>Email Recipients</label>

                {/* Recipients List */}
                <div class="space-y-1 mb-2">
                  <Show
                    when={recipients().length > 0}
                    fallback={
                      <div class="text-[10px] text-[#808080] py-1 text-center">
                        No recipients added yet
                      </div>
                    }
                  >
                    <For each={recipients()}>
                      {(recipient) => {
                        const email = () => emails()?.find(e => e.id === recipient.emailId);
                        return (
                          <div class="flex items-center gap-2 px-2 py-1.5 bg-[#2d2d2d] border border-[#3c3c3c] rounded-[3px]">
                            {/* Email info */}
                            <div class="flex-1 min-w-0">
                              <div class="text-[11px] text-[#cccccc] truncate">
                                {email()?.name || 'Unnamed'}
                              </div>
                              <div class="text-[10px] text-[#808080] truncate">
                                {email()?.recipient}
                              </div>
                            </div>

                            {/* Type selector */}
                            <select
                              class="h-[22px] px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[10px] rounded-none focus:outline-none focus:border-[#007acc] cursor-pointer w-16"
                              value={recipient.type}
                              onChange={(e) => handleRecipientTypeChange(recipient.emailId, e.currentTarget.value)}
                            >
                              <option value="to">To</option>
                              <option value="cc">CC</option>
                            </select>

                            {/* Remove button */}
                            <button
                              type="button"
                              onClick={() => handleRemoveRecipient(recipient.emailId)}
                              class="p-1 rounded-[2px] transition-colors bg-[#3c3c3c] text-[#808080] hover:bg-[#c72e0f]/20 hover:text-[#c72e0f]"
                              title="Remove recipient"
                            >
                              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"/>
                                <path d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1zM4.118 4 4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/>
                              </svg>
                            </button>
                          </div>
                        );
                      }}
                    </For>
                  </Show>
                </div>

                {/* Add Recipient Controls */}
                <div class="flex items-center gap-2">
                  <Show when={availableEmails().length > 0}>
                    <select
                      class={cn(selectClass, 'flex-1')}
                      value=""
                      onChange={(e) => handleAddRecipient(e.currentTarget.value)}
                    >
                      <option value="">+ Add Existing Recipient</option>
                      <For each={availableEmails()}>
                        {(email) => (
                          <option value={email.id.toString()}>{email.name || email.recipient}</option>
                        )}
                      </For>
                    </select>
                  </Show>
                  <button
                    type="button"
                    class={cn(
                      'h-[28px] px-3 text-[11px] rounded-[2px] border transition-colors whitespace-nowrap',
                      'border-[#007acc] text-[#007acc] hover:bg-[#007acc]/10',
                      availableEmails().length === 0 && 'flex-1'
                    )}
                    onClick={() => setShowNewRecipientForm(!showNewRecipientForm())}
                  >
                    {showNewRecipientForm() ? 'Cancel' : '+ New Recipient'}
                  </button>
                </div>

                {/* Inline New Recipient Form */}
                <Show when={showNewRecipientForm()}>
                  <div class="mt-2 p-2.5 bg-[#2d2d2d] border border-[#007acc]/30 rounded-[3px] space-y-2">
                    <div class="flex items-center gap-2">
                      <input
                        type="text"
                        class={cn(inputClass, 'flex-1')}
                        value={newRecipientName()}
                        onInput={(e) => setNewRecipientName(e.currentTarget.value)}
                        placeholder="Name (optional)"
                      />
                      <input
                        type="email"
                        class={cn(inputClass, 'flex-[2]')}
                        value={newRecipientEmail()}
                        onInput={(e) => setNewRecipientEmail(e.currentTarget.value)}
                        placeholder="email@example.com"
                        onKeyDown={(e) => { if (e.key === 'Enter') handleCreateRecipient(); }}
                      />
                      <button
                        type="button"
                        class="h-[28px] px-3 text-[11px] rounded-[2px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
                        onClick={handleCreateRecipient}
                        disabled={!newRecipientEmail().trim() || creatingRecipient()}
                      >
                        {creatingRecipient() ? 'Adding...' : 'Add'}
                      </button>
                    </div>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </div>

        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={() => props.onOpenChange(false)}
            disabled={saving()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={handleSave}
            disabled={!isValid() || saving()}
          >
            {saving() ? 'Saving...' : 'Save Changes'}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
