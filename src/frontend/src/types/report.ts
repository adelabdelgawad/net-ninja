// Report types

export interface CombinedResult {
  lineId: number;
  lineNumber: string;
  name: string;
  isp: string;
  description: string;
  download: number | null;
  upload: number | null;
  ping: number | null;
  dataUsed: number | null;
  usagePercentage: number | null;
  dataRemaining: number | null;
  renewalDate: string | null;
  balance: number | null;
  lastUpdated?: string;
}
