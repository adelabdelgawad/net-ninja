/**
 * DesktopTable - Desktop-native data table with VS Code-inspired styling
 *
 * Features:
 * - Status filter tabs with counts
 * - Search bar with field selector
 * - Sortable columns
 * - Pagination with rows per page selector
 * - Context menu with keyboard shortcuts
 * - Full keyboard navigation (arrows, enter, page up/down)
 * - Status bar with keyboard hints
 */

import { For, Show, createSignal, createMemo, createEffect, onMount, onCleanup, type JSX } from 'solid-js';

// ============================================================================
// Types
// ============================================================================

export interface TableRow {
  id: string | number;
}

export interface ColumnDef<T extends TableRow> {
  key: keyof T | string;
  header: string;
  align?: 'left' | 'center' | 'right';
  width?: string;
  sortable?: boolean;
  cellRenderer?: (row: () => T) => JSX.Element;
}

export interface ContextMenuItem {
  action: string;
  label: string;
  shortcut?: string;
  destructive?: boolean;
}

export interface ContextMenuSeparator {
  type: 'separator';
}

export type ContextMenuEntry = ContextMenuItem | ContextMenuSeparator;

export interface StatusFilter<T> {
  key: string;
  label: string;
  count: (data: T[]) => number;
  filter: (item: T) => boolean;
}

export interface SearchField {
  key: string;
  label: string;
}

export interface HoverAction<T> {
  icon: (props: { class?: string }) => JSX.Element;
  label: string;
  action: string;
  variant?: 'default' | 'destructive';
  show?: (row: T) => boolean;
  disabled?: (row: T) => boolean;
}

export interface FacetedFilterConfig<T> {
  /** Unique key for this filter */
  key: string;
  /** Display label (e.g. "Level", "Function") */
  label: string;
  /** Extract the value to filter on from a row */
  accessor: (item: T) => string | null | undefined;
  /** Optional icon before label */
  icon?: string;
}

interface DesktopTableProps<T extends TableRow> {
  data: T[];
  columns: ColumnDef<T>[];
  loadingState?: 'idle' | 'loading' | 'error';
  error?: string;
  emptyMessage?: string;

  // Features
  enableSearch?: boolean;
  enablePagination?: boolean;
  enableStatusBar?: boolean;

  // Configuration
  statusFilters?: StatusFilter<T>[];
  facetedFilters?: FacetedFilterConfig<T>[];
  searchFields?: SearchField[];
  /** Inline search in control bar. Keys are fields to search across, placeholder is the input placeholder. */
  inlineSearch?: { keys: string[]; placeholder?: string };
  defaultPageSize?: number;
  pageSizeOptions?: number[];
  defaultSort?: { key: string; direction: 'asc' | 'desc' };

  // Context menu
  contextMenuItems?: ContextMenuEntry[];
  onRowAction?: (action: string, row: T) => void;

  // Header actions (buttons shown in top-right of control bar)
  headerActions?: JSX.Element;

  /** Actions that appear on row hover, rendered as icon buttons at row end */
  hoverActions?: HoverAction<T>[];
}

// ============================================================================
// Styles (CSS-in-JS matching reference HTML)
// ============================================================================

const styles = {
  // Root container
  root: `
    flex flex-col h-full overflow-hidden
    bg-[#1e1e1e] text-[#cccccc]
    font-['IBM_Plex_Sans',-apple-system,BlinkMacSystemFont,sans-serif]
    text-[13px] leading-normal select-none
  `,

  // Control head bar
  controlBar: `
    flex justify-between items-center px-3 py-1.5
    bg-[#252526] border-b border-[#3c3c3c]
    flex-shrink-0
  `,
  controlLeft: 'flex items-center gap-2',
  controlRight: 'flex items-center gap-1.5',

  // Status filter
  statusFilter: `
    flex bg-[#1e1e1e] border border-[#3c3c3c] rounded-[6px] overflow-hidden
  `,
  statusFilterBtn: `
    px-3 py-1.5 bg-transparent border-none border-r border-[#3c3c3c]
    text-[#808080] text-[11px] cursor-pointer
    flex items-center gap-1.5
    hover:bg-white/[0.04] hover:text-[#cccccc]
    last:border-r-0
  `,
  statusFilterBtnActive: `
    bg-[#3584e4] text-white
    hover:bg-[#3584e4] hover:text-white
  `,
  filterCount: `
    font-mono text-[10px] px-1.5 py-px
    bg-white/10 rounded-sm min-w-[18px] text-center
  `,

  // Control buttons
  controlBtn: `
    px-3 py-1.5 bg-[#2d2d2d] border border-[#3c3c3c]
    text-[#cccccc] text-[11px] cursor-pointer rounded-[6px]
    hover:bg-[#3c3c3c] hover:border-[#808080]
    active:bg-[#094771]
    disabled:text-[#5a5a5a] disabled:cursor-not-allowed disabled:opacity-50
  `,
  controlBtnPrimary: `
    bg-[#3584e4] border-[#3584e4] text-white
    hover:bg-[#4a9ff1] hover:border-[#4a9ff1]
  `,
  controlSeparator: 'w-px h-5 bg-[#3c3c3c]',

  // Search bar
  searchBar: `
    flex items-center px-3 py-1.5
    bg-[#1e1e1e] border-b border-[#3c3c3c]
    flex-shrink-0 gap-2
  `,
  searchLabel: 'text-[11px] text-[#808080]',
  searchSelect: `
    px-2 py-1 bg-[#2d2d2d] border border-[#3c3c3c]
    text-[#cccccc] text-[11px] rounded-[3px] cursor-pointer
    focus:outline-none focus:border-[#3584e4]
  `,
  searchInput: `
    px-2 py-1 bg-[#2d2d2d] border border-[#3c3c3c]
    text-[#cccccc] text-[11px] rounded-[3px] w-[200px]
    focus:outline-none focus:border-[#3584e4]
    placeholder:text-[#808080]
  `,

  // Table container
  tableContainer: `
    flex-1 flex flex-col overflow-hidden outline-none
  `,

  // Header
  tableHeader: `
    flex bg-[#222222] border-b border-[#2a2a2a] flex-shrink-0 w-full
  `,
  headerCell: `
    px-3 py-2 font-medium text-[#808080] text-[11px]
    flex items-center justify-between cursor-pointer flex-shrink-0
    hover:bg-white/[0.04]
  `,
  headerCellNoSort: 'cursor-default hover:bg-transparent',
  headerContent: 'flex items-center gap-1.5',
  sortIndicator: 'text-[8px] text-[#808080] opacity-70',
  sortIndicatorActive: 'opacity-100 text-[#3584e4]',

  // Body
  tableBody: `
    flex-1 overflow-y-auto overflow-x-hidden
    scrollbar-thin scrollbar-track-[#1e1e1e] scrollbar-thumb-[#3c3c3c]
  `,

  // Row
  tableRow: `
    flex border-b border-[#2a2a2a] cursor-default w-full
    hover:bg-[#2f2f2f] group relative
  `,
  tableRowEven: 'bg-[#252525]',
  tableRowOdd: 'bg-[#282828]',
  tableRowSelected: 'bg-[#3584e4]/20',
  tableRowFocused: 'outline outline-1 outline-[#3584e4] -outline-offset-1',

  // Cell
  tableCell: `
    px-3 py-2.5 text-[13px] leading-5
    overflow-hidden
    text-ellipsis whitespace-nowrap flex-shrink-0
    flex items-center
  `,
  tableCellAlignRight: 'justify-end text-right',
  tableCellAlignCenter: 'justify-center text-center',

  // Pagination (DataGrip-style)
  paginationBar: `
    flex items-center justify-between py-1 px-2
    bg-[#252526] border-t border-[#3c3c3c] flex-shrink-0
  `,
  paginationLeft: 'flex items-center gap-1.5',
  paginationRight: 'flex items-center gap-3',
  paginationInfo: 'text-[11px] text-[#808080] tabular-nums h-[20px] inline-flex items-center',
  paginationSeparator: 'w-px h-3.5 bg-[#3c3c3c]',
  paginationLabel: 'text-[11px] text-[#808080] h-[20px] inline-flex items-center',
  paginationSelect: `
    px-1.5 py-0.5 bg-[#1e1e1e] border border-[#3c3c3c]
    text-[#cccccc] text-[11px] leading-none rounded-[2px] cursor-pointer
    focus:outline-none focus:border-[#3584e4]
    hover:border-[#808080]
  `,
  paginationNav: 'flex items-center gap-0.5',
  paginationNavBtn: `
    w-[22px] h-[20px] flex items-center justify-center
    bg-transparent border border-transparent
    text-[#808080] rounded-[2px] cursor-pointer
    hover:bg-[#3c3c3c] hover:text-[#cccccc]
    disabled:text-[#3c3c3c] disabled:cursor-default disabled:hover:bg-transparent
  `,
  paginationPageInput: `
    w-[42px] h-[20px] px-1.5 bg-[#1e1e1e] border border-[#3c3c3c]
    text-[#cccccc] text-[11px] text-center rounded-[2px] tabular-nums
    focus:outline-none focus:border-[#3584e4]
    hover:border-[#808080]
  `,
  paginationPageLabel: 'text-[11px] text-[#808080] tabular-nums h-[20px] inline-flex items-center',

  // Status bar
  statusBar: `
    flex justify-between px-3 py-1
    bg-[#3584e4] text-white text-[11px] flex-shrink-0
  `,
  statusBarSection: 'flex gap-5',
  kbd: `
    font-mono text-[10px] px-1 py-px
    bg-white/15 rounded-sm mx-0.5
  `,

  // Context menu
  contextMenu: `
    fixed bg-[#252526] border border-[#3c3c3c] rounded-[8px]
    shadow-[0_4px_12px_rgba(0,0,0,0.5)] min-w-[200px] py-1 z-50
  `,
  contextMenuHeader: `
    px-3 py-1.5 text-[10px] text-[#808080]
    border-b border-[#3c3c3c] mb-1
    uppercase tracking-wider
  `,
  contextMenuItem: `
    px-3 py-1.5 text-xs text-[#cccccc] cursor-pointer
    flex justify-between items-center
    hover:bg-[#3584e4]/20
  `,
  contextMenuItemDanger: 'text-[#f14c4c]',
  contextMenuShortcut: 'text-[11px] text-[#808080] font-mono',
  contextMenuSeparator: 'h-px bg-[#3c3c3c] my-1',

  // Empty state
  emptyState: `
    flex-1 flex items-center justify-center
    text-[#808080] text-sm
  `,

  // Loading state
  loadingState: `
    flex-1 flex items-center justify-center
  `,
  spinner: `
    w-8 h-8 border-4 border-[#3584e4] border-t-transparent
    rounded-full animate-spin
  `,

  // Error state
  errorState: `
    flex-1 flex items-center justify-center text-[#f14c4c] text-sm
  `,
};

// Utility to combine class names
const cx = (...classes: (string | false | undefined)[]) => classes.filter(Boolean).join(' ');

// ============================================================================
// Component
// ============================================================================

export function DesktopTable<T extends TableRow>(props: DesktopTableProps<T>) {
  // State
  const [focusedId, setFocusedId] = createSignal<string | number | null>(null);
  const [sortConfig, setSortConfig] = createSignal<{ key: string; direction: 'asc' | 'desc' }>(props.defaultSort ?? { key: 'id', direction: 'asc' });
  const [currentFilter, setCurrentFilter] = createSignal<string>('all');
  const [searchField, setSearchField] = createSignal<string>(props.searchFields?.[0]?.key ?? '');
  const [searchQuery, setSearchQuery] = createSignal<string>('');
  const [currentPage, setCurrentPage] = createSignal<number>(1);
  const [pageSize, setPageSize] = createSignal<number>(props.defaultPageSize ?? 25);
  const [pageInputValue, setPageInputValue] = createSignal<string>('1');
  const [pageInputEditing, setPageInputEditing] = createSignal(false);
  const [contextMenu, setContextMenu] = createSignal<{ x: number; y: number; row: T } | null>(null);
  const [facetedSelections, setFacetedSelections] = createSignal<Record<string, Set<string>>>({});
  const [openFacetedKey, setOpenFacetedKey] = createSignal<string | null>(null);
  const [facetedSearch, setFacetedSearch] = createSignal<string>('');
  const [inlineSearchQuery, setInlineSearchQuery] = createSignal<string>('');

  let tableContainerRef: HTMLDivElement | undefined;

  // Defaults
  const enableSearch = () => props.enableSearch ?? true;
  const enablePagination = () => props.enablePagination ?? true;
  const enableStatusBar = () => props.enableStatusBar ?? true;
  const pageSizeOptions = () => props.pageSizeOptions ?? [10, 25, 50, 100];

  // Filtered data
  const filteredData = createMemo(() => {
    let data = props.data;

    // Apply status filter
    if (props.statusFilters && currentFilter() !== 'all') {
      const filter = props.statusFilters.find(f => f.key === currentFilter());
      if (filter) {
        data = data.filter(filter.filter);
      }
    }

    // Apply faceted filters
    if (props.facetedFilters) {
      const sel = facetedSelections();
      for (const fc of props.facetedFilters) {
        const selected = sel[fc.key];
        if (selected && selected.size > 0) {
          data = data.filter(item => {
            const val = fc.accessor(item) ?? '(empty)';
            return selected.has(val);
          });
        }
      }
    }

    // Apply inline search (multi-key)
    if (props.inlineSearch && inlineSearchQuery()) {
      const q = inlineSearchQuery().toLowerCase();
      const keys = props.inlineSearch.keys;
      data = data.filter(item => {
        const rec = item as Record<string, unknown>;
        return keys.some(k => String(rec[k] ?? '').toLowerCase().includes(q));
      });
    }

    // Apply search
    if (searchQuery() && searchField()) {
      const query = searchQuery().toLowerCase();
      data = data.filter(item => {
        const value = String((item as Record<string, unknown>)[searchField()] ?? '').toLowerCase();
        return value.includes(query);
      });
    }

    return data;
  });

  // Helper to compare values for sorting
  const compareValues = (aVal: unknown, bVal: unknown): number => {
    // Handle null/undefined
    if (aVal === bVal) return 0;
    if (aVal === null || aVal === undefined) return 1;
    if (bVal === null || bVal === undefined) return -1;

    // Both are numbers
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      return aVal - bVal;
    }

    // Try to parse as numbers (handles numeric strings)
    const aNum = typeof aVal === 'string' ? parseFloat(aVal) : NaN;
    const bNum = typeof bVal === 'string' ? parseFloat(bVal) : NaN;
    if (!isNaN(aNum) && !isNaN(bNum)) {
      return aNum - bNum;
    }

    // Try to parse as dates (ISO format or common date strings)
    const aStr = String(aVal);
    const bStr = String(bVal);
    const aDate = Date.parse(aStr);
    const bDate = Date.parse(bStr);
    if (!isNaN(aDate) && !isNaN(bDate)) {
      return aDate - bDate;
    }

    // Fallback to string comparison (case-insensitive)
    return aStr.toLowerCase().localeCompare(bStr.toLowerCase());
  };

  // Sorted data
  const sortedData = createMemo(() => {
    const data = [...filteredData()];
    const { key, direction } = sortConfig();

    return data.sort((a, b) => {
      const aVal = (a as Record<string, unknown>)[key];
      const bVal = (b as Record<string, unknown>)[key];
      const comparison = compareValues(aVal, bVal);
      return direction === 'asc' ? comparison : -comparison;
    });
  });

  // Paginated data
  const paginatedData = createMemo(() => {
    if (!enablePagination()) return sortedData();
    const start = (currentPage() - 1) * pageSize();
    return sortedData().slice(start, start + pageSize());
  });

  // Total pages
  const totalPages = createMemo(() => {
    if (!enablePagination()) return 1;
    return Math.ceil(sortedData().length / pageSize()) || 1;
  });

  // Status filter counts
  const filterCounts = createMemo(() => {
    if (!props.statusFilters) return {};
    const counts: Record<string, number> = { all: props.data.length };
    for (const filter of props.statusFilters) {
      counts[filter.key] = filter.count(props.data);
    }
    return counts;
  });

  // Faceted filter options (unique values + counts per filter)
  const facetedOptions = createMemo(() => {
    if (!props.facetedFilters) return {};
    const result: Record<string, { value: string; count: number }[]> = {};
    for (const fc of props.facetedFilters) {
      const counts = new Map<string, number>();
      for (const item of props.data) {
        const val = fc.accessor(item);
        const key = val ?? '(empty)';
        counts.set(key, (counts.get(key) ?? 0) + 1);
      }
      result[fc.key] = Array.from(counts.entries())
        .map(([value, count]) => ({ value, count }))
        .sort((a, b) => b.count - a.count);
    }
    return result;
  });

  // Check if any faceted filter is active
  const hasFacetedFilters = createMemo(() => {
    const sel = facetedSelections();
    return Object.values(sel).some(s => s.size > 0);
  });

  // Toggle a value in a faceted filter
  const toggleFacetedValue = (filterKey: string, value: string) => {
    setFacetedSelections(prev => {
      const next = { ...prev };
      const set = new Set(prev[filterKey] ?? []);
      if (set.has(value)) set.delete(value);
      else set.add(value);
      next[filterKey] = set;
      return next;
    });
  };

  const resetAllFacetedFilters = () => {
    setFacetedSelections({});
  };

  // Reset page when filter/search changes
  createEffect(() => {
    currentFilter();
    searchQuery();
    inlineSearchQuery();
    facetedSelections();
    setCurrentPage(1);
  });

  // Handlers
  const handleSort = (key: string) => {
    setSortConfig(prev => ({
      key,
      direction: prev.key === key && prev.direction === 'asc' ? 'desc' : 'asc',
    }));
  };

  const handleRowClick = (row: T) => {
    setFocusedId(row.id);
  };

  const handleRowDblClick = (row: T) => {
    props.onRowAction?.('edit', row);
  };

  const handleContextMenu = (e: MouseEvent, row: T) => {
    e.preventDefault();
    setFocusedId(row.id);
    setContextMenu({ x: e.clientX, y: e.clientY, row });
  };

  const closeContextMenu = () => setContextMenu(null);

  const handleContextAction = (action: string) => {
    const menu = contextMenu();
    if (menu) {
      props.onRowAction?.(action, menu.row);
      closeContextMenu();
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    const data = paginatedData();
    const currentIndex = data.findIndex(r => r.id === focusedId());

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        if (data.length === 0) break;
        if (currentIndex < data.length - 1) {
          setFocusedId(data[currentIndex + 1].id);
          scrollToFocused();
        } else if (currentIndex === -1 && data.length > 0) {
          setFocusedId(data[0].id);
          scrollToFocused();
        }
        break;

      case 'ArrowUp':
        e.preventDefault();
        if (currentIndex > 0) {
          setFocusedId(data[currentIndex - 1].id);
          scrollToFocused();
        }
        break;

      case 'Enter':
        e.preventDefault();
        if (focusedId() !== null) {
          const row = data.find(r => r.id === focusedId());
          if (row) {
            props.onRowAction?.('edit', row);
          }
        }
        break;

      case 'Delete':
        e.preventDefault();
        if (focusedId() !== null) {
          const row = data.find(r => r.id === focusedId());
          if (row) {
            props.onRowAction?.('delete', row);
          }
        }
        break;

      case 'Escape':
        closeContextMenu();
        break;

      case 'Home':
        e.preventDefault();
        if (data.length > 0) {
          setFocusedId(data[0].id);
          scrollToFocused();
        }
        break;

      case 'End':
        e.preventDefault();
        if (data.length > 0) {
          setFocusedId(data[data.length - 1].id);
          scrollToFocused();
        }
        break;

      case 'PageDown':
        e.preventDefault();
        if (enablePagination() && currentPage() < totalPages()) {
          setCurrentPage(p => p + 1);
        }
        break;

      case 'PageUp':
        e.preventDefault();
        if (enablePagination() && currentPage() > 1) {
          setCurrentPage(p => p - 1);
        }
        break;
    }
  };

  const scrollToFocused = () => {
    const el = document.querySelector(`[data-row-id="${focusedId()}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  };

  const goToPage = (action: 'first' | 'prev' | 'next' | 'last') => {
    switch (action) {
      case 'first': setCurrentPage(1); break;
      case 'prev': setCurrentPage(p => Math.max(1, p - 1)); break;
      case 'next': setCurrentPage(p => Math.min(totalPages(), p + 1)); break;
      case 'last': setCurrentPage(totalPages()); break;
    }
  };

  const formatNumber = (n: number) => n.toLocaleString();

  const commitPageInput = () => {
    const val = parseInt(pageInputValue(), 10);
    if (!isNaN(val) && val >= 1 && val <= totalPages()) {
      setCurrentPage(val);
    }
    setPageInputValue(String(currentPage()));
    setPageInputEditing(false);
  };

  // Keep page input in sync when page changes externally
  createEffect(() => {
    if (!pageInputEditing()) {
      setPageInputValue(String(currentPage()));
    }
  });

  const getCellValue = (row: T, column: ColumnDef<T>) => {
    if (column.cellRenderer) {
      return column.cellRenderer(() => row);
    }
    const value = (row as Record<string, unknown>)[column.key as string];
    if (value === null || value === undefined) return '-';
    return String(value);
  };

  // Click outside to close context menu and faceted dropdowns
  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const menu = document.querySelector('[data-context-menu="true"]');
      if (menu && !menu.contains(e.target as Node)) {
        closeContextMenu();
      }
      const faceted = document.querySelector('[data-faceted-dropdown="true"]');
      if (faceted && !faceted.contains(e.target as Node)) {
        const trigger = (e.target as HTMLElement).closest('[data-faceted-trigger]');
        if (!trigger) {
          setOpenFacetedKey(null);
          setFacetedSearch('');
        }
      }
    };
    document.addEventListener('click', handleClickOutside);
    onCleanup(() => document.removeEventListener('click', handleClickOutside));
  });

  // Focus table on mount
  onMount(() => {
    tableContainerRef?.focus();
  });

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div class={styles.root}>
      {/* Control Head Bar */}
      <Show when={(props.statusFilters && props.statusFilters.length > 0) || (props.facetedFilters && props.facetedFilters.length > 0) || props.inlineSearch}>
        <div class={styles.controlBar}>
          <div class={styles.controlLeft}>
            {/* Inline search input */}
            <Show when={props.inlineSearch}>
              <input
                type="text"
                class="h-[28px] px-3 w-[180px] bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[11px] rounded-[3px] focus:outline-none focus:border-[#3584e4] placeholder:text-[#808080]"
                placeholder={props.inlineSearch?.placeholder ?? 'Search...'}
                value={inlineSearchQuery()}
                onInput={(e) => setInlineSearchQuery(e.currentTarget.value)}
              />
            </Show>

            {/* Status filter tabs */}
            <Show when={props.statusFilters && props.statusFilters.length > 0}>
              <div class={styles.statusFilter}>
                <button
                  type="button"
                  class={cx(styles.statusFilterBtn, currentFilter() === 'all' && styles.statusFilterBtnActive)}
                  onClick={() => setCurrentFilter('all')}
                >
                  All <span class={styles.filterCount}>{filterCounts().all ?? 0}</span>
                </button>
                <For each={props.statusFilters}>
                  {(filter) => (
                    <button
                      type="button"
                      class={cx(styles.statusFilterBtn, currentFilter() === filter.key && styles.statusFilterBtnActive)}
                      onClick={() => setCurrentFilter(filter.key)}
                    >
                      {filter.label} <span class={styles.filterCount}>{filterCounts()[filter.key] ?? 0}</span>
                    </button>
                  )}
                </For>
              </div>
            </Show>

            {/* Faceted filter dropdowns */}
            <Show when={props.facetedFilters && props.facetedFilters.length > 0}>
              <div class="w-px h-5 bg-[#3c3c3c]" />
              <For each={props.facetedFilters}>
                {(fc) => {
                  const selected = () => facetedSelections()[fc.key] ?? new Set<string>();
                  const isOpen = () => openFacetedKey() === fc.key;
                  const options = () => facetedOptions()[fc.key] ?? [];
                  const filteredOpts = () => {
                    const q = facetedSearch().toLowerCase();
                    if (!q) return options();
                    return options().filter(o => o.value.toLowerCase().includes(q));
                  };
                  const allVisibleSelected = () => {
                    const opts = filteredOpts();
                    const sel = selected();
                    return opts.length > 0 && opts.every(o => sel.has(o.value));
                  };
                  const toggleAll = () => {
                    const opts = filteredOpts();
                    if (allVisibleSelected()) {
                      setFacetedSelections(prev => {
                        const next = { ...prev };
                        const set = new Set(prev[fc.key] ?? []);
                        for (const o of opts) set.delete(o.value);
                        next[fc.key] = set;
                        return next;
                      });
                    } else {
                      setFacetedSelections(prev => {
                        const next = { ...prev };
                        const set = new Set(prev[fc.key] ?? []);
                        for (const o of opts) set.add(o.value);
                        next[fc.key] = set;
                        return next;
                      });
                    }
                  };

                  return (
                    <div class="relative">
                      <button
                        type="button"
                        data-faceted-trigger
                        class={cx(
                          'h-[26px] px-2 text-[11px] rounded-[3px] border bg-[#2d2d2d] text-[#cccccc] cursor-pointer flex items-center gap-1.5',
                          'hover:bg-[#3c3c3c] hover:border-[#808080]',
                          isOpen()
                            ? 'border-[#3584e4] bg-[#3584e4]/20'
                            : selected().size > 0
                              ? 'border-[#3584e4]/60 bg-[#3584e4]/15'
                              : 'border-[#3c3c3c]'
                        )}
                        onClick={() => {
                          if (isOpen()) {
                            setOpenFacetedKey(null);
                            setFacetedSearch('');
                          } else {
                            setOpenFacetedKey(fc.key);
                            setFacetedSearch('');
                          }
                        }}
                      >
                        <span class="text-[#808080] text-[10px]">
                          <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M1 3h14v1H1V3zm2 4h10v1H3V7zm2 4h6v1H5v-1z"/></svg>
                        </span>
                        <span>{fc.label}</span>
                        <Show when={selected().size > 0}>
                          <span class="font-mono text-[9px] px-1 py-px bg-[#3584e4] text-white rounded-[2px] min-w-[14px] text-center leading-tight">
                            {selected().size}
                          </span>
                        </Show>
                        <span class={cx(
                          'text-[8px] text-[#808080] ml-0.5 transition-transform',
                          isOpen() && 'rotate-180'
                        )}>
                          ▾
                        </span>
                      </button>

                      {/* Dropdown panel */}
                      <Show when={isOpen()}>
                        <div
                          data-faceted-dropdown="true"
                          class="absolute top-full left-0 mt-1 z-50 bg-[#1e1e1e] border border-[#3c3c3c] rounded-[3px] shadow-[0_4px_16px_rgba(0,0,0,0.6)] w-[220px]"
                        >
                          {/* Header */}
                          <div class="flex items-center justify-between px-2.5 py-1.5 bg-[#252526] border-b border-[#3c3c3c] rounded-t-[3px]">
                            <span class="text-[10px] text-[#808080] uppercase tracking-wider font-medium">Filter by {fc.label}</span>
                            <button
                              type="button"
                              class="text-[#808080] hover:text-[#cccccc] text-[11px] leading-none p-0.5"
                              onClick={() => { setOpenFacetedKey(null); setFacetedSearch(''); }}
                            >
                              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
                            </button>
                          </div>

                          {/* Search */}
                          <div class="px-2 py-1.5 border-b border-[#2a2a2a]">
                            <div class="relative">
                              <span class="absolute left-2 top-1/2 -translate-y-1/2 text-[#808080]">
                                <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"/></svg>
                              </span>
                              <input
                                type="text"
                                class="w-full pl-6 pr-2 py-1 bg-[#2d2d2d] border border-[#3c3c3c] text-[#cccccc] text-[11px] rounded-[3px] focus:outline-none focus:border-[#3584e4] placeholder:text-[#555]"
                                placeholder={`Search ${fc.label.toLowerCase()}...`}
                                value={facetedSearch()}
                                onInput={(e) => setFacetedSearch(e.currentTarget.value)}
                                autofocus
                              />
                            </div>
                          </div>

                          {/* Select all row */}
                          <div
                            class="flex items-center justify-between px-2.5 py-1 cursor-pointer hover:bg-white/[0.04] border-b border-[#2a2a2a]"
                            onClick={toggleAll}
                          >
                            <div class="flex items-center gap-2">
                              <div class={cx(
                                'w-[13px] h-[13px] border rounded-[2px] flex items-center justify-center',
                                allVisibleSelected()
                                  ? 'bg-[#3584e4] border-[#3584e4]'
                                  : selected().size > 0
                                    ? 'border-[#3584e4] bg-transparent'
                                    : 'border-[#555] bg-transparent'
                              )}>
                                <Show when={allVisibleSelected()}>
                                  <svg width="9" height="9" viewBox="0 0 16 16" fill="white"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0z"/></svg>
                                </Show>
                                <Show when={!allVisibleSelected() && selected().size > 0}>
                                  <svg width="9" height="9" viewBox="0 0 16 16" fill="#3584e4"><rect x="3" y="7" width="10" height="2" rx="0.5"/></svg>
                                </Show>
                              </div>
                              <span class="text-[11px] text-[#808080]">Select all</span>
                            </div>
                            <span class="font-mono text-[10px] text-[#555]">{filteredOpts().length}</span>
                          </div>

                          {/* Options list */}
                          <div class="max-h-[180px] overflow-y-auto scrollbar-thin scrollbar-track-[#1e1e1e] scrollbar-thumb-[#3c3c3c] py-0.5">
                            <For each={filteredOpts()}>
                              {(opt) => {
                                const isChecked = () => selected().has(opt.value);
                                return (
                                  <div
                                    class={cx(
                                      'flex items-center justify-between px-2.5 py-[5px] cursor-pointer text-[11px]',
                                      isChecked()
                                        ? 'bg-[#3584e4]/20 text-[#cccccc] hover:bg-[#3584e4]/30'
                                        : 'text-[#cccccc] hover:bg-white/[0.04]'
                                    )}
                                    onClick={() => toggleFacetedValue(fc.key, opt.value)}
                                  >
                                    <div class="flex items-center gap-2">
                                      <div class={cx(
                                        'w-[13px] h-[13px] border rounded-[2px] flex items-center justify-center transition-colors',
                                        isChecked()
                                          ? 'bg-[#3584e4] border-[#3584e4]'
                                          : 'border-[#555] bg-transparent hover:border-[#808080]'
                                      )}>
                                        <Show when={isChecked()}>
                                          <svg width="9" height="9" viewBox="0 0 16 16" fill="white"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0z"/></svg>
                                        </Show>
                                      </div>
                                      <span>{opt.value}</span>
                                    </div>
                                    <span class="font-mono text-[10px] text-[#555] tabular-nums">{opt.count}</span>
                                  </div>
                                );
                              }}
                            </For>
                            <Show when={filteredOpts().length === 0}>
                              <div class="px-2.5 py-3 text-[11px] text-[#555] text-center">No matches</div>
                            </Show>
                          </div>

                          {/* Footer */}
                          <Show when={selected().size > 0}>
                            <div class="flex items-center justify-between px-2.5 py-1.5 border-t border-[#3c3c3c] bg-[#252526] rounded-b-[3px]">
                              <span class="text-[10px] text-[#808080]">{selected().size} selected</span>
                              <button
                                type="button"
                                class="text-[10px] text-[#3584e4] hover:text-[#4a9ff1] cursor-pointer bg-transparent border-none"
                                onClick={() => {
                                  setFacetedSelections(prev => {
                                    const next = { ...prev };
                                    next[fc.key] = new Set();
                                    return next;
                                  });
                                }}
                              >
                                Clear
                              </button>
                            </div>
                          </Show>
                        </div>
                      </Show>
                    </div>
                  );
                }}
              </For>

              {/* Reset all filters */}
              <Show when={hasFacetedFilters()}>
                <button
                  type="button"
                  class="h-[26px] px-2 text-[11px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#808080] cursor-pointer flex items-center gap-1 hover:bg-[#3c3c3c] hover:text-[#cccccc]"
                  onClick={resetAllFacetedFilters}
                >
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
                  Reset
                </button>
              </Show>
            </Show>
          </div>
          <div class={styles.controlRight}>
            {props.headerActions}
          </div>
        </div>
      </Show>

      {/* Search Bar */}
      <Show when={enableSearch() && props.searchFields && props.searchFields.length > 0}>
        <div class={styles.searchBar}>
          <span class={styles.searchLabel}>Search by:</span>
          <select
            class={styles.searchSelect}
            value={searchField()}
            onChange={(e) => setSearchField(e.currentTarget.value)}
          >
            <For each={props.searchFields}>
              {(field) => <option value={field.key}>{field.label}</option>}
            </For>
          </select>
          <input
            type="text"
            class={styles.searchInput}
            placeholder="Type and press Enter..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            onKeyDown={(e) => e.key === 'Enter' && setSearchQuery(e.currentTarget.value)}
          />
          <button
            type="button"
            class={styles.controlBtn}
            onClick={() => setSearchQuery('')}
          >
            Clear
          </button>
        </div>
      </Show>

      {/* Table Container */}
      <div
        ref={tableContainerRef}
        class={styles.tableContainer}
        tabIndex={0}
        onKeyDown={handleKeyDown}
      >
        {/* Loading State */}
        <Show when={props.loadingState === 'loading'}>
          <div class={styles.loadingState}>
            <div class={styles.spinner} />
          </div>
        </Show>

        {/* Error State */}
        <Show when={props.loadingState === 'error'}>
          <div class={styles.errorState}>
            {props.error || 'An error occurred'}
          </div>
        </Show>

        {/* Data Table */}
        <Show when={props.loadingState !== 'loading' && props.loadingState !== 'error'}>
          <Show
            when={paginatedData().length > 0}
            fallback={<div class={styles.emptyState}>{props.emptyMessage || 'No data found'}</div>}
          >
            {/* Header */}
            <div class={styles.tableHeader}>
              <For each={props.columns}>
                {(column, colIndex) => {
                  const isLastColumn = () => colIndex() === props.columns.length - 1;
                  return (
                    <div
                      class={cx(
                        styles.headerCell,
                        !column.sortable && styles.headerCellNoSort,
                        column.align === 'right' && 'justify-end',
                        column.align === 'center' && 'justify-center'
                      )}
                      style={{
                        width: column.width,
                        'min-width': column.width?.endsWith('%') ? undefined : column.width,
                        flex: column.width?.endsWith('%') ? `0 0 ${column.width}` : (isLastColumn() ? '1 1 auto' : '0 0 auto'),
                      }}
                      onClick={() => column.sortable && handleSort(column.key as string)}
                    >
                      <div class={cx(styles.headerContent, column.align === 'right' && 'flex-row-reverse')}>
                        <span>{column.header}</span>
                        <Show when={column.sortable}>
                          <span class={cx(
                            styles.sortIndicator,
                            sortConfig().key === column.key && styles.sortIndicatorActive
                          )}>
                            {sortConfig().key === column.key
                              ? (sortConfig().direction === 'asc' ? '▲' : '▼')
                              : ''}
                          </span>
                        </Show>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>

            {/* Body */}
            <div class={styles.tableBody}>
              <For each={paginatedData()}>
                {(row, index) => (
                  <div
                    data-row-id={row.id}
                    class={cx(
                      styles.tableRow,
                      index() % 2 === 0 ? styles.tableRowEven : styles.tableRowOdd,
                      focusedId() === row.id && styles.tableRowFocused
                    )}
                    onClick={() => handleRowClick(row)}
                    onDblClick={() => handleRowDblClick(row)}
                    onContextMenu={(e) => handleContextMenu(e, row)}
                  >
                    <For each={props.columns}>
                      {(column, colIndex) => {
                        const isLastColumn = () => colIndex() === props.columns.length - 1;
                        return (
                          <div
                            class={cx(
                              styles.tableCell,
                              column.align === 'right' && styles.tableCellAlignRight,
                              column.align === 'center' && styles.tableCellAlignCenter
                            )}
                            style={{
                              width: column.width,
                              'min-width': column.width?.endsWith('%') ? undefined : column.width,
                              flex: column.width?.endsWith('%') ? `0 0 ${column.width}` : (isLastColumn() ? '1 1 auto' : '0 0 auto'),
                            }}
                          >
                            {getCellValue(row, column)}
                          </div>
                        );
                      }}
                    </For>
                    {/* Row actions overlay */}
                    <Show when={props.hoverActions && props.hoverActions.length > 0}>
                      <div class="absolute right-0 top-0 bottom-0 flex items-center gap-0.5 pr-2"
                        style={{ background: 'linear-gradient(to right, transparent, ' + (index() % 2 === 0 ? '#252525' : '#282828') + ' 12px)' }}
                      >
                        <For each={props.hoverActions}>
                          {(ha) => {
                            const visible = () => !ha.show || ha.show(row);
                            const isDisabled = () => ha.disabled ? ha.disabled(row) : false;
                            return (
                              <Show when={visible()}>
                                <button
                                  type="button"
                                  disabled={isDisabled()}
                                  class={cx(
                                    'p-1 rounded-[3px] transition-colors',
                                    isDisabled()
                                      ? 'text-[#555555] cursor-not-allowed'
                                      : ha.variant === 'destructive'
                                        ? 'text-[#c72e0f] hover:bg-[#c72e0f]/20'
                                        : 'text-[#cccccc] hover:bg-[#3c3c3c]'
                                  )}
                                  title={ha.label}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    if (!isDisabled()) {
                                      props.onRowAction?.(ha.action, row);
                                    }
                                  }}
                                >
                                  {ha.icon({ class: 'w-3.5 h-3.5' })}
                                </button>
                              </Show>
                            );
                          }}
                        </For>
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* Pagination Bar (DataGrip-style) */}
      <Show when={enablePagination() && paginatedData().length > 0}>
        <div class={styles.paginationBar}>
          {/* Left: row info */}
          <div class={styles.paginationLeft}>
            <span class={styles.paginationInfo}>
              {formatNumber(Math.min((currentPage() - 1) * pageSize() + 1, sortedData().length))}–{formatNumber(Math.min(currentPage() * pageSize(), sortedData().length))} of {formatNumber(sortedData().length)} rows
            </span>
          </div>

          {/* Right: controls */}
          <div class={styles.paginationRight}>
            {/* Rows per page */}
            <div class="flex items-center gap-1.5">
              <span class={styles.paginationLabel}>Rows:</span>
              <select
                class={styles.paginationSelect}
                value={pageSize()}
                onChange={(e) => {
                  setPageSize(parseInt(e.currentTarget.value));
                  setCurrentPage(1);
                }}
              >
                <For each={pageSizeOptions()}>
                  {(size) => <option value={size}>{size}</option>}
                </For>
              </select>
            </div>

            <div class={styles.paginationSeparator} />

            {/* Navigation */}
            <div class={styles.paginationNav}>
              {/* First */}
              <button
                type="button"
                class={styles.paginationNavBtn}
                onClick={() => goToPage('first')}
                disabled={currentPage() === 1}
                title="First page (Home)"
              >
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="11 13 6 8 11 3" />
                  <line x1="5" y1="3" x2="5" y2="13" />
                </svg>
              </button>
              {/* Prev */}
              <button
                type="button"
                class={styles.paginationNavBtn}
                onClick={() => goToPage('prev')}
                disabled={currentPage() === 1}
                title="Previous page (PgUp)"
              >
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="10 13 5 8 10 3" />
                </svg>
              </button>

              {/* Editable page input */}
              <div class="flex items-center gap-1">
                <input
                  type="text"
                  class={styles.paginationPageInput}
                  value={pageInputValue()}
                  onFocus={(e) => {
                    setPageInputEditing(true);
                    e.currentTarget.select();
                  }}
                  onBlur={() => commitPageInput()}
                  onInput={(e) => setPageInputValue(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      commitPageInput();
                      e.currentTarget.blur();
                    } else if (e.key === 'Escape') {
                      setPageInputValue(String(currentPage()));
                      setPageInputEditing(false);
                      e.currentTarget.blur();
                    }
                    e.stopPropagation();
                  }}
                />
                <span class={styles.paginationPageLabel}>/ {formatNumber(totalPages())}</span>
              </div>

              {/* Next */}
              <button
                type="button"
                class={styles.paginationNavBtn}
                onClick={() => goToPage('next')}
                disabled={currentPage() === totalPages()}
                title="Next page (PgDn)"
              >
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="6 3 11 8 6 13" />
                </svg>
              </button>
              {/* Last */}
              <button
                type="button"
                class={styles.paginationNavBtn}
                onClick={() => goToPage('last')}
                disabled={currentPage() === totalPages()}
                title="Last page (End)"
              >
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="5 3 10 8 5 13" />
                  <line x1="11" y1="3" x2="11" y2="13" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Status Bar */}
      <Show when={enableStatusBar()}>
        <div class={styles.statusBar}>
          <div class={styles.statusBarSection}>
            <span><kbd class={styles.kbd}>↑↓</kbd> Navigate</span>
            <span><kbd class={styles.kbd}>PgUp/Dn</kbd> Pages</span>
            <span><kbd class={styles.kbd}>Enter</kbd> Edit</span>
          </div>
          <div class={styles.statusBarSection}>
            <span><kbd class={styles.kbd}>Ctrl+C</kbd> Copy</span>
            <span><kbd class={styles.kbd}>Del</kbd> Delete</span>
          </div>
        </div>
      </Show>

      {/* Context Menu */}
      <Show when={contextMenu()}>
        {(menu) => (
          <div
            data-context-menu="true"
            class={styles.contextMenu}
            style={{
              left: `${Math.min(menu().x, window.innerWidth - 220)}px`,
              top: `${Math.min(menu().y, window.innerHeight - 250)}px`,
            }}
          >
            <div class={styles.contextMenuHeader}>
              Row Actions
            </div>
            <For each={props.contextMenuItems ?? []}>
              {(item) => {
                if ('type' in item && item.type === 'separator') {
                  return <div class={styles.contextMenuSeparator} />;
                }
                const menuItem = item as ContextMenuItem;
                return (
                  <div
                    class={cx(styles.contextMenuItem, menuItem.destructive && styles.contextMenuItemDanger)}
                    onClick={() => handleContextAction(menuItem.action)}
                  >
                    <span>{menuItem.label}</span>
                    <Show when={menuItem.shortcut}>
                      <span class={styles.contextMenuShortcut}>{menuItem.shortcut}</span>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>
        )}
      </Show>
    </div>
  );
}
