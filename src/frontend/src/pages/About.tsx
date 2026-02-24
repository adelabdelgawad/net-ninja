import { type Component, createResource, Show } from 'solid-js';
import { Shield, Monitor, Terminal, Mail, Heart, Database, FileText } from 'lucide-solid';
import { appApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';

export const About: Component = () => {
  const [dbPath] = createResource(async () => {
    try {
      return await appApi.getDatabasePath();
    } catch {
      return 'Unable to retrieve';
    }
  });

  const [logsPath] = createResource(async () => {
    try {
      return await appApi.getLogsPath();
    } catch {
      return 'Unable to retrieve';
    }
  });

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    showToast({ title: `${label} copied to clipboard`, variant: 'default', duration: 2000 });
  };
  return (
    <div class="flex flex-col h-full">
      {/* Content Header with Actions */}
      <div class="px-3 pt-2 pb-3 border-b border-[#3c3c3c] flex-shrink-0">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-[20px] font-semibold text-[#eeeeee]">About NetNinja</h1>
            <p class="text-[13px] text-[#999999]">Network monitoring and management for ISP lines</p>
          </div>
          <span class="px-3 py-1 text-[12px] font-mono bg-[#2d2d2d] text-[#999999] rounded-[6px] border border-[#3c3c3c]">
            v0.1.1
          </span>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto p-6">
        <div class="space-y-8">

          {/* Description */}
          <section class="space-y-3">
            <p class="text-[13px] text-[#cccccc] leading-relaxed">
              NetNinja is a desktop application designed for network administrators and ISPs to monitor
              internet lines, automate quota checking, run speed tests, and generate detailed reports —
              all from a single, unified interface.
            </p>
            <p class="text-[13px] text-[#999999] leading-relaxed">
              Built with performance and reliability in mind using Tauri, Rust, and SolidJS.
            </p>
          </section>

          {/* Features grid */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">Key Features</h2>
            <div class="grid grid-cols-2 gap-3">
              <FeatureCard
                icon={Monitor}
                title="Line Monitoring"
                description="Track multiple ISP lines with real-time quota and speed data"
              />
              <FeatureCard
                icon={Shield}
                title="Automated Testing"
                description="Schedule quota checks and speed tests with flexible task scheduling"
              />
              <FeatureCard
                icon={Mail}
                title="Email Reports"
                description="Receive automated reports via SMTP with configurable recipients"
              />
              <FeatureCard
                icon={Terminal}
                title="Local Storage"
                description="Lightweight SQLite database for fast, self-contained data storage"
              />
            </div>
          </section>

          {/* Free & Open */}
          <section class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/50 p-4 space-y-2">
            <div class="flex items-center gap-2">
              <Heart class="h-4 w-4 text-[#e06c75]" />
              <h2 class="text-[14px] font-semibold text-[#eeeeee]">Free to Use</h2>
            </div>
            <p class="text-[13px] text-[#cccccc] leading-relaxed">
              NetNinja is built and provided completely free of charge — no license fees,
              no subscriptions, no hidden costs. You are welcome to use it for personal
              or commercial purposes at no cost.
            </p>
          </section>

          {/* System Paths */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">System Paths</h2>
            <div class="space-y-2">
              <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-3">
                <div class="flex items-center gap-2 mb-1.5">
                  <Database class="h-3.5 w-3.5 text-[#4ec9b0]" />
                  <span class="text-[12px] font-medium text-[#999999]">Database Path</span>
                </div>
                <Show when={dbPath()} fallback={<span class="text-[13px] text-[#808080]">Loading...</span>}>
                  {(path) => (
                    <div class="flex items-center justify-between gap-2">
                      <span class="text-[13px] text-[#cccccc] font-mono truncate">{path()}</span>
                      <button
                        onclick={() => copyToClipboard(path(), 'Database path')}
                        class="px-2 py-1 text-[11px] bg-[#3c3c3c] hover:bg-[#4a4a4a] text-[#999999] hover:text-[#cccccc] rounded-[4px] transition-colors flex-shrink-0"
                      >
                        Copy
                      </button>
                    </div>
                  )}
                </Show>
              </div>
              <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-3">
                <div class="flex items-center gap-2 mb-1.5">
                  <FileText class="h-3.5 w-3.5 text-[#dcdcaa]" />
                  <span class="text-[12px] font-medium text-[#999999]">Logs Path</span>
                </div>
                <Show when={logsPath()} fallback={<span class="text-[13px] text-[#808080]">Loading...</span>}>
                  {(path) => (
                    <div class="flex items-center justify-between gap-2">
                      <span class="text-[13px] text-[#cccccc] font-mono truncate">{path()}</span>
                      <button
                        onclick={() => copyToClipboard(path(), 'Logs path')}
                        class="px-2 py-1 text-[11px] bg-[#3c3c3c] hover:bg-[#4a4a4a] text-[#999999] hover:text-[#cccccc] rounded-[4px] transition-colors flex-shrink-0"
                      >
                        Copy
                      </button>
                    </div>
                  )}
                </Show>
              </div>
            </div>
            <p class="text-[12px] text-[#808080]">
              These paths are used for data storage and log files. Click Copy to copy the path to clipboard.
            </p>
          </section>

          {/* Platform Support */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">Platform Support</h2>
            <div class="space-y-2">
              <div class="flex items-center gap-3 px-3 py-2.5 rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30">
                <span class="w-2 h-2 rounded-full bg-[#4ec9b0]" />
                <span class="text-[13px] text-[#cccccc] font-medium">Windows</span>
                <span class="ml-auto text-[12px] text-[#4ec9b0] font-mono">Stable</span>
              </div>
              <div class="flex items-center gap-3 px-3 py-2.5 rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30">
                <span class="w-2 h-2 rounded-full bg-[#dcdcaa]" />
                <span class="text-[13px] text-[#cccccc] font-medium">Linux</span>
                <span class="ml-auto text-[12px] text-[#dcdcaa] font-mono">Not stable for production</span>
              </div>
            </div>
            <p class="text-[12px] text-[#808080]">
              Windows is the primary supported platform. The Linux version is available but
              not yet stable enough for production use.
            </p>
          </section>

          {/* Author & Support */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">Author & Support</h2>
            <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-4 space-y-3">
              <div class="flex items-center gap-2">
                <span class="text-[13px] text-[#999999]">Developed by</span>
                <span class="text-[13px] text-[#eeeeee] font-medium">Adel</span>
              </div>
              <div class="flex items-center gap-2">
                <Mail class="h-3.5 w-3.5 text-[#999999]" />
                <a
                  href="mailto:tech.adel87@gmail.com"
                  class="text-[13px] text-[#3584e4] hover:text-[#4a9ff1] hover:underline"
                >
                  tech.adel87@gmail.com
                </a>
              </div>
              <p class="text-[13px] text-[#999999] leading-relaxed">
                For any questions, bug reports, feature requests, or support — feel free to
                reach out via email. All feedback is welcome.
              </p>
            </div>
          </section>

          {/* Footer spacer */}
          <div class="pb-4" />
        </div>
      </div>
    </div>
  );
};

/* ── Tiny helper ───────────────────────────────────────────── */

const FeatureCard: Component<{
  icon: Component<{ class?: string }>;
  title: string;
  description: string;
}> = (props) => (
  <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-3 space-y-1.5">
    <div class="flex items-center gap-2">
      <props.icon class="h-4 w-4 text-[#3584e4]" />
      <span class="text-[13px] font-medium text-[#eeeeee]">{props.title}</span>
    </div>
    <p class="text-[12px] text-[#999999] leading-relaxed">{props.description}</p>
  </div>
);
