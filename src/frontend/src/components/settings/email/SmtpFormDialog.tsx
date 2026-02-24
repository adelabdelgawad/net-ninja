import { type Component, type Accessor, type Setter } from 'solid-js';
import { Switch } from '~/components/ui/switch';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '~/components/ui/dialog';
import type { SmtpConfigCreate } from '~/types';
import { VENDOR_CONFIGS, applyVendorDefaults, type SmtpVendor } from '~/utils/smtp-vendors';

interface SmtpFormDialogProps {
  open: Accessor<boolean>;
  onOpenChange: Setter<boolean>;
  mode: 'add' | 'edit';
  formData: Accessor<Partial<SmtpConfigCreate>>;
  updateFormField: <K extends keyof SmtpConfigCreate>(field: K, value: SmtpConfigCreate[K]) => void;
  onSubmit: () => void;
  isLoading: Accessor<boolean>;
}

const inputClass = "h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none focus:outline-none focus:border-[#007acc] placeholder:text-[#555]";
const labelClass = "text-[11px] text-[#808080] mb-1 block";

export const SmtpFormDialog: Component<SmtpFormDialogProps> = (props) => {
  const isAdd = () => props.mode === 'add';

  return (
    <Dialog open={props.open()} onOpenChange={props.onOpenChange}>
      <DialogContent class="max-w-[520px]">
        <DialogHeader>
          <DialogTitle>{isAdd() ? 'Add SMTP Server' : 'Edit SMTP Server'}</DialogTitle>
          <button
            type="button"
            class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/></svg>
          </button>
        </DialogHeader>
        <DialogDescription class="px-4 py-2 text-[11px] text-[#808080]">
          {isAdd()
            ? 'Configure a new SMTP server for sending emails'
            : 'Update the SMTP server configuration'}
        </DialogDescription>

        <div class="px-4 py-3 space-y-4">
          <div>
            <label class={labelClass}>Email Provider</label>
            <select
              class="w-full px-3 py-1.5 text-[12px] text-[#cccccc] bg-[#2d2d2d] border border-[#3c3c3c] focus:outline-none focus:border-[#007acc]"
              value={props.formData().vendor || 'gmail'}
              onChange={(e) => {
                const vendor = e.currentTarget.value as SmtpVendor;
                const newFormData = applyVendorDefaults(vendor, {
                  name: props.formData().name,
                  username: props.formData().username,
                  password: props.formData().password,
                  senderEmail: props.formData().senderEmail,
                  senderName: props.formData().senderName,
                });
                Object.entries(newFormData).forEach(([key, value]) => {
                  props.updateFormField(key as keyof SmtpConfigCreate, value as SmtpConfigCreate[keyof SmtpConfigCreate]);
                });
              }}
            >
              <option value="gmail">Gmail</option>
              <option value="exchange">Microsoft Exchange</option>
              <option value="outlook365">Outlook 365</option>
            </select>
            <p class="mt-1.5 text-[10px] text-[#808080]">
              {VENDOR_CONFIGS[props.formData().vendor || 'gmail'].description}
            </p>
          </div>

          <div>
            <label class={labelClass}>Name</label>
            <input
              type="text"
              class={inputClass}
              value={props.formData().name || ''}
              onInput={(e) => props.updateFormField('name', e.currentTarget.value)}
              placeholder={isAdd() ? 'e.g., Gmail, Office 365' : ''}
            />
          </div>

          <div class="grid grid-cols-[2fr_1fr] gap-3">
            <div>
              <label class={labelClass}>
                Host
                {props.formData().vendor === 'exchange' && <span class="text-[#f48771] ml-1">*</span>}
              </label>
              <input
                type="text"
                class={inputClass}
                value={props.formData().host || ''}
                onInput={(e) => props.updateFormField('host', e.currentTarget.value)}
                placeholder={props.formData().vendor === 'exchange' ? 'mail.company.com' : ''}
                disabled={!VENDOR_CONFIGS[props.formData().vendor || 'gmail'].hostEditable}
              />
            </div>
            <div>
              <label class={labelClass}>Port</label>
              <input
                type="number"
                class={inputClass}
                value={props.formData().port || 587}
                onInput={(e) => props.updateFormField('port', parseInt(e.currentTarget.value, 10))}
              />
            </div>
          </div>

          <div>
            <label class={labelClass}>Username</label>
            <input
              type="text"
              class={inputClass}
              value={props.formData().username || ''}
              onInput={(e) => props.updateFormField('username', e.currentTarget.value)}
              placeholder={isAdd() ? 'user@example.com' : ''}
            />
          </div>

          <div>
            <label class={labelClass}>Password</label>
            <input
              type="password"
              class={inputClass}
              value={props.formData().password || ''}
              onInput={(e) => props.updateFormField('password', e.currentTarget.value)}
              placeholder={isAdd() ? 'Enter password' : 'Leave empty to keep current'}
            />
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class={labelClass}>Sender Email</label>
              <input
                type="email"
                class={inputClass}
                value={props.formData().senderEmail || ''}
                onInput={(e) => props.updateFormField('senderEmail', e.currentTarget.value)}
                placeholder={isAdd() ? 'noreply@example.com' : ''}
              />
            </div>
            <div>
              <label class={labelClass}>Sender Name</label>
              <input
                type="text"
                class={inputClass}
                value={props.formData().senderName || 'NetBOT'}
                onInput={(e) => props.updateFormField('senderName', e.currentTarget.value)}
              />
            </div>
          </div>

          <div class="flex items-center justify-between px-3 py-2 rounded-[3px] border border-[#3c3c3c] bg-[#1e1e1e]">
            <div class="space-y-0.5">
              <span class="text-[11px] text-[#cccccc] font-medium">Use TLS</span>
              <p class="text-[10px] text-[#808080]">Encrypt connection with TLS</p>
            </div>
            <Switch
              checked={props.formData().useTls ?? true}
              onChange={(checked) => props.updateFormField('useTls', checked)}
            />
          </div>
        </div>

        <DialogFooter>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc] hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors"
            onClick={() => props.onOpenChange(false)}
          >
            Cancel
          </button>
          <button
            type="button"
            class="h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            onClick={props.onSubmit}
            disabled={props.isLoading()}
          >
            {props.isLoading() ? (
              <>
                <svg class="inline-block w-3 h-3 mr-1.5 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Saving...
              </>
            ) : (
              isAdd() ? 'Add Server' : 'Save Changes'
            )}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
