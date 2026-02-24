import { type Component, For } from 'solid-js';
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
} from 'lucide-solid';

interface NavItem {
  path: string;
  icon: Component<{ class?: string }>;
  label: string;
}

const navItems: NavItem[] = [
  { path: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { path: '/lines', icon: Globe, label: 'Lines' },
  { path: '/tasks', icon: ListTodo, label: 'Tasks' },
  { path: '/quota', icon: BarChart3, label: 'Quota' },
  { path: '/speed', icon: Zap, label: 'Speed' },
  { path: '/email-settings', icon: Send, label: 'Email' },
  { path: '/logs', icon: FileText, label: 'Logs' },
];

export const Sidebar: Component = () => {
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  const navItemClass = (active: boolean) =>
    cn(
      'sidebar-item flex flex-row items-center gap-3 px-4 py-2.5 mx-3 my-2 text-[13px] font-medium transition-colors rounded-[8px]',
      active
        ? 'bg-[#2d2d2d] text-[#ffffff]'
        : 'text-[#999999] hover:text-[#cccccc] hover:bg-[#2d2d2d]/50'
    );

  return (
    <nav class="sidebar-nav flex h-full w-[200px] flex-col bg-sidebar">
      {/* Navigation section */}
      <div class="flex flex-1 flex-col">
        <For each={navItems}>
          {(item) => (
            <A href={item.path} class={navItemClass(isActive(item.path))}>
              <item.icon class="h-[18px] w-[18px]" />
              <span>{item.label}</span>
            </A>
          )}
        </For>
      </div>

      {/* Bottom section */}
      <div class="border-t border-sidebar-border">
        <A href="/about" class={navItemClass(isActive('/about'))}>
          <Info class="h-[18px] w-[18px]" />
          <span>About</span>
        </A>
      </div>
    </nav>
  );
};
