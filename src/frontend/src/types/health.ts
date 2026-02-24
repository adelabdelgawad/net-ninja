// Health check types

export interface HealthResponse {
  status: string;
  databaseConnected: boolean;
  databasePath: string;
  initMode: string;
}

export interface ReadinessResponse {
  status: string;
  database: string;
}
