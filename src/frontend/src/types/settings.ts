// Settings types

// Fallback mode types
export type InitMode = 'Full' | 'Fallback';

export interface FallbackStatusResponse {
  is_fallback: boolean;
  init_mode: InitMode;
  error: string | null;
}
