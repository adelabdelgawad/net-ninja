// Log types

export interface Log {
  id: number;
  processId: string | null;
  function: string | null;
  level: string | null;
  message: string | null;
  lineId: number | null;
  timestamp: string;
}
