import { type Component, createSignal } from 'solid-js';
import { DesktopTable, type ColumnDef, type FacetedFilterConfig, type HoverAction } from '~/components/desktop-table';
import { showToast } from '~/components/ui/toast';
import {
  useLinesQuery,
  useCreateLineMutation,
  useUpdateLineMutation,
  useDeleteLineMutation,
  useToggleLineActiveMutation,
} from '~/api/queries';
import type { Line, LineCreate, LineUpdate } from '~/types';
import {
  ViewLineDialog,
  AddLineDialog,
  DeleteLineDialog,
} from './lines/_components/modals';
import { Switch } from '@kobalte/core';
import { Edit2 } from 'lucide-solid';

export const ISP_OPTIONS = ['WE', 'Vodafone', 'Orange', 'Etisalat'] as const;

const emptyLineForm: LineCreate = {
  lineNumber: '',
  name: '',
  description: '',
  isp: ISP_OPTIONS[0],
  ipAddress: '',
  gatewayIp: '',
  portalUsername: '',
  portalPassword: '',
};

export const Lines: Component = () => {
  // Use TanStack Query for instant navigation with caching
  const query = useLinesQuery();

  // Mutations for CRUD operations
  const createMutation = useCreateLineMutation();
  const updateMutation = useUpdateLineMutation();
  const deleteMutation = useDeleteLineMutation();
  const toggleActiveMutation = useToggleLineActiveMutation();

  const [isFormOpen, setIsFormOpen] = createSignal(false);
  const [isDeleteOpen, setIsDeleteOpen] = createSignal(false);
  const [isViewOpen, setIsViewOpen] = createSignal(false);
  const [formData, setFormData] = createSignal<LineCreate>(emptyLineForm);
  const [isEditing, setIsEditing] = createSignal(false);
  const [editingLine, setEditingLine] = createSignal<Line | null>(null);
  const [viewingLine, setViewingLine] = createSignal<Line | null>(null);
  const [deletingLine, setDeletingLine] = createSignal<Line | null>(null);
  const [saving, setSaving] = createSignal(false);

  const openEditForm = (line: Line) => {
    setFormData({
      lineNumber: line.lineNumber,
      name: line.name,
      description: line.description ?? '',
      isp: line.isp ?? '',
      ipAddress: line.ipAddress ?? '',
      gatewayIp: line.gatewayIp ?? '',
      portalUsername: line.portalUsername ?? '',
      portalPassword: line.portalPassword ?? '',
    });
    setIsEditing(true);
    setEditingLine(line);
    setIsFormOpen(true);
  };

  const openViewModal = (line: Line) => {
    setViewingLine(line);
    setIsViewOpen(true);
  };

  const openDeleteModal = (line: Line) => {
    setDeletingLine(line);
    setIsDeleteOpen(true);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      if (isEditing() && editingLine()) {
        const data: LineUpdate = { ...formData() };
        // Don't send empty credentials - preserve existing values on the backend
        if (!data.portalUsername) delete data.portalUsername;
        if (!data.portalPassword) delete data.portalPassword;
        await updateMutation.mutateAsync({
          id: editingLine()!.id,
          data,
        });
        showToast({ title: 'Line updated', variant: 'success', duration: 3000 });
      } else {
        await createMutation.mutateAsync(formData());
        showToast({ title: 'Line created', variant: 'success', duration: 3000 });
      }
      setIsFormOpen(false);
    } catch (e) {
      showToast({ title: 'Failed to save line', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!deletingLine()) return;
    try {
      await deleteMutation.mutateAsync(deletingLine()!.id);
      setIsDeleteOpen(false);
      setDeletingLine(null);
      showToast({ title: 'Line deleted', variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to delete line', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  const updateField = (field: keyof LineCreate, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  const handleToggleActive = async (line: Line) => {
    try {
      await toggleActiveMutation.mutateAsync({
        id: line.id,
        isActive: !line.isActive,
      });
      showToast({ title: `Line ${!line.isActive ? 'activated' : 'deactivated'}`, variant: 'success', duration: 3000 });
    } catch (e) {
      showToast({ title: 'Failed to toggle line status', description: e instanceof Error ? e.message : 'An unexpected error occurred', variant: 'error', duration: 5000 });
    }
  };

  // Faceted filters
  const facetedFilters: FacetedFilterConfig<Line>[] = [
    {
      key: 'isp',
      label: 'ISP',
      accessor: (item) => item.isp ?? null,
    },
    {
      key: 'status',
      label: 'Status',
      accessor: (item) => item.isActive ? 'Active' : 'Inactive',
    },
  ];

  const columns: ColumnDef<Line>[] = [
    {
      key: 'id',
      header: 'ID',
      sortable: true,
      width: '5%',
      align: 'center',
    },
    {
      key: 'lineNumber',
      header: 'Line #',
      sortable: true,
      width: '12%',
      align: 'center',
    },
    {
      key: 'name',
      header: 'Name',
      sortable: true,
      width: '15%',
      align: 'center',
    },
    {
      key: 'isp',
      header: 'ISP',
      sortable: true,
      width: '8%',
      align: 'center',
    },
    {
      key: 'ipAddress',
      header: 'IP Address',
      sortable: true,
      width: '13%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs text-[#808080]">{row().ipAddress || '-'}</span>
      ),
    },
    {
      key: 'gatewayIp',
      header: 'Gateway',
      sortable: true,
      width: '13%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs text-[#808080]">{row().gatewayIp || '-'}</span>
      ),
    },
    {
      key: 'isActive',
      header: 'Active',
      sortable: true,
      width: '8%',
      align: 'center',
      cellRenderer: (row) => (
        <div class="flex items-center justify-center">
          <Switch.Root
            checked={row().isActive ?? true}
            onChange={() => handleToggleActive(row())}
            class="relative inline-flex h-4 w-7 items-center rounded-full bg-[#3c3c3c] transition-colors focus:outline-none focus:ring-2 focus:ring-[#3584e4] focus:ring-offset-2 focus:ring-offset-[#1e1e1e] data-[checked]:bg-[#3584e4]"
          >
            <Switch.Thumb class="h-3 w-3 transform rounded-full bg-white transition-transform data-[checked]:translate-x-3.5 data-[unchecked]:translate-x-0.5" />
          </Switch.Root>
        </div>
      ),
    },
    {
      key: 'description',
      header: 'Description',
      sortable: false,
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs truncate inline-block max-w-full" title={row().description || ''}>
          {row().description || '-'}
        </span>
      ),
    },
  ];

  const hoverActions: HoverAction<Line>[] = [
    {
      icon: (props) => <Edit2 {...props} />,
      label: 'Edit line',
      action: 'edit',
    },
  ];

  // Derive loading state for table
  const loadingState = () => {
    if (query.isPending && !query.data) return 'loading';
    return 'idle';
  };

  const data = () => query.data ?? [];

  return (
    <div class="flex flex-col h-full">
      <div class="flex flex-col h-full overflow-hidden">
        {/* Content Header with Actions */}
        <div class="px-3 pt-2 pb-3 flex-shrink-0">
          <div class="flex items-center justify-between">
            <div>
              <h1 class="text-[20px] font-semibold text-[#eeeeee]">Lines</h1>
              <p class="text-[13px] text-[#999999]">Manage internet line configurations and credentials</p>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="px-3 py-1.5 text-[11px] rounded-[8px] bg-[#3584e4] text-white hover:bg-[#4a9ff1]"
                onClick={() => {
                  setFormData(emptyLineForm);
                  setIsEditing(false);
                  setEditingLine(null);
                  setIsFormOpen(true);
                }}
              >
                + Add Line
              </button>
            </div>
          </div>
        </div>

        {/* Table container */}
        <div class="flex-1 overflow-hidden">
          <DesktopTable
            data={data()}
            columns={columns}
            hoverActions={hoverActions}
            loadingState={loadingState()}
            emptyMessage="No lines configured. Click '+ Add Line' to create one."
            inlineSearch={{ keys: ['name', 'lineNumber', 'ipAddress', 'isp', 'description'], placeholder: 'Search lines...' }}
            facetedFilters={facetedFilters}
            defaultPageSize={25}
            enablePagination={true}
            enableStatusBar={true}
            contextMenuItems={[
              { action: 'view', label: 'View Details', shortcut: 'Enter' },
              { action: 'edit', label: 'Edit', shortcut: 'E' },
              { action: 'copy', label: 'Copy Row', shortcut: 'Ctrl+C' },
              { type: 'separator' },
              { action: 'delete', label: 'Delete', shortcut: 'Del', destructive: true },
            ]}
            onRowAction={(action, row) => {
              const line = row as Line;
              switch (action) {
                case 'view':
                  openViewModal(line);
                  break;
                case 'edit':
                  openEditForm(line);
                  break;
                case 'copy':
                  navigator.clipboard.writeText(JSON.stringify(row, null, 2));
                  showToast({ title: 'Copied to clipboard', variant: 'default', duration: 2000 });
                  break;
                case 'delete':
                  openDeleteModal(line);
                  break;
              }
            }}
          />
        </div>
      </div>

      <ViewLineDialog
        open={isViewOpen()}
        line={viewingLine()}
        onOpenChange={setIsViewOpen}
        onEdit={openEditForm}
      />

      <AddLineDialog
        open={isFormOpen()}
        formData={formData()}
        isEditing={isEditing()}
        saving={saving()}
        onOpenChange={setIsFormOpen}
        onUpdateField={updateField}
        onSave={handleSave}
      />

      <DeleteLineDialog
        open={isDeleteOpen()}
        line={deletingLine()}
        onOpenChange={setIsDeleteOpen}
        onDelete={handleDelete}
      />
    </div>
  );
};
