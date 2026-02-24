import { type Component, For } from 'solid-js';
import type { LineCreate } from '~/types';
import { ISP_OPTIONS } from '~/pages/Lines';

export interface NetworkSectionProps {
  formData: LineCreate;
  onUpdateField: (field: keyof LineCreate, value: string) => void;
}

const inputClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const labelClass = "text-[11px] text-[#808080] mb-1 block";

export const NetworkSection: Component<NetworkSectionProps> = (props) => {
  return (
    <div>
      <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-2">Network</div>
      <div class="grid grid-cols-2 gap-3 mb-3">
        <div>
          <label class={labelClass}>ISP</label>
          <select
            class={inputClass + " cursor-pointer"}
            value={props.formData.isp}
            onChange={(e) => props.onUpdateField('isp', e.currentTarget.value)}
          >
            <For each={[...ISP_OPTIONS]}>
              {(isp) => <option value={isp}>{isp}</option>}
            </For>
          </select>
        </div>
        <div>
          <label class={labelClass}>Description</label>
          <input
            type="text"
            class={inputClass}
            value={props.formData.description}
            onInput={(e) => props.onUpdateField('description', e.currentTarget.value)}
            placeholder="Main office connection"
          />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class={labelClass}>IP Address</label>
          <input
            type="text"
            class={inputClass + " font-mono"}
            value={props.formData.ipAddress}
            onInput={(e) => props.onUpdateField('ipAddress', e.currentTarget.value)}
            placeholder="192.168.1.1"
          />
        </div>
        <div>
          <label class={labelClass}>Gateway IP</label>
          <input
            type="text"
            class={inputClass + " font-mono"}
            value={props.formData.gatewayIp}
            onInput={(e) => props.onUpdateField('gatewayIp', e.currentTarget.value)}
            placeholder="192.168.1.254"
          />
        </div>
      </div>
    </div>
  );
};
