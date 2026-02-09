/**
 * Simple logging utility for frontend
 * Provides structured logging with namespaces
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

// Check if we're in development mode (always log in dev, suppress in production)
// In Vite/Tauri production builds, this will be tree-shaken away
const IS_DEV = process.env.NODE_ENV !== 'production';

class Logger {
  private namespace: string;

  constructor(namespace: string) {
    this.namespace = namespace;
  }

  private log(level: LogLevel, message: string, data?: unknown): void {
    // Always log to console in development
    // In production builds, this gets stripped by bundler
    if (IS_DEV) {
      const prefix = `[${this.namespace}]`;
      switch (level) {
        case 'debug':
          // eslint-disable-next-line no-console
          console.log(prefix, message, data ?? '');
          break;
        case 'info':
          // eslint-disable-next-line no-console
          console.info(prefix, message, data ?? '');
          break;
        case 'warn':
          // eslint-disable-next-line no-console
          console.warn(prefix, message, data ?? '');
          break;
        case 'error':
          // eslint-disable-next-line no-console
          console.error(prefix, message, data ?? '');
          break;
      }
    }

    // In production, errors could be sent to backend logging service
    // Future: await invoke('log_frontend_error', { timestamp, level, namespace, message, data });
  }

  debug(message: string, data?: unknown): void {
    this.log('debug', message, data);
  }

  info(message: string, data?: unknown): void {
    this.log('info', message, data);
  }

  warn(message: string, data?: unknown): void {
    this.log('warn', message, data);
  }

  error(message: string, data?: unknown): void {
    this.log('error', message, data);
  }
}

/**
 * Create a logger instance for a specific namespace
 */
export function createLogger(namespace: string): Logger {
  return new Logger(namespace);
}
