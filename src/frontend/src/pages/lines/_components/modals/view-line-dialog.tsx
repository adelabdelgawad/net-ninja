import { type Component, Show } from 'solid-js';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '~/components/ui/dialog';
import type { Line } from '~/types';

export interface ViewLineDialogProps {
  open: boolean;
  line: Line | null;
  onOpenChange: (open: boolean) => void;
  onEdit: (line: Line) => void;
}

const DetailRow: Component<{ label: string; value: string; mono?: boolean }> = (props) => (
  <div class="flex items-baseline justify-between py-1.5 border-b border-[#2a2a2a] last:border-b-0">
    <span class="text-[11px] text-[#808080] uppercase tracking-wider">{props.label}</span>
    <span class={`text-[12px] text-[#cccccc] ${props.mono ? 'font-mono' : ''}`}>{props.value || '-'}</span>
  </div>
);

export const ViewLineDialog: Component<ViewLineDialogProps> = (props) => {
  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[460px]">
        <DialogHeader>
          <DialogTitle>Line Details</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <Show when={props.line}>
          {(line) => (
            <div class="px-4 py-3">
              {/* Basic Info */}
              <div class="mb-3">
                <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-1.5">General</div>
                <div class="bg-[#1e1e1e] rounded-[3px] border border-[#2a2a2a] px-3">
                  <DetailRow label="Name" value={line().name} />
                  <DetailRow label="Line Number" value={line().lineNumber} mono />
                  <DetailRow label="ISP" value={line().isp ?? '-'} />
                  <DetailRow label="Description" value={line().description ?? '-'} />
                  <DetailRow label="Active" value={line().isActive ? 'Yes' : 'No'} />
                </div>
              </div>

              {/* Network */}
              <div class="mb-3">
                <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-1.5">Network</div>
                <div class="bg-[#1e1e1e] rounded-[3px] border border-[#2a2a2a] px-3">
                  <DetailRow label="IP Address" value={line().ipAddress ?? '-'} mono />
                  <DetailRow label="Gateway IP" value={line().gatewayIp ?? '-'} mono />
                </div>
              </div>

              {/* Meta */}
              <div>
                <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-1.5">Metadata</div>
                <div class="bg-[#1e1e1e] rounded-[3px] border border-[#2a2a2a] px-3">
                  <DetailRow label="Created" value={line().createdAt ?? '-'} mono />
                </div>
              </div>
            </div>
          )}
        </Show>
        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            Close
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] transition-colors flex items-center gap-1.5"
            onClick={() => {
              if (props.line) {
                props.onEdit(props.line);
                props.onOpenChange(false);
              }
            }}
          >
            <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M13.23 1h-1.46L3.52 9.25l-.16.22L1 13.59 2.41 15l4.12-2.36.22-.16L15 4.23V2.77L13.23 1zM2.41 13.59l1.51-3 1.45 1.45-2.96 1.55zm3.83-2.06L4.47 9.76l6-6 1.77 1.77-6 6z"/></svg>
            Edit
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
