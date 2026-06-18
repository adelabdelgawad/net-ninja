import { type Component, For, Show, createSignal } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import { cn } from '~/lib/utils';
import {
  LayoutDashboard,
  Globe,
  ListTodo,
  BarChart3,
  Zap,
  FileText,
  Send,
  Info,
  Network,
  ChevronsLeft,
} from 'lucide-solid';

interface NavItem {
  path: string;
  icon: Component<{ class?: string }>;
  label: string;
  /** Optional count badge (e.g. active tasks, unread alerts). */
  badge?: number;
}

interface NavSection {
  title: string;
  items: NavItem[];
}

const navSections: NavSection[] = [
  {
    title: 'Monitoring',
    items: [
      { path: '/', icon: LayoutDashboard, label: 'Dashboard' },
      { path: '/lines', icon: Globe, label: 'Lines' },
      { path: '/tasks', icon: ListTodo, label: 'Tasks' },
    ],
  },
  {
    title: 'Results',
    items: [
      { path: '/quota', icon: BarChart3, label: 'Quota' },
      { path: '/speed', icon: Zap, label: 'Speed' },
      { path: '/logs', icon: FileText, label: 'Logs' },
    ],
  },
  {
    title: 'Settings',
    items: [{ path: '/email-settings', icon: Send, label: 'Email' }],
  },
];

const STORAGE_KEY = 'netninja.sidebar.collapsed';

export const Sidebar: Component = () => {
  const location = useLocation();

  const [collapsed, setCollapsed] = createSignal(
    typeof localStorage !== 'undefined' && localStorage.getItem(STORAGE_KEY) === '1'
  );

  const toggle = () => {
    const next = !collapsed();
    setCollapsed(next);
    try {
      localStorage.setItem(STORAGE_KEY, next ? '1' : '0');
    } catch {
      /* localStorage unavailable (private mode) — ignore */
    }
  };

  const isActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  const navItemClass = (active: boolean) =>
    cn(
      'sidebar-item flex flex-row items-center gap-3 rounded-[12px] px-4 py-2.5 text-[13px] font-medium transition-colors',
      collapsed() && 'justify-center px-0',
      active
        ? 'bg-sidebar-accent text-sidebar-accent-foreground'
        : 'text-sidebar-foreground hover:bg-accent hover:text-foreground'
    );

  const renderItem = (item: NavItem) => (
    <A
      href={item.path}
      class={navItemClass(isActive(item.path))}
      title={collapsed() ? item.label : undefined}
    >
      <item.icon class="h-[18px] w-[18px] shrink-0" />
      <Show when={!collapsed()}>
        <span class="flex-1 truncate">{item.label}</span>
        <Show when={item.badge}>
          <span class="rounded-full bg-warning px-2 py-0.5 text-[11px] font-bold text-warning-foreground">
            {item.badge}
          </span>
        </Show>
      </Show>
      {/* Collapsed: surface the badge as a corner dot */}
      <Show when={collapsed() && item.badge}>
        <span class="absolute right-1.5 top-1.5 h-2 w-2 rounded-full bg-warning" />
      </Show>
    </A>
  );

  return (
    <nav
      class={cn(
        'sidebar-nav flex h-full shrink-0 flex-col bg-sidebar transition-[width] duration-300 ease-in-out',
        collapsed() ? 'w-[64px]' : 'w-[200px]'
      )}
    >
      {/* Logo / brand */}
      <div class="flex h-[60px] shrink-0 items-center gap-3 border-b border-sidebar-border px-4">
        <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-[10px] bg-primary">
          <Network class="h-5 w-5 text-primary-foreground" />
        </div>
        <Show when={!collapsed()}>
          <span class="truncate text-[17px] font-bold tracking-tight text-foreground">
            NetNinja
          </span>
        </Show>
      </div>

      {/* Grouped navigation */}
      <div class="flex flex-1 flex-col gap-5 overflow-y-auto overflow-x-hidden px-3 py-4">
        <For each={navSections}>
          {(section) => (
            <div class="flex flex-col gap-1">
              <Show when={!collapsed()}>
                <div class="px-3 pb-1 text-[11px] font-semibold uppercase tracking-wider text-sidebar-foreground/50">
                  {section.title}
                </div>
              </Show>
              <For each={section.items}>{(item) => renderItem(item)}</For>
            </div>
          )}
        </For>
      </div>

      {/* Bottom: About + collapse toggle */}
      <div class="flex flex-col gap-1 border-t border-sidebar-border px-3 py-3">
        {renderItem({ path: '/about', icon: Info, label: 'About' })}
        <button
          type="button"
          onClick={toggle}
          title={collapsed() ? 'Expand sidebar' : 'Collapse sidebar'}
          class={cn(
            'flex items-center rounded-[12px] px-4 py-2.5 text-sidebar-foreground transition-colors hover:bg-accent hover:text-foreground',
            collapsed() ? 'justify-center px-0' : 'justify-end'
          )}
        >
          <ChevronsLeft
            class={cn('h-[18px] w-[18px] transition-transform duration-300', collapsed() && 'rotate-180')}
          />
        </button>
      </div>
    </nav>
  );
};
