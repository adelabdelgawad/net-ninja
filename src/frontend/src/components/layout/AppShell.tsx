import { type Component, type JSX } from 'solid-js';
import { Sidebar } from './Sidebar';
import { StatusBar } from './StatusBar';
import { FallbackAlert } from './FallbackAlert';
import { Toaster } from '~/components/ui/toast';

interface AppShellProps {
  children: JSX.Element;
}

export const AppShell: Component<AppShellProps> = (props) => {
  return (
    <div class="flex h-screen w-screen overflow-hidden">
      <Sidebar />
      <div class="flex flex-1 flex-col overflow-hidden">
        <main class="flex flex-1 flex-col overflow-hidden">
          <FallbackAlert />
          {props.children}
        </main>
        <StatusBar />
      </div>
      <Toaster />
    </div>
  );
};
