import type { Component } from 'solid-js';
import type { LineCreate } from '~/types';

export interface BasicInfoSectionProps {
  formData: LineCreate;
  onUpdateField: (field: keyof LineCreate, value: string) => void;
}

const inputClass = "h-[34px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const labelClass = "text-[11px] text-[#808080] mb-1 block";

export const BasicInfoSection: Component<BasicInfoSectionProps> = (props) => {
  return (
    <div>
      <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-2">Basic Info</div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class={labelClass}>
            Name <span class="text-[#c72e0f]">*</span>
          </label>
          <input
            type="text"
            class={inputClass}
            value={props.formData.name}
            onInput={(e) => props.onUpdateField('name', e.currentTarget.value)}
            placeholder="Office Main"
          />
        </div>
        <div>
          <label class={labelClass}>
            Line Number <span class="text-[#c72e0f]">*</span>
          </label>
          <input
            type="text"
            class={inputClass}
            value={props.formData.lineNumber}
            onInput={(e) => props.onUpdateField('lineNumber', e.currentTarget.value)}
            placeholder="01234567890"
          />
        </div>
      </div>
    </div>
  );
};
