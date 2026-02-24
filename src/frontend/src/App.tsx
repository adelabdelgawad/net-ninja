import { type Component, lazy, createSignal, Show, onMount } from 'solid-js';
import { Router, Route } from '@solidjs/router';
import { AppShell } from '~/components/layout';
import { LoadingSkeleton } from '~/components/layout/LoadingSkeleton';
import { getSettingsStore } from '~/stores/SettingsStore';
import { QueryProvider } from '~/providers/QueryProvider';

// Lazy load pages
const Dashboard = lazy(() => import('~/pages/Dashboard').then(m => ({ default: m.Dashboard })));
const Lines = lazy(() => import('~/pages/Lines').then(m => ({ default: m.Lines })));
const Tasks = lazy(() => import('~/pages/Tasks').then(m => ({ default: m.Tasks })));
const QuotaResults = lazy(() => import('~/pages/QuotaResults').then(m => ({ default: m.QuotaResults })));
const SpeedResults = lazy(() => import('~/pages/SpeedResults').then(m => ({ default: m.SpeedResults })));
const Logs = lazy(() => import('~/pages/Logs').then(m => ({ default: m.Logs })));
// Email Settings - kept for SMTP configuration
const EmailSettings = lazy(() => import('~/pages/EmailSettings').then(m => ({ default: m.EmailSettings })));
const About = lazy(() => import('~/pages/About').then(m => ({ default: m.About })));

const App: Component = () => {
  const store = getSettingsStore();
  const [isReady, setIsReady] = createSignal(false);

  onMount(async () => {
    try {
      await store.load();
    } catch (e) {
      console.error('Failed to initialize settings:', e);
    } finally {
      setIsReady(true);
    }
  });

  return (
    <Show when={isReady()} fallback={<LoadingSkeleton />}>
      <QueryProvider>
        <Router root={(props) => <AppShell>{props.children}</AppShell>}>
          <Route path="/" component={Dashboard} />
          <Route path="/lines" component={Lines} />
          <Route path="/tasks" component={Tasks} />
          <Route path="/quota" component={QuotaResults} />
          <Route path="/speed" component={SpeedResults} />
          <Route path="/logs" component={Logs} />
          {/* Email Settings - SMTP configuration */}
          <Route path="/email-settings" component={EmailSettings} />
          <Route path="/about" component={About} />
        </Router>
      </QueryProvider>
    </Show>
  );
};

export default App;
