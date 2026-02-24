// Line types

export interface Line {
  id: number;
  lineNumber: string;
  name: string;
  portalUsername: string;
  portalPassword: string;
  description: string | null;
  isp: string | null;
  ipAddress: string | null;
  gatewayIp: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface LineCreate {
  lineNumber: string;
  name: string;
  description: string;
  isp: string;
  ipAddress: string;
  gatewayIp: string;
  portalUsername: string;
  portalPassword: string;
}

export interface LineUpdate extends Partial<LineCreate> {
  isActive?: boolean;
}
