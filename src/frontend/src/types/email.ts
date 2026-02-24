// Email and SMTP Configuration types

export type SmtpVendor = 'gmail' | 'exchange' | 'outlook365';

export interface Email {
  id: number;
  recipient: string;
  name: string | null;
  isCc: boolean;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface EmailCreate {
  recipient: string;
  name?: string;
  isCc?: boolean;
  isActive?: boolean;
}

export interface EmailUpdate {
  recipient?: string;
  name?: string;
  isCc?: boolean;
  isActive?: boolean;
}

export interface SmtpConfig {
  id: number;
  name: string;
  host: string;
  port: number;
  username: string;
  senderEmail: string;
  senderName: string;
  useTls: boolean;
  isDefault: boolean;
  isActive: boolean;
  vendor: SmtpVendor;
  createdAt: string;
  updatedAt: string;
}

export interface SmtpConfigCreate {
  name: string;
  host: string;
  port: number;
  username: string;
  password: string;
  senderEmail: string;
  senderName: string;
  useTls: boolean;
  vendor: SmtpVendor;
}

export interface SmtpConfigUpdate {
  name?: string;
  host?: string;
  port?: number;
  username?: string;
  password?: string;
  senderEmail?: string;
  senderName?: string;
  useTls?: boolean;
  vendor?: SmtpVendor;
}

export interface SmtpConfigTestRequest {
  host: string;
  port: number;
  username: string;
  password: string;
  senderEmail: string;
  senderName: string;
  useTls: boolean;
  testRecipient: string;
}

export interface SmtpConfigTestResponse {
  success: boolean;
  message: string;
}
