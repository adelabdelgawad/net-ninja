import { type Component, createSignal, createResource, Show, For, Switch, Match } from 'solid-js';
import { Shield, Monitor, Terminal, Mail, Heart, Database, FileText, ChevronDown, ChevronRight, Download, RefreshCw, CheckCircle, AlertCircle, Loader2 } from 'lucide-solid';
import { getVersion } from '@tauri-apps/api/app';
import { check } from '@tauri-apps/plugin-updater';
import { open } from '@tauri-apps/plugin-shell';
import { appApi } from '~/api/client';
import { showToast } from '~/components/ui/toast';

// ============================================================================
// Types
// ============================================================================

interface GitHubRelease {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  html_url: string;
  assets: Array<{
    name: string;
    browser_download_url: string;
  }>;
}

type UpdateState = 'idle' | 'checking' | 'up-to-date' | 'available' | 'downloading' | 'error';

// ============================================================================
// Helpers
// ============================================================================

const GITHUB_REPO = 'adelabdelgawad/net-ninja';

/** Strip leading 'v' and parse semver into [major, minor, patch] */
function parseSemver(v: string): number[] {
  return v.replace(/^v/, '').split('.').map(Number);
}

/** Returns true if a > b */
function isNewerVersion(a: string, b: string): boolean {
  const pa = parseSemver(a);
  const pb = parseSemver(b);
  for (let i = 0; i < 3; i++) {
    if ((pa[i] ?? 0) > (pb[i] ?? 0)) return true;
    if ((pa[i] ?? 0) < (pb[i] ?? 0)) return false;
  }
  return false;
}

/** Fetch all releases newer than currentVersion from GitHub */
async function fetchNewerReleases(currentVersion: string): Promise<GitHubRelease[]> {
  const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases?per_page=50`);
  if (!res.ok) throw new Error(`GitHub API error: ${res.status}`);
  const all: GitHubRelease[] = await res.json();
  return all.filter(r => !r.tag_name.includes('draft') && isNewerVersion(r.tag_name, currentVersion));
}

/** Find the NSIS installer asset URL from a release */
function findInstallerUrl(release: GitHubRelease): string | null {
  const asset = release.assets.find(a =>
    a.name.endsWith('_x64-setup.exe') || a.name.endsWith('-setup.exe') || a.name.endsWith('.exe')
  );
  return asset?.browser_download_url ?? null;
}

/** Format ISO date to readable string */
function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

/** Simple markdown-to-HTML for release notes */
function renderMarkdown(md: string): string {
  return md
    // Headers
    .replace(/^### (.+)$/gm, '<h4 class="text-[13px] font-semibold text-[#eeeeee] mt-3 mb-1">$1</h4>')
    .replace(/^## (.+)$/gm, '<h3 class="text-[14px] font-semibold text-[#eeeeee] mt-4 mb-1.5">$1</h3>')
    .replace(/^# (.+)$/gm, '<h2 class="text-[15px] font-bold text-[#eeeeee] mt-4 mb-2">$1</h2>')
    // Bold
    .replace(/\*\*(.+?)\*\*/g, '<strong class="text-[#eeeeee]">$1</strong>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code class="px-1 py-0.5 bg-[#1e1e1e] text-[#ce9178] text-[12px] rounded-[3px]">$1</code>')
    // Unordered list items
    .replace(/^- (.+)$/gm, '<li class="ml-4 text-[12px] text-[#cccccc] leading-relaxed list-disc">$1</li>')
    // Horizontal rules
    .replace(/^---$/gm, '<hr class="border-[#3c3c3c] my-3" />')
    // Table rows (simple)
    .replace(/\|(.+)\|/g, (_, content: string) => {
      const cells = content.split('|').map((c: string) => c.trim()).filter(Boolean);
      if (cells.every((c: string) => /^-+$/.test(c))) return '';
      return `<div class="flex gap-4 text-[12px] text-[#cccccc] py-0.5">${cells.map((c: string) => `<span class="flex-1">${c}</span>`).join('')}</div>`;
    })
    // Line breaks
    .replace(/\n\n/g, '<br/>')
    .replace(/\n/g, '\n');
}

// ============================================================================
// Component
// ============================================================================

export const About: Component = () => {
  const [appVersion] = createResource(() => getVersion());
  const [dbPath] = createResource(async () => {
    try { return await appApi.getDatabasePath(); } catch { return 'Unable to retrieve'; }
  });
  const [logsPath] = createResource(async () => {
    try { return await appApi.getLogsPath(); } catch { return 'Unable to retrieve'; }
  });

  const [updateState, setUpdateState] = createSignal<UpdateState>('idle');
  const [latestVersion, setLatestVersion] = createSignal<string>('');
  const [releases, setReleases] = createSignal<GitHubRelease[]>([]);
  const [showDetails, setShowDetails] = createSignal(false);
  const [expandedReleases, setExpandedReleases] = createSignal<Set<string>>(new Set());
  const [downloadProgress, setDownloadProgress] = createSignal(0);
  const [errorMessage, setErrorMessage] = createSignal('');

  const checkForUpdates = async () => {
    setUpdateState('checking');
    setShowDetails(false);
    setErrorMessage('');
    try {
      const currentVersion = appVersion() ?? '0.0.0';
      const newer = await fetchNewerReleases(currentVersion);
      if (newer.length === 0) {
        setUpdateState('up-to-date');
      } else {
        setReleases(newer);
        setLatestVersion(newer[0].tag_name);
        setUpdateState('available');
      }
    } catch (e) {
      setErrorMessage(e instanceof Error ? e.message : String(e));
      setUpdateState('error');
    }
  };

  const installLatest = async () => {
    setUpdateState('downloading');
    setDownloadProgress(0);
    try {
      // Try the Tauri updater plugin first (works when signing is configured)
      const update = await check();
      if (update) {
        let downloaded = 0;
        let contentLength = 0;
        await update.downloadAndInstall((event) => {
          switch (event.event) {
            case 'Started':
              contentLength = event.data.contentLength ?? 0;
              break;
            case 'Progress':
              downloaded += event.data.chunkLength;
              if (contentLength > 0) {
                setDownloadProgress(Math.round((downloaded / contentLength) * 100));
              }
              break;
            case 'Finished':
              setDownloadProgress(100);
              break;
          }
        });
        showToast({ title: 'Update installed — restart to apply', variant: 'success', duration: 5000 });
        setUpdateState('idle');
        return;
      }
    } catch {
      // Updater plugin not configured (no pubkey) — fall back to browser download
    }

    // Fallback: open the installer download in the browser
    const latest = releases()[0];
    if (latest) {
      await downloadRelease(latest);
    }
    setUpdateState('available');
  };

  const downloadRelease = async (release: GitHubRelease) => {
    const url = findInstallerUrl(release);
    try {
      await open(url ?? release.html_url);
      showToast({ title: `Downloading ${release.tag_name}...`, variant: 'default', duration: 3000 });
    } catch {
      // Fallback if shell plugin fails
      window.open(url ?? release.html_url, '_blank');
    }
  };

  const toggleRelease = (tag: string) => {
    setExpandedReleases(prev => {
      const next = new Set(prev);
      if (next.has(tag)) next.delete(tag); else next.add(tag);
      return next;
    });
  };

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    showToast({ title: `${label} copied to clipboard`, variant: 'default', duration: 2000 });
  };

  return (
    <div class="flex flex-col h-full">
      {/* Content Header */}
      <div class="px-3 pt-2 pb-3 border-b border-[#3c3c3c] flex-shrink-0">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-[20px] font-semibold text-[#eeeeee]">About NetNinja</h1>
            <p class="text-[13px] text-[#999999]">Network monitoring and management for ISP lines</p>
          </div>
          <span class="px-3 py-1 text-[12px] font-mono bg-[#2d2d2d] text-[#999999] rounded-[6px] border border-[#3c3c3c]">
            v{appVersion() ?? '...'}
          </span>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto p-6">
        <div class="space-y-8">

          {/* ── Update Section ── */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">Software Update</h2>

            <Switch>
              {/* Idle — show check button */}
              <Match when={updateState() === 'idle'}>
                <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-4">
                  <div class="flex items-center justify-between">
                    <p class="text-[13px] text-[#999999]">Check if a newer version is available.</p>
                    <button
                      type="button"
                      class="flex items-center gap-2 px-4 py-1.5 text-[12px] font-medium rounded-[8px] bg-[#3584e4] text-white hover:bg-[#4a9ff1] transition-colors"
                      onClick={checkForUpdates}
                    >
                      <RefreshCw class="h-3.5 w-3.5" />
                      Check for Updates
                    </button>
                  </div>
                </div>
              </Match>

              {/* Checking spinner */}
              <Match when={updateState() === 'checking'}>
                <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-4">
                  <div class="flex items-center gap-3">
                    <Loader2 class="h-4 w-4 text-[#3584e4] animate-spin" />
                    <span class="text-[13px] text-[#cccccc]">Checking for updates...</span>
                  </div>
                </div>
              </Match>

              {/* Up to date */}
              <Match when={updateState() === 'up-to-date'}>
                <div class="rounded-[8px] border border-[#26a269]/30 bg-[#26a269]/5 p-4">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <CheckCircle class="h-4 w-4 text-[#26a269]" />
                      <span class="text-[13px] text-[#cccccc]">You're up to date! <span class="text-[#999999]">(v{appVersion()})</span></span>
                    </div>
                    <button
                      type="button"
                      class="px-3 py-1 text-[11px] rounded-[6px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#999999] hover:text-[#cccccc] hover:bg-[#3c3c3c] transition-colors"
                      onClick={checkForUpdates}
                    >
                      Check Again
                    </button>
                  </div>
                </div>
              </Match>

              {/* Update available */}
              <Match when={updateState() === 'available'}>
                <div class="space-y-3">
                  {/* Banner */}
                  <div class="rounded-[8px] border border-[#3584e4]/30 bg-[#3584e4]/5 p-4">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <Download class="h-4 w-4 text-[#3584e4]" />
                        <div>
                          <span class="text-[13px] text-[#cccccc]">
                            Version <strong class="text-white">{latestVersion()}</strong> is available
                          </span>
                          <span class="text-[11px] text-[#808080] ml-2">
                            (current: v{appVersion()})
                          </span>
                        </div>
                      </div>
                      <div class="flex items-center gap-2">
                        <button
                          type="button"
                          class="px-3 py-1.5 text-[12px] font-medium rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#555555] transition-colors"
                          onClick={() => setShowDetails(!showDetails())}
                        >
                          {showDetails() ? 'Hide Details' : 'View Details'}
                        </button>
                        <button
                          type="button"
                          class="flex items-center gap-2 px-4 py-1.5 text-[12px] font-medium rounded-[8px] bg-[#3584e4] text-white hover:bg-[#4a9ff1] transition-colors"
                          onClick={installLatest}
                        >
                          <Download class="h-3.5 w-3.5" />
                          Install Now
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Release details */}
                  <Show when={showDetails()}>
                    <div class="space-y-2">
                      <p class="text-[12px] text-[#808080]">
                        {releases().length} release{releases().length > 1 ? 's' : ''} between your version and the latest:
                      </p>
                      <For each={releases()}>
                        {(release) => {
                          const isExpanded = () => expandedReleases().has(release.tag_name);
                          return (
                            <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 overflow-hidden">
                              {/* Release header */}
                              <div class="flex items-center justify-between px-4 py-2.5">
                                <button
                                  type="button"
                                  class="flex items-center gap-2 flex-1 text-left"
                                  onClick={() => toggleRelease(release.tag_name)}
                                >
                                  {isExpanded()
                                    ? <ChevronDown class="h-3.5 w-3.5 text-[#808080] shrink-0" />
                                    : <ChevronRight class="h-3.5 w-3.5 text-[#808080] shrink-0" />
                                  }
                                  <span class="text-[13px] font-medium text-[#eeeeee]">{release.tag_name}</span>
                                  <span class="text-[11px] text-[#808080]">{formatDate(release.published_at)}</span>
                                  <Show when={release.tag_name === releases()[0]?.tag_name}>
                                    <span class="px-1.5 py-0.5 text-[10px] font-medium bg-[#3584e4]/15 text-[#3584e4] rounded-[4px]">latest</span>
                                  </Show>
                                </button>
                                <button
                                  type="button"
                                  class="flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium rounded-[6px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3584e4] hover:border-[#3584e4] hover:text-white transition-colors shrink-0"
                                  onClick={() => downloadRelease(release)}
                                >
                                  <Download class="h-3 w-3" />
                                  Download
                                </button>
                              </div>

                              {/* Release body (expanded) */}
                              <Show when={isExpanded()}>
                                <div class="px-4 pb-3 border-t border-[#3c3c3c]">
                                  <div
                                    class="pt-3 text-[12px] text-[#cccccc] leading-relaxed prose-sm"
                                    innerHTML={renderMarkdown(release.body || 'No release notes.')}
                                  />
                                </div>
                              </Show>
                            </div>
                          );
                        }}
                      </For>
                    </div>
                  </Show>
                </div>
              </Match>

              {/* Downloading */}
              <Match when={updateState() === 'downloading'}>
                <div class="rounded-[8px] border border-[#3584e4]/30 bg-[#3584e4]/5 p-4 space-y-3">
                  <div class="flex items-center gap-3">
                    <Loader2 class="h-4 w-4 text-[#3584e4] animate-spin" />
                    <span class="text-[13px] text-[#cccccc]">Downloading update... {downloadProgress()}%</span>
                  </div>
                  <div class="h-2 bg-[#1e1e1e] rounded-full overflow-hidden">
                    <div
                      class="h-full bg-[#3584e4] rounded-full transition-all duration-300"
                      style={{ width: `${downloadProgress()}%` }}
                    />
                  </div>
                </div>
              </Match>

              {/* Error */}
              <Match when={updateState() === 'error'}>
                <div class="rounded-[8px] border border-[#c72e0f]/30 bg-[#c72e0f]/5 p-4">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <AlertCircle class="h-4 w-4 text-[#c72e0f]" />
                      <div>
                        <span class="text-[13px] text-[#cccccc]">Failed to check for updates</span>
                        <Show when={errorMessage()}>
                          <p class="text-[11px] text-[#808080] mt-0.5">{errorMessage()}</p>
                        </Show>
                      </div>
                    </div>
                    <button
                      type="button"
                      class="px-3 py-1 text-[11px] rounded-[6px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#999999] hover:text-[#cccccc] hover:bg-[#3c3c3c] transition-colors"
                      onClick={checkForUpdates}
                    >
                      Retry
                    </button>
                  </div>
                </div>
              </Match>
            </Switch>
          </section>

          {/* ── Description ── */}
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

          {/* ── Features grid ── */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">Key Features</h2>
            <div class="grid grid-cols-2 gap-3">
              <FeatureCard icon={Monitor} title="Line Monitoring" description="Track multiple ISP lines with real-time quota and speed data" />
              <FeatureCard icon={Shield} title="Automated Testing" description="Schedule quota checks and speed tests with flexible task scheduling" />
              <FeatureCard icon={Mail} title="Email Reports" description="Receive automated reports via SMTP with configurable recipients" />
              <FeatureCard icon={Terminal} title="Local Storage" description="Lightweight SQLite database for fast, self-contained data storage" />
            </div>
          </section>

          {/* ── Free & Open ── */}
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

          {/* ── System Paths ── */}
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
                      <button onclick={() => copyToClipboard(path(), 'Database path')} class="px-2 py-1 text-[11px] bg-[#3c3c3c] hover:bg-[#4a4a4a] text-[#999999] hover:text-[#cccccc] rounded-[4px] transition-colors flex-shrink-0">Copy</button>
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
                      <button onclick={() => copyToClipboard(path(), 'Logs path')} class="px-2 py-1 text-[11px] bg-[#3c3c3c] hover:bg-[#4a4a4a] text-[#999999] hover:text-[#cccccc] rounded-[4px] transition-colors flex-shrink-0">Copy</button>
                    </div>
                  )}
                </Show>
              </div>
            </div>
            <p class="text-[12px] text-[#808080]">
              These paths are used for data storage and log files. Click Copy to copy the path to clipboard.
            </p>
          </section>

          {/* ── Platform Support ── */}
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

          {/* ── Author & Support ── */}
          <section class="space-y-3">
            <h2 class="text-[14px] font-semibold text-[#eeeeee]">Author & Support</h2>
            <div class="rounded-[8px] border border-[#3c3c3c] bg-[#2d2d2d]/30 p-4 space-y-3">
              <div class="flex items-center gap-2">
                <span class="text-[13px] text-[#999999]">Developed by</span>
                <span class="text-[13px] text-[#eeeeee] font-medium">Adel</span>
              </div>
              <div class="flex items-center gap-2">
                <Mail class="h-3.5 w-3.5 text-[#999999]" />
                <a href="mailto:tech.adel87@gmail.com" class="text-[13px] text-[#3584e4] hover:text-[#4a9ff1] hover:underline">
                  tech.adel87@gmail.com
                </a>
              </div>
              <p class="text-[13px] text-[#999999] leading-relaxed">
                For any questions, bug reports, feature requests, or support — feel free to
                reach out via email. All feedback is welcome.
              </p>
            </div>
          </section>

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
