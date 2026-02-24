import { type Component, createSignal, createMemo, Show, onMount, For } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog';
import { cn } from '~/lib/utils';
import type { CreateTaskRequest, Schedule, Task } from '~/types';
import { tasksApi, taskNotificationApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';

import { StepTaskName } from './steps/StepTaskName';
import { StepLineSelection } from './steps/StepLineSelection';
import { StepTaskType } from './steps/StepTaskType';
import { StepRunMode } from './steps/StepRunMode';
import { StepNotifications, type NotificationConfig } from './steps/StepNotifications';

export interface CreateTaskWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: (data: CreateTaskRequest) => Promise<number>;
  onSuccess?: () => void | Promise<void>;
  task?: Task;
}

const STEPS = [
  { id: 1, title: 'Task Name', description: 'Enter a unique name for your task' },
  { id: 2, title: 'Select Lines', description: 'Choose which lines to test' },
  { id: 3, title: 'Task Type', description: 'Speed test or quota check' },
  { id: 4, title: 'Run Mode', description: 'One-time or scheduled' },
  { id: 5, title: 'Notifications', description: 'Email notification settings' },
] as const;

const cancelBtnClass = "h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] disabled:opacity-50 disabled:cursor-not-allowed transition-colors";
const primaryBtnClass = "h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors";

export const CreateTaskWizard: Component<CreateTaskWizardProps> = (props) => {
  const [currentStep, setCurrentStep] = createSignal(1);
  const [saving, setSaving] = createSignal(false);
  const isEditMode = () => props.task !== undefined;

  // Form data
  const [taskName, setTaskName] = createSignal('');
  const [selectedLineIds, setSelectedLineIds] = createSignal<number[]>([]);
  const [taskTypes, setTaskTypes] = createSignal<('speed_test' | 'quota_check')[]>([]);
  const [showBrowser, setShowBrowser] = createSignal(false);  // Browser visibility for quota check (default: hidden)
  const [runMode, setRunMode] = createSignal<'one_time' | 'scheduled' | ''>('');
  const [schedule, setSchedule] = createSignal<Schedule | null>(null);
  const [notificationConfig, setNotificationConfig] = createSignal<NotificationConfig>({
    isEnabled: false,
    toRecipientIds: [],
    ccRecipientIds: [],
  });

  // Pre-populate form when editing
  onMount(async () => {
    const task = props.task;
    if (task) {
      setTaskName(task.name);
      setSelectedLineIds(task.lineIds);
      setTaskTypes(task.taskTypes);
      setShowBrowser(task.showBrowser);
      setRunMode(task.runMode);
      setSchedule(task.schedule || null);

      // Load notification config
      try {
        const config = await taskNotificationApi.get(task.id);
        if (config) {
          setNotificationConfig({
            isEnabled: config.isEnabled,
            smtpConfigId: config.smtpConfigId ?? undefined,
            emailSubject: config.emailSubject ?? undefined,
            toRecipientIds: config.toRecipientIds,
            ccRecipientIds: config.ccRecipientIds,
          });
        }
      } catch (error) {
        showToast({ title: 'Failed to load notification config', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      }
    }
  });

  // Validation for each step
  const isStep1Valid = createMemo(() => taskName().trim().length > 0);
  const isStep2Valid = createMemo(() => selectedLineIds().length > 0);
  const isStep3Valid = createMemo(() => taskTypes().length > 0);
  const isStep4Valid = createMemo(() => {
    if (runMode() === '') return false;
    if (runMode() === 'scheduled') {
      const sched = schedule();
      return sched !== null && sched.days.length > 0 && sched.times.length > 0;
    }
    return true;
  });
  const isStep5Valid = createMemo(() => {
    const config = notificationConfig();
    // Step is always valid - notifications are optional
    // But if enabled, require SMTP and at least one TO recipient
    if (!config.isEnabled) return true;
    return config.smtpConfigId !== undefined && config.toRecipientIds.length > 0;
  });

  const canGoNext = createMemo(() => {
    switch (currentStep()) {
      case 1: return isStep1Valid();
      case 2: return isStep2Valid();
      case 3: return isStep3Valid();
      case 4: return isStep4Valid();
      case 5: return isStep5Valid();
      default: return false;
    }
  });

  const isLastStep = createMemo(() => currentStep() === STEPS.length);

  const handleNext = () => {
    if (canGoNext() && !isLastStep()) {
      setCurrentStep(currentStep() + 1);
    }
  };

  const handleBack = () => {
    if (currentStep() > 1) {
      setCurrentStep(currentStep() - 1);
    }
  };

  const handleComplete = async () => {
    if (!canGoNext()) return;

    setSaving(true);
    try {
      const request: CreateTaskRequest = {
        name: taskName().trim(),
        lineIds: selectedLineIds(),
        taskTypes: taskTypes(),
        showBrowser: showBrowser(),
        runMode: runMode() as 'one_time' | 'scheduled',
        schedule: runMode() === 'scheduled' ? schedule() || undefined : undefined,
      };

      const task = props.task;
      let taskId: number;

      if (task) {
        // Edit mode - call update
        await tasksApi.update(task.id, request);
        taskId = task.id;
        // Call onSuccess callback to refresh the task list
        await props.onSuccess?.();
      } else {
        // Create mode - let parent handle creation and return task ID
        taskId = await props.onComplete(request);
      }

      // Save notification config
      const config = notificationConfig();
      await taskNotificationApi.upsert(taskId, {
        isEnabled: config.isEnabled,
        smtpConfigId: config.smtpConfigId,
        emailSubject: config.emailSubject,
        toRecipientIds: config.toRecipientIds,
        ccRecipientIds: config.ccRecipientIds,
      });

      showToast({ title: task ? 'Task updated' : 'Task created', variant: 'success', duration: 3000 });
      handleReset();
      props.onOpenChange(false);
    } catch (error) {
      showToast({ title: 'Failed to save task', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setCurrentStep(1);
    setTaskName('');
    setSelectedLineIds([]);
    setTaskTypes([]);
    setShowBrowser(false);
    setRunMode('');
    setSchedule(null);
    setNotificationConfig({
      isEnabled: false,
      toRecipientIds: [],
      ccRecipientIds: [],
    });
  };

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      handleReset();
    }
    props.onOpenChange(open);
  };

  return (
    <Dialog open={props.open} onOpenChange={handleOpenChange}>
      <DialogContent class="max-w-[720px]">
        <DialogHeader>
          <DialogTitle>{isEditMode() ? 'Edit Task' : 'Create New Task'}</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>

        {/* Progress indicator */}
        <div class="flex items-center gap-2 px-4 py-2 bg-[#1e1e1e] border-b border-[#3c3c3c]">
          <For each={STEPS}>
            {(step, index) => (
              <>
                <div class="flex items-center gap-2 flex-1 min-w-0">
                  <div
                    class={cn(
                      'w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-medium transition-colors shrink-0',
                      currentStep() > step.id
                        ? 'bg-[#388a34] text-white'
                        : currentStep() === step.id
                        ? 'bg-[#007acc] text-white'
                        : 'bg-[#3c3c3c] text-[#808080]'
                    )}
                  >
                    <Show when={currentStep() > step.id} fallback={step.id}>
                      <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={3} d="M5 13l4 4L19 7" />
                      </svg>
                    </Show>
                  </div>
                  <span
                    class={cn(
                      'text-[10px] truncate',
                      currentStep() >= step.id ? 'text-[#cccccc]' : 'text-[#555]'
                    )}
                  >
                    {step.title}
                  </span>
                </div>
                <Show when={index() < STEPS.length - 1}>
                  <div
                    class={cn(
                      'h-px w-6 transition-colors shrink-0',
                      currentStep() > step.id ? 'bg-[#388a34]' : 'bg-[#3c3c3c]'
                    )}
                  />
                </Show>
              </>
            )}
          </For>
        </div>

        {/* Step content */}
        <div class="min-h-[280px] px-4 py-3">
          <Show when={currentStep() === 1}>
            <StepTaskName
              value={taskName()}
              onChange={setTaskName}
              isValid={isStep1Valid()}
            />
          </Show>
          <Show when={currentStep() === 2}>
            <StepLineSelection
              selectedLineIds={selectedLineIds()}
              onChange={setSelectedLineIds}
            />
          </Show>
          <Show when={currentStep() === 3}>
            <StepTaskType
              value={taskTypes()}
              onChange={setTaskTypes}
              showBrowser={showBrowser()}
              onShowBrowserChange={setShowBrowser}
            />
          </Show>
          <Show when={currentStep() === 4}>
            <StepRunMode
              runMode={runMode()}
              schedule={schedule()}
              onRunModeChange={setRunMode}
              onScheduleChange={setSchedule}
            />
          </Show>
          <Show when={currentStep() === 5}>
            <StepNotifications
              value={notificationConfig()}
              onChange={setNotificationConfig}
            />
          </Show>
        </div>

        {/* Navigation buttons */}
        <div class="flex items-center justify-between border-t border-[#3c3c3c] px-4 py-2.5">
          <button
            type="button"
            class={cancelBtnClass}
            onClick={handleBack}
            disabled={currentStep() === 1 || saving()}
          >
            <svg class="inline-block w-3 h-3 mr-1" viewBox="0 0 16 16" fill="currentColor">
              <path fill-rule="evenodd" d="M11.354 1.646a.5.5 0 0 1 0 .708L5.707 8l5.647 5.646a.5.5 0 0 1-.708.708l-6-6a.5.5 0 0 1 0-.708l6-6a.5.5 0 0 1 .708 0z"/>
            </svg>
            Back
          </button>

          <div class="text-[10px] text-[#808080]">
            Step {currentStep()} of {STEPS.length}
          </div>

          <Show
            when={isLastStep()}
            fallback={
              <button
                type="button"
                class={primaryBtnClass}
                onClick={handleNext}
                disabled={!canGoNext() || saving()}
              >
                Next
                <svg class="inline-block w-3 h-3 ml-1" viewBox="0 0 16 16" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/>
                </svg>
              </button>
            }
          >
            <button
              type="button"
              class={primaryBtnClass}
              onClick={handleComplete}
              disabled={!canGoNext() || saving()}
            >
              {saving() ? (
                <>
                  <svg class="inline-block w-3 h-3 mr-1 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                  {isEditMode() ? 'Saving...' : 'Creating...'}
                </>
              ) : (
                isEditMode() ? 'Save Task' : 'Create Task'
              )}
            </button>
          </Show>
        </div>
      </DialogContent>
    </Dialog>
  );
};
