import { type Component, Show } from 'solid-js';
import type { SmtpConfig } from '~/types';
import { VENDOR_CONFIGS } from '~/utils/smtp-vendors';
import { SmtpServerActions } from './SmtpServerActions';

interface SmtpServerRowProps {
  config: SmtpConfig;
  onTest: () => void;
  onEdit: () => void;
  onSetDefault: () => void;
  onDelete: () => void;
}

export const SmtpServerRow: Component<SmtpServerRowProps> = (props) => {
  return (
    <div class="group flex items-center gap-3 px-4 py-3 hover:bg-[#2d2d2d] transition-colors rounded-[8px]">
      {/* Mail icon — green: default+active, blue: active, grey: inactive */}
      <div
        class="flex items-center justify-center h-9 w-9 rounded-[10px] shrink-0"
        classList={{
          'bg-[#26a269]/15': props.config.isDefault && props.config.isActive,
          'bg-[#3584e4]/15': !props.config.isDefault && props.config.isActive,
          'bg-[#808080]/10': !props.config.isActive,
        }}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill={
            !props.config.isActive
              ? '#808080'
              : props.config.isDefault
                ? '#57e389'
                : '#3584e4'
          }
        >
          <path d="M2 4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4zm2-1a1 1 0 0 0-1 1v.217l5 3.125 5-3.125V4a1 1 0 0 0-1-1H4zm9 2.441-4.724 2.953a.5.5 0 0 1-.552 0L3 5.441V12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V5.441z"/>
        </svg>
      </div>

      {/* Content */}
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-[13px] font-medium text-[#eeeeee] truncate">{props.config.name}</span>
          <span class="px-2 py-0.5 text-[10px] rounded-[3px] bg-[#2d2d2d] text-[#808080] border border-[#3c3c3c]">
            {VENDOR_CONFIGS[props.config.vendor].displayName}
          </span>
          <Show when={props.config.isDefault}>
            <span class="px-1.5 py-0 text-[10px] leading-4 font-medium rounded-full bg-[#26a269]/20 text-[#57e389]">
              Default
            </span>
          </Show>
          <Show when={!props.config.isActive}>
            <span class="px-1.5 py-0 text-[10px] leading-4 font-medium rounded-full bg-[#808080]/15 text-[#808080]">
              Inactive
            </span>
          </Show>
        </div>
        <div class="text-[11px] text-[#999999] mt-0.5">
          {props.config.host}:{props.config.port} &middot; {props.config.senderName} &lt;{props.config.senderEmail}&gt;
        </div>
      </div>

      {/* Actions */}
      <SmtpServerActions
        config={props.config}
        onTest={props.onTest}
        onEdit={props.onEdit}
        onSetDefault={props.onSetDefault}
        onDelete={props.onDelete}
      />
    </div>
  );
};
