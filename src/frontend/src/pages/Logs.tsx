import { type Component, createMemo } from 'solid-js';
import { Badge } from '~/components/ui/badge';
import { DesktopTable, type ColumnDef, type StatusFilter, type FacetedFilterConfig } from '~/components/desktop-table';
import { useLogsQuery, useLinesQuery } from '~/api/queries';
import { showToast } from '~/components/ui/toast';
import type { Log } from '~/types';
import { formatDateTime } from '~/lib/date';

export const Logs: Component = () => {
  // Use TanStack Query for instant navigation with caching
  const query = useLogsQuery({ page: 1, pageSize: 500 });
  const linesQuery = useLinesQuery();

  // Build lineId -> name lookup
  const lineNameMap = createMemo(() => {
    const map = new Map<number, string>();
    for (const line of linesQuery.data ?? []) {
      map.set(line.id, line.name);
    }
    return map;
  });

  const getLevelVariant = (level: string | null): 'default' | 'success' | 'warning' | 'destructive' | 'secondary' => {
    switch (level?.toUpperCase()) {
      case 'ERROR': return 'destructive';
      case 'WARNING': return 'warning';
      case 'INFO': return 'success';
      case 'DEBUG': return 'secondary';
      default: return 'default';
    }
  };

  // Status filters for log levels
  const statusFilters: StatusFilter<Log>[] = [
    {
      key: 'error',
      label: 'Error',
      count: (data) => data.filter(l => l.level?.toUpperCase() === 'ERROR').length,
      filter: (item) => item.level?.toUpperCase() === 'ERROR',
    },
    {
      key: 'warning',
      label: 'Warning',
      count: (data) => data.filter(l => l.level?.toUpperCase() === 'WARNING').length,
      filter: (item) => item.level?.toUpperCase() === 'WARNING',
    },
    {
      key: 'info',
      label: 'Info',
      count: (data) => data.filter(l => l.level?.toUpperCase() === 'INFO').length,
      filter: (item) => item.level?.toUpperCase() === 'INFO',
    },
    {
      key: 'debug',
      label: 'Debug',
      count: (data) => data.filter(l => l.level?.toUpperCase() === 'DEBUG').length,
      filter: (item) => item.level?.toUpperCase() === 'DEBUG',
    },
  ];

  // Faceted filters
  const facetedFilters: FacetedFilterConfig<Log>[] = [
    {
      key: 'lineName',
      label: 'Lines',
      accessor: (item) => item.lineId != null ? (lineNameMap().get(item.lineId) ?? String(item.lineId)) : null,
    },
    {
      key: 'function',
      label: 'Function',
      accessor: (item) => item.function ?? null,
    },
  ];

  const columns: ColumnDef<Log>[] = [
    {
      key: 'id',
      header: 'ID',
      sortable: true,
      width: '5%',
      align: 'center',
    },
    {
      key: 'timestamp',
      header: 'Time',
      sortable: true,
      width: '15%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{formatDateTime(row().timestamp, {
          month: 'short',
          day: 'numeric',
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
        })}</span>
      ),
    },
    {
      key: 'level',
      header: 'Level',
      sortable: true,
      width: '8%',
      align: 'center',
      cellRenderer: (row) => (
        <Badge variant={getLevelVariant(row().level)}>
          {row().level || 'UNKNOWN'}
        </Badge>
      ),
    },
    {
      key: 'lineId',
      header: 'Line',
      sortable: true,
      width: '10%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{row().lineId != null ? (lineNameMap().get(row().lineId!) ?? String(row().lineId)) : '-'}</span>
      ),
    },
    {
      key: 'function',
      header: 'Function',
      sortable: true,
      width: '14%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{row().function || '-'}</span>
      ),
    },
    {
      key: 'processId',
      header: 'Process',
      sortable: true,
      width: '10%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs text-[#808080]">
          {row().processId?.slice(0, 8) ?? '-'}
        </span>
      ),
    },
    {
      key: 'message',
      header: 'Message',
      sortable: false,
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs truncate block" title={row().message || ''}>
          {row().message || '-'}
        </span>
      ),
    },
  ];

  // Derive loading state for table
  const loadingState = () => {
    if (query.isPending && !query.data) return 'loading';
    return 'idle';
  };

  const data = () => query.data?.items ?? [];

  return (
    <div class="flex flex-col h-full">
      <div class="flex flex-col h-full overflow-hidden">
        {/* Content Header with Actions */}
        <div class="px-3 pt-2 pb-3 flex-shrink-0">
          <div class="flex items-center justify-between">
            <div>
              <h1 class="text-[20px] font-semibold text-[#eeeeee]">Logs</h1>
              <p class="text-[13px] text-[#999999]">View application logs and debugging information</p>
            </div>
          </div>
        </div>

        {/* Table wrapper */}
        <div class="flex-1 overflow-hidden">
          <DesktopTable
            data={data()}
            columns={columns}
            loadingState={loadingState()}
            error={query.error?.message}
            emptyMessage="No logs found"
            statusFilters={statusFilters}
            facetedFilters={facetedFilters}
            defaultPageSize={50}
            enablePagination={true}
            enableStatusBar={true}
            contextMenuItems={[
              { action: 'copy', label: 'Copy Row', shortcut: 'Ctrl+C' },
              { type: 'separator' },
              { action: 'view', label: 'View Details', shortcut: 'Enter' },
              { type: 'separator' },
              { action: 'delete', label: 'Delete', shortcut: 'Del', destructive: true },
            ]}
            onRowAction={(action, row) => {
              switch (action) {
                case 'copy':
                  navigator.clipboard.writeText(JSON.stringify(row, null, 2));
                  showToast({ title: 'Copied to clipboard', variant: 'default', duration: 2000 });
                  break;
                case 'view':
                  // TODO: implement view details
                  break;
                case 'delete':
                  // TODO: implement delete
                  break;
              }
            }}
          />
        </div>
      </div>
    </div>
  );
};
