/**
 * Network diagnostics report for speedtest.net connectivity
 */
export interface DiagnosticReport {
  /** Target host that was tested */
  host: string;

  /** Whether DNS resolution succeeded */
  dns_resolved: boolean;

  /** IP addresses resolved from DNS */
  dns_ips: string[];

  /** Whether TCP connection to port 443 succeeded */
  tcp_reachable: boolean;

  /** TCP connection latency in milliseconds (if successful) */
  tcp_latency_ms?: number;

  /** Proxy environment variable detected (if any) */
  proxy_detected?: string;

  /** List of errors encountered during diagnostics */
  errors: string[];
}
