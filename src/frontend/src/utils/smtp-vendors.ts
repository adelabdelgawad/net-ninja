import type { SmtpConfigCreate, SmtpVendor } from '~/types';

export { type SmtpVendor };

export interface VendorDefaults {
  host: string;
  port: number;
  useTls: boolean;
  displayName: string;
  description: string;
  hostEditable: boolean;
}

export const VENDOR_CONFIGS: Record<SmtpVendor, VendorDefaults> = {
  gmail: {
    host: 'smtp.gmail.com',
    port: 465,
    useTls: true,
    displayName: 'Gmail',
    description: 'Google Gmail SMTP (requires App Password) - Fully Supported',
    hostEditable: false,
  },
  exchange: {
    host: '',
    port: 587,
    useTls: true,
    displayName: 'Microsoft Exchange',
    description: 'Microsoft Exchange Server - Requires basic auth enabled on server',
    hostEditable: true,
  },
  outlook365: {
    host: 'smtp.office365.com',
    port: 587,
    useTls: true,
    displayName: 'Outlook 365',
    description: 'Office 365 / Outlook.com - Requires App Password (OAuth2 not yet supported)',
    hostEditable: false,
  },
};

export function applyVendorDefaults(
  vendor: SmtpVendor,
  current: Partial<SmtpConfigCreate>
): Partial<SmtpConfigCreate> {
  const defaults = VENDOR_CONFIGS[vendor];
  return {
    ...current,
    vendor,
    host: defaults.host || current.host || '',
    port: defaults.port,
    useTls: defaults.useTls,
  };
}
