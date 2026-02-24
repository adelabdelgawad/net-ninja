import { type Component, createSignal } from 'solid-js';
import { DesktopTable, type ColumnDef, type StatusFilter, type HoverAction } from '~/components/desktop-table';
import {
  useTasksQuery,
  useCreateTaskMutation,
  useDeleteTaskMutation,
  useToggleTaskActiveMutation,
  useUpdateTaskMutation,
  useExecuteTaskMutation,
  useStopTaskMutation,
  useTaskNotificationQuery,
  useToggleTaskNotificationMutation,
} from '~/api/queries';
import type { Task, CreateTaskRequest } from '~/types';
import { cn } from '~/lib/utils';
import { showToast } from '~/components/ui/toast';
import { CreateTaskWizard } from './tasks/_components/wizard/CreateTaskWizard';
import { EditTaskModal } from './tasks/_components/modals/EditTaskModal';
import { ExecutionHistoryModal } from './tasks/_components/modals/ExecutionHistoryModal';
import { RunNowDialog } from './tasks/_components/modals/RunNowDialog';
import { StopTaskDialog } from './tasks/_components/modals/StopTaskDialog';
import { Play, Square, Edit2, Trash2, History } from 'lucide-solid';
import type { RuntimeNotificationConfig } from '~/types';

export const Tasks: Component = () => {
  // Use TanStack Query for instant navigation with caching
  const query = useTasksQuery();

  // Mutations for CRUD operations
  const createMutation = useCreateTaskMutation();
  const deleteMutation = useDeleteTaskMutation();
  const toggleActiveMutation = useToggleTaskActiveMutation();
  const updateMutation = useUpdateTaskMutation();
  const executeMutation = useExecuteTaskMutation();
  const stopMutation = useStopTaskMutation();

  const [isWizardOpen, setIsWizardOpen] = createSignal(false);
  const [isEditModalOpen, setIsEditModalOpen] = createSignal(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = createSignal(false);
  const [isRunNowDialogOpen, setIsRunNowDialogOpen] = createSignal(false);
  const [isStopDialogOpen, setIsStopDialogOpen] = createSignal(false);
  const [editingTask, setEditingTask] = createSignal<Task | null>(null);
  const [historyTask, setHistoryTask] = createSignal<Task | null>(null);
  const [runNowTask, setRunNowTask] = createSignal<Task | null>(null);
  const [stopTask, setStopTask] = createSignal<Task | null>(null);
  const [, setExecutingTaskIds] = createSignal<Set<number>>(new Set());

  const handleCreateTask = async (data: CreateTaskRequest): Promise<number> => {
    const task = await createMutation.mutateAsync(data);
    return task.id;
  };

  const handleToggleActive = async (task: Task, isActive: boolean) => {
    try {
      await toggleActiveMutation.mutateAsync({ id: task.id, isActive });
      showToast({ title: `Task ${isActive ? 'activated' : 'deactivated'}`, variant: 'success', duration: 3000 });
    } catch (error) {
      showToast({ title: 'Failed to toggle task status', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  const handleToggleShowBrowser = async (task: Task, showBrowser: boolean) => {
    try {
      await updateMutation.mutateAsync({ id: task.id, data: { showBrowser } });
      showToast({ title: `Browser ${showBrowser ? 'visible' : 'hidden'}`, variant: 'success', duration: 3000 });
    } catch (error) {
      showToast({ title: 'Failed to toggle browser visibility', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  const handleExecuteTask = (task: Task) => {
    if (!task.isActive || task.status === 'running') return;
    setRunNowTask(task);
    setIsRunNowDialogOpen(true);
  };

  const handleRunNowExecute = async (task: Task, notificationOverride?: RuntimeNotificationConfig) => {
    try {
      setExecutingTaskIds(prev => new Set(prev).add(task.id));
      await executeMutation.mutateAsync({ id: task.id, notificationOverride });
      showToast({ title: 'Task execution submitted', variant: 'success', duration: 3000 });
    } catch (error) {
      showToast({ title: 'Failed to execute task', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setExecutingTaskIds(prev => {
        const next = new Set(prev);
        next.delete(task.id);
        return next;
      });
    }
  };

  const handleDeleteTask = async (task: Task) => {
    if (task.status === 'running') {
      showToast({ title: 'Cannot delete a running task', variant: 'error', duration: 5000 });
      return;
    }

    if (confirm(`Are you sure you want to delete task "${task.name}"?`)) {
      try {
        await deleteMutation.mutateAsync(task.id);
        showToast({ title: 'Task deleted', variant: 'success', duration: 3000 });
      } catch (error) {
        showToast({ title: 'Failed to delete task', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      }
    }
  };

  const handleEditTask = (task: Task) => {
    if (task.status === 'running') {
      showToast({ title: 'Cannot edit a running task', variant: 'warning', duration: 5000 });
      return;
    }
    setEditingTask(task);
    setIsEditModalOpen(true);
  };

  const handleViewHistory = (task: Task) => {
    setHistoryTask(task);
    setIsHistoryModalOpen(true);
  };

  const handleStopTask = (task: Task) => {
    setStopTask(task);
    setIsStopDialogOpen(true);
  };

  const handleConfirmStop = async (task: Task) => {
    try {
      await stopMutation.mutateAsync(task.id);
      showToast({ title: 'Task stopped', description: `Task "${task.name}" has been stopped`, variant: 'success', duration: 3000 });
    } catch (error) {
      showToast({ title: 'Failed to stop task', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  // Status filters
  const statusFilters: StatusFilter<Task>[] = [
    {
      key: 'pending',
      label: 'Pending',
      count: (data) => data.filter((t: Task) => t.status === 'pending').length,
      filter: (item) => item.status === 'pending',
    },
    {
      key: 'running',
      label: 'Running',
      count: (data) => data.filter((t: Task) => t.status === 'running').length,
      filter: (item) => item.status === 'running',
    },
    {
      key: 'completed',
      label: 'Completed',
      count: (data) => data.filter((t: Task) => t.status === 'completed').length,
      filter: (item) => item.status === 'completed',
    },
    {
      key: 'failed',
      label: 'Failed',
      count: (data) => data.filter((t: Task) => t.status === 'failed').length,
      filter: (item) => item.status === 'failed',
    },
  ];

  // Format task type for display
  const formatTaskType = (type: string): string => {
    switch (type) {
      case 'speed_test':
        return 'Speed Test';
      case 'quota_check':
        return 'Quota Check';
      default:
        return type;
    }
  };

  // Format run mode for display
  const formatRunMode = (mode: string): string => {
    switch (mode) {
      case 'one_time':
        return 'One Time';
      case 'scheduled':
        return 'Scheduled';
      default:
        return mode;
    }
  };

  // Format status badge
  const StatusBadge = (props: { status: string }) => {
    const colors = {
      pending: 'bg-[#5a5a5a] text-[#cccccc]',
      running: 'bg-[#3584e4] text-white',
      completed: 'bg-[#388a34] text-white',
      failed: 'bg-[#c72e0f] text-white',
    };
    const color = colors[props.status as keyof typeof colors] || colors.pending;

    return (
      <span class={cn(
        'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[8px] text-[10px] font-medium uppercase tracking-wide',
        color
      )}>
        {props.status === 'running' && (
          <svg class="animate-spin h-3 w-3" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
        )}
        {props.status}
      </span>
    );
  };

  // Lines count with tooltip
  const LinesCount = (props: { task: Task }) => {
    const lineCount = props.task.lineIds.length;
    const linesList = props.task.lines.map(l => l.name).join(', ');

    return (
      <span
        class="text-xs text-[#cccccc] cursor-help"
        title={linesList || 'No lines assigned'}
      >
        {lineCount} {lineCount === 1 ? 'line' : 'lines'}
      </span>
    );
  };

  // Toggle switch component
  const ToggleSwitch = (props: { checked: boolean; onChange: (checked: boolean) => void; disabled?: boolean }) => (
    <button
      type="button"
      role="switch"
      aria-checked={props.checked}
      disabled={props.disabled}
      onClick={() => !props.disabled && props.onChange(!props.checked)}
      class={cn(
        'relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-[#3584e4] focus:ring-offset-2 focus:ring-offset-[#1e1e1e]',
        props.checked ? 'bg-[#3584e4]' : 'bg-[#3c3c3c]',
        props.disabled && 'opacity-50 cursor-not-allowed'
      )}
    >
      <span
        class={cn(
          'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
          props.checked ? 'translate-x-4' : 'translate-x-0'
        )}
      />
    </button>
  );

  // Notification toggle component
  const NotificationToggle = (props: { taskId: number }) => {
    const notificationQuery = useTaskNotificationQuery(props.taskId);
    const toggleMutation = useToggleTaskNotificationMutation();

    const isEnabled = () => notificationQuery.data?.isEnabled ?? false;
    const isLoading = () => notificationQuery.isLoading || toggleMutation.isPending;

    const handleToggle = async (checked: boolean) => {
      try {
        await toggleMutation.mutateAsync({ taskId: props.taskId, isEnabled: checked });
        showToast({ title: `Notifications ${checked ? 'enabled' : 'disabled'}`, variant: 'success', duration: 3000 });
      } catch (error) {
        showToast({ title: 'Failed to toggle notification', description: error instanceof Error ? error.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
      }
    };

    return (
      <div class="flex items-center justify-center">
        <ToggleSwitch
          checked={isEnabled()}
          onChange={handleToggle}
          disabled={isLoading()}
        />
      </div>
    );
  };

  const columns: ColumnDef<Task>[] = [
    {
      key: 'id',
      header: 'ID',
      sortable: true,
      width: '4%',
      align: 'center',
    },
    {
      key: 'name',
      header: 'Name',
      sortable: true,
      width: '10%',
      align: 'center',
    },
    {
      key: 'lineIds',
      header: 'Lines',
      sortable: false,
      width: '5%',
      align: 'center',
      cellRenderer: (row) => <LinesCount task={row()} />,
    },
    {
      key: 'taskTypes',
      header: 'Type',
      sortable: true,
      width: '14%',
      align: 'center',
      cellRenderer: (row) => {
        const types = row().taskTypes;
        return (
          <span class="text-xs">
            {types.map(t => formatTaskType(t)).join(', ')}
          </span>
        );
      },
    },
    {
      key: 'runMode',
      header: 'Mode',
      sortable: true,
      width: '8%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{formatRunMode(row().runMode)}</span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      sortable: true,
      width: '8%',
      align: 'center',
      cellRenderer: (row) => <StatusBadge status={row().status} />,
    },
    {
      key: 'isActive',
      header: 'Active',
      sortable: true,
      width: '6%',
      align: 'center',
      cellRenderer: (row) => {
        const task = row();
        return (
          <div class="flex items-center justify-center">
            <ToggleSwitch
              checked={task.isActive}
              onChange={(checked) => handleToggleActive(task, checked)}
            />
          </div>
        );
      },
    },
    {
      key: 'showBrowser',
      header: 'Show Browser',
      sortable: true,
      width: '7%',
      align: 'center',
      cellRenderer: (row) => {
        const task = row();
        return (
          <div class="flex items-center justify-center">
            <ToggleSwitch
              checked={task.showBrowser}
              onChange={(checked) => handleToggleShowBrowser(task, checked)}
            />
          </div>
        );
      },
    },
    {
      key: 'notifications',
      header: 'Notify',
      sortable: false,
      width: '6%',
      align: 'center',
      cellRenderer: (row) => {
        const task = row();
        return <NotificationToggle taskId={task.id} />;
      },
    },
    {
      key: 'lastRunAt',
      header: 'Last Run',
      sortable: true,
      width: '15%',
      align: 'center',
      cellRenderer: (row) => {
        const lastRunAt = row().lastRunAt;
        return (
          <span class="text-xs text-[#808080]">
            {lastRunAt ? new Date(lastRunAt).toLocaleString() : '-'}
          </span>
        );
      },
    },
    {
      key: 'nextRunAt',
      header: 'Next Run',
      sortable: true,
      width: '15%',
      align: 'center',
      cellRenderer: (row) => {
        const nextRunAt = row().nextRunAt;
        return (
          <span class="text-xs text-[#808080]">
            {nextRunAt ? new Date(nextRunAt).toLocaleString() : '-'}
          </span>
        );
      },
    },
  ];

  const hoverActions: HoverAction<Task>[] = [
    {
      icon: (props) => <Play {...props} />,
      label: 'Execute task now',
      action: 'execute',
      show: (task) => task.isActive && task.status !== 'running',
    },
    {
      icon: (props) => <Square {...props} />,
      label: 'Stop task',
      action: 'stop',
      variant: 'destructive',
      show: (task) => task.status === 'running',
    },
    {
      icon: (props) => <History {...props} />,
      label: 'View execution history',
      action: 'history',
    },
    {
      icon: (props) => <Edit2 {...props} />,
      label: 'Edit task',
      action: 'edit',
      disabled: (task) => task.status === 'running',
    },
    {
      icon: (props) => <Trash2 {...props} />,
      label: 'Delete task',
      action: 'delete',
      variant: 'destructive',
      disabled: (task) => task.status === 'running',
    },
  ];

  // Derive loading state for table
  const loadingState = () => {
    if (query.isPending && !query.data) return 'loading';
    return 'idle';
  };

  const data = () => query.data ?? [];

  const handleCloseWizard = () => {
    setIsWizardOpen(false);
    setEditingTask(null);
  };

  const handleCloseEditModal = () => {
    setIsEditModalOpen(false);
    setEditingTask(null);
  };

  return (
    <div class="flex flex-col h-full">
      {/* Wrapper for all content */}
      <div class="flex flex-col h-full overflow-hidden">
        {/* Content Header with Actions */}
        <div class="px-3 pt-2 pb-3 flex-shrink-0">
          <div class="flex items-center justify-between">
            <div>
              <h1 class="text-[20px] font-semibold text-[#eeeeee]">Tasks</h1>
              <p class="text-[13px] text-[#999999]">Manage scheduled network monitoring tasks</p>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class={cn(
                  "px-3 py-1.5 text-[11px] rounded-[8px]",
                  "bg-[#3584e4] text-white",
                  "hover:bg-[#4a9ff1]"
                )}
                onClick={() => {
                  setEditingTask(null);
                  setIsWizardOpen(true);
                }}
              >
                + Create Task
              </button>
            </div>
          </div>
        </div>

        {/* DesktopTable wrapper */}
        <div class="flex-1 overflow-hidden">
          <DesktopTable
            data={data()}
            columns={columns}
            hoverActions={hoverActions}
            loadingState={loadingState()}
            emptyMessage="No tasks configured. Click '+ Create Task' to create one."
            statusFilters={statusFilters}
            defaultPageSize={25}
            enableSearch={false}
            enablePagination={true}
            enableStatusBar={true}
            contextMenuItems={[
              { action: 'view', label: 'View Details', shortcut: 'Enter' },
              { action: 'copy', label: 'Copy Row', shortcut: 'Ctrl+C' },
              { type: 'separator' },
              { action: 'delete', label: 'Delete', shortcut: 'Del', destructive: true },
            ]}
            onRowAction={(action, row) => {
              const task = row as Task;
              switch (action) {
                case 'view':
                  // TODO: implement view details
                  break;
                case 'execute':
                  handleExecuteTask(task);
                  break;
                case 'stop':
                  handleStopTask(task);
                  break;
                case 'history':
                  handleViewHistory(task);
                  break;
                case 'edit':
                  handleEditTask(task);
                  break;
                case 'copy':
                  navigator.clipboard.writeText(JSON.stringify(row, null, 2));
                  showToast({ title: 'Copied to clipboard', variant: 'default', duration: 2000 });
                  break;
                case 'delete':
                  handleDeleteTask(task);
                  break;
              }
            }}
          />
        </div>
      </div>

      <CreateTaskWizard
        open={isWizardOpen()}
        onOpenChange={handleCloseWizard}
        onComplete={handleCreateTask}
        task={undefined}
      />

      <EditTaskModal
        open={isEditModalOpen()}
        task={editingTask()}
        onOpenChange={handleCloseEditModal}
        onSave={async () => await query.refetch()}
      />

      <ExecutionHistoryModal
        open={isHistoryModalOpen()}
        task={historyTask()}
        onOpenChange={(open) => {
          setIsHistoryModalOpen(open);
          if (!open) setHistoryTask(null);
        }}
      />

      <RunNowDialog
        open={isRunNowDialogOpen}
        task={runNowTask}
        onOpenChange={(open) => {
          setIsRunNowDialogOpen(open);
          if (!open) setRunNowTask(null);
        }}
        onExecute={handleRunNowExecute}
      />

      <StopTaskDialog
        open={isStopDialogOpen}
        task={stopTask}
        onOpenChange={(open) => {
          setIsStopDialogOpen(open);
          if (!open) setStopTask(null);
        }}
        onConfirm={handleConfirmStop}
      />
    </div>
  );
};
