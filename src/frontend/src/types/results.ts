// Quota and Speed Test Result types

export interface QuotaResult {
  id: number;
  lineId: number;
  processId: string;
  balance: number | null;
  quotaPercentage: number | null;
  usedQuota: number | null;
  totalQuota: number | null;
  remainingQuota: number | null;
  renewalDate: string | null;
  renewalCost: number | null;
  extraQuota: number | null;
  status: string | null;
  message: string | null;
  createdAt: string;
}

export interface SpeedTestResult {
  id: number;
  lineId: number;
  processId: string;
  downloadSpeed: number | null;
  uploadSpeed: number | null;
  ping: number | null;
  serverName: string | null;
  serverLocation: string | null;
  publicIp: string | null;
  status: string | null;
  errorMessage: string | null;
  createdAt: string;
}
