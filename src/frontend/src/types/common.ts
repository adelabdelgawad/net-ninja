// Common types and utilities

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface PaginationParams {
  page?: number;
  pageSize?: number;
}

export interface ResultFilterParams extends PaginationParams {
  lineId?: number;
  startDate?: string;
  endDate?: string;
}

export interface LogFilterParams extends PaginationParams {
  level?: string;
  function?: string;
  startDate?: string;
  endDate?: string;
}

export interface MessageResponse {
  message: string;
}

export interface ErrorResponse {
  detail: string;
  code?: string;
}
