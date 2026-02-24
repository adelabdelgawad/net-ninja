import { type Component, createMemo } from 'solid-js';
import { Badge } from '~/components/ui/badge';
import { DesktopTable, type ColumnDef, type StatusFilter, type FacetedFilterConfig } from '~/components/desktop-table';
import { useSpeedTestsQuery, useLinesQuery } from '~/api/queries';
import { showToast } from '~/components/ui/toast';
import type { SpeedTestResult } from '~/types';
import { formatShortDateTime } from '~/lib/date';

export const SpeedResults: Component = () => {
  // Use TanStack Query for instant navigation with caching
  const query = useSpeedTestsQuery({ page: 1, pageSize: 500 });
  const linesQuery = useLinesQuery();

  // Build lineId -> name lookup
  const lineNameMap = createMemo(() => {
    const map = new Map<number, string>();
    for (const line of linesQuery.data ?? []) {
      map.set(line.id, line.name);
    }
    return map;
  });

  const getSpeedVariant = (speed: number | null): 'success' | 'warning' | 'destructive' | 'secondary' => {
    if (speed === null || speed === 0) return 'destructive';
    if (speed >= 50) return 'success';
    if (speed >= 20) return 'warning';
    return 'destructive';
  };

  const getStatusVariant = (status: string | null): 'success' | 'warning' | 'destructive' | 'secondary' => {
    switch (status) {
      case 'success': return 'success';
      case 'failed': return 'destructive';
      default: return 'secondary';
    }
  };

  const getStatusLabel = (status: string | null): string => {
    switch (status) {
      case 'success': return 'Success';
      case 'failed': return 'Failed';
      default: return status ?? 'Unknown';
    }
  };

  // Status filters based on test result status
  const statusFilters: StatusFilter<SpeedTestResult>[] = [
    {
      key: 'success',
      label: 'Success',
      count: (data) => data.filter(r => r.status === 'success').length,
      filter: (item) => item.status === 'success',
    },
    {
      key: 'failed',
      label: 'Failed',
      count: (data) => data.filter(r => r.status === 'failed').length,
      filter: (item) => item.status === 'failed',
    },
  ];

  // Faceted filters for lines
  const facetedFilters: FacetedFilterConfig<SpeedTestResult>[] = [
    {
      key: 'lineName',
      label: 'Lines',
      accessor: (item) => item.lineId != null ? (lineNameMap().get(item.lineId) ?? String(item.lineId)) : null,
    },
  ];

  const columns: ColumnDef<SpeedTestResult>[] = [
    {
      key: 'id',
      header: 'ID',
      sortable: true,
      width: '5%',
      align: 'center',
    },
    {
      key: 'lineId',
      header: 'Line',
      sortable: true,
      width: '12%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{lineNameMap().get(row().lineId) ?? String(row().lineId)}</span>
      ),
    },
    {
      key: 'downloadSpeed',
      header: 'Download',
      sortable: true,
      width: '13%',
      align: 'center',
      cellRenderer: (row) => (
        <Badge variant={getSpeedVariant(row().downloadSpeed)}>
          {row().downloadSpeed != null ? `${row().downloadSpeed!.toFixed(2)} Mbps` : '-'}
        </Badge>
      ),
    },
    {
      key: 'uploadSpeed',
      header: 'Upload',
      sortable: true,
      width: '12%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">
          {row().uploadSpeed != null ? `${row().uploadSpeed!.toFixed(2)} Mbps` : '-'}
        </span>
      ),
    },
    {
      key: 'ping',
      header: 'Ping',
      sortable: true,
      width: '9%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">
          {row().ping != null ? `${row().ping!.toFixed(2)} ms` : '-'}
        </span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      sortable: true,
      width: '10%',
      align: 'center',
      cellRenderer: (row) => (
        <Badge variant={getStatusVariant(row().status)}>
          {getStatusLabel(row().status)}
        </Badge>
      ),
    },
    {
      key: 'publicIp',
      header: 'Public IP',
      sortable: true,
      width: '13%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs text-[#808080]">
          {row().publicIp || '-'}
        </span>
      ),
    },
    {
      key: 'createdAt',
      header: 'Tested',
      sortable: true,
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs text-[#808080]">
          {formatShortDateTime(row().createdAt)}
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
              <h1 class="text-[20px] font-semibold text-[#eeeeee]">Speed Test Results</h1>
              <p class="text-[13px] text-[#999999]">Network speed test history and performance metrics</p>
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
            emptyMessage="No speed test results found"
            statusFilters={statusFilters}
            facetedFilters={facetedFilters}
            defaultPageSize={25}
            defaultSort={{ key: 'id', direction: 'desc' }}
            enablePagination={true}
            enableStatusBar={true}
            contextMenuItems={[
              { action: 'view', label: 'View Details', shortcut: 'Enter' },
              { action: 'copy', label: 'Copy Row', shortcut: 'Ctrl+C' },
              { type: 'separator' },
              { action: 'delete', label: 'Delete', shortcut: 'Del', destructive: true },
            ]}
            onRowAction={(action, row) => {
              switch (action) {
                case 'view':
                  // TODO: implement view details
                  break;
                case 'copy':
                  navigator.clipboard.writeText(JSON.stringify(row, null, 2));
                  showToast({ title: 'Copied to clipboard', variant: 'default', duration: 2000 });
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
