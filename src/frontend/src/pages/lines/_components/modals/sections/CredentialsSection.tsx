import type { Component } from 'solid-js';
import type { LineCreate } from '~/types';

export interface CredentialsSectionProps {
  formData: LineCreate;
  onUpdateField: (field: keyof LineCreate, value: string) => void;
}

const inputClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const labelClass = "text-[11px] text-[#808080] mb-1 block";

export const CredentialsSection: Component<CredentialsSectionProps> = (props) => {
  return (
    <div>
      <div class="flex items-center gap-1.5 mb-2">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="#808080"><path d="M8 1a4 4 0 0 0-4 4v2H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V8a1 1 0 0 0-1-1h-1V5a4 4 0 0 0-4-4zm0 1a3 3 0 0 1 3 3v2H5V5a3 3 0 0 1 3-3zm-5 6h10v6H3V8z"/></svg>
        <span class="text-[10px] text-[#808080] uppercase tracking-wider font-medium">Portal Credentials</span>
      </div>
      <div class="bg-[#1e1e1e] rounded-none border border-[#2a2a2a] p-3">
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class={labelClass}>Username</label>
            <input
              type="text"
              class={inputClass + " bg-[#2d2d2d]"}
              value={props.formData.portalUsername}
              onInput={(e) => props.onUpdateField('portalUsername', e.currentTarget.value)}
              placeholder="username"
            />
          </div>
          <div>
            <label class={labelClass}>Password</label>
            <input
              type="text"
              class={inputClass + " bg-[#2d2d2d]"}
              value={props.formData.portalPassword}
              onInput={(e) => props.onUpdateField('portalPassword', e.currentTarget.value)}
              placeholder="password"
            />
          </div>
        </div>
      </div>
    </div>
  );
};
