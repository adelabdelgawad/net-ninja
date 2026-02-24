import { type Component, createMemo } from 'solid-js';
import { Badge } from '~/components/ui/badge';
import { DesktopTable, type ColumnDef, type StatusFilter, type FacetedFilterConfig } from '~/components/desktop-table';
import { useQuotaResultsQuery, useLinesQuery } from '~/api/queries';
import { showToast } from '~/components/ui/toast';
import type { QuotaResult } from '~/types';
import { formatShortDateTime } from '~/lib/date';

export const QuotaResults: Component = () => {
  // Use TanStack Query for instant navigation with caching
  const query = useQuotaResultsQuery({ page: 1, pageSize: 500 });
  const linesQuery = useLinesQuery();

  // Build lineId -> name lookup
  const lineNameMap = createMemo(() => {
    const map = new Map<number, string>();
    for (const line of linesQuery.data ?? []) {
      map.set(line.id, line.name);
    }
    return map;
  });

  const getUsageVariant = (percentage: number | null): 'success' | 'warning' | 'destructive' | 'secondary' => {
    if (percentage === null) return 'secondary';
    if (percentage >= 90) return 'destructive';
    if (percentage >= 70) return 'warning';
    return 'success';
  };

  // Status filters based on usage levels
  const statusFilters: StatusFilter<QuotaResult>[] = [
    {
      key: 'critical',
      label: 'Critical (>90%)',
      count: (data) => data.filter(r => r.quotaPercentage != null && r.quotaPercentage >= 90).length,
      filter: (item) => item.quotaPercentage != null && item.quotaPercentage >= 90,
    },
    {
      key: 'warning',
      label: 'Warning (70-90%)',
      count: (data) => data.filter(r => r.quotaPercentage != null && r.quotaPercentage >= 70 && r.quotaPercentage < 90).length,
      filter: (item) => item.quotaPercentage != null && item.quotaPercentage >= 70 && item.quotaPercentage < 90,
    },
    {
      key: 'healthy',
      label: 'Healthy (<70%)',
      count: (data) => data.filter(r => r.quotaPercentage != null && r.quotaPercentage < 70).length,
      filter: (item) => item.quotaPercentage != null && item.quotaPercentage < 70,
    },
    {
      key: 'no_data',
      label: 'No Data',
      count: (data) => data.filter(r => r.quotaPercentage == null).length,
      filter: (item) => item.quotaPercentage == null,
    },
  ];

  // Faceted filters for lines
  const facetedFilters: FacetedFilterConfig<QuotaResult>[] = [
    {
      key: 'lineName',
      label: 'Lines',
      accessor: (item) => item.lineId != null ? (lineNameMap().get(item.lineId) ?? String(item.lineId)) : null,
    },
  ];

  const columns: ColumnDef<QuotaResult>[] = [
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
      key: 'usedQuota',
      header: 'Used',
      sortable: true,
      width: '10%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{row().usedQuota != null ? `${row().usedQuota!.toFixed(2)} GB` : '-'}</span>
      ),
    },
    {
      key: 'quotaPercentage',
      header: 'Usage',
      sortable: true,
      width: '10%',
      align: 'center',
      cellRenderer: (row) => (
        <Badge variant={getUsageVariant(row().quotaPercentage)}>
          {row().quotaPercentage != null ? `${row().quotaPercentage!.toFixed(1)}%` : '-'}
        </Badge>
      ),
    },
    {
      key: 'remainingQuota',
      header: 'Remaining',
      sortable: true,
      width: '11%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{row().remainingQuota != null ? `${row().remainingQuota!.toFixed(2)} GB` : '-'}</span>
      ),
    },
    {
      key: 'balance',
      header: 'Balance',
      sortable: true,
      width: '10%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{row().balance != null ? `${row().balance} EGP` : '-'}</span>
      ),
    },
    {
      key: 'renewalDate',
      header: 'Renewal',
      sortable: true,
      width: '12%',
      align: 'center',
      cellRenderer: (row) => (
        <span class="text-xs">{row().renewalDate || '-'}</span>
      ),
    },
    {
      key: 'createdAt',
      header: 'Checked',
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
              <h1 class="text-[20px] font-semibold text-[#eeeeee]">Quota Check Results</h1>
              <p class="text-[13px] text-[#999999]">Data usage tracking and quota monitoring</p>
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
            emptyMessage="No quota results found"
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
