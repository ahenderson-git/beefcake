import { describe, test, expect } from 'vitest';

import { fmtBytes, fmtDuration, escapeHtml } from './utils';

describe('utils', () => {
  describe('fmtBytes', () => {
    test('should format zero bytes', () => {
      expect(fmtBytes(0)).toBe('0 B');
    });

    test('should format bytes correctly', () => {
      expect(fmtBytes(100)).toBe('100 B');
      expect(fmtBytes(500)).toBe('500 B');
    });

    test('should format kilobytes correctly', () => {
      expect(fmtBytes(1024)).toBe('1 KB');
      expect(fmtBytes(2048)).toBe('2 KB');
      expect(fmtBytes(1536)).toBe('1.5 KB');
    });

    test('should format megabytes correctly', () => {
      expect(fmtBytes(1048576)).toBe('1 MB');
      expect(fmtBytes(5242880)).toBe('5 MB');
    });

    test('should format gigabytes correctly', () => {
      expect(fmtBytes(1073741824)).toBe('1 GB');
      expect(fmtBytes(2147483648)).toBe('2 GB');
    });

    test('should format terabytes correctly', () => {
      expect(fmtBytes(1099511627776)).toBe('1 TB');
    });

    test('should handle decimal values correctly', () => {
      expect(fmtBytes(1536)).toBe('1.5 KB');
      expect(fmtBytes(2621440)).toBe('2.5 MB');
    });
  });

  describe('fmtDuration', () => {
    test('should format milliseconds correctly', () => {
      const duration = { secs: 0, nanos: 500000000 };
      expect(fmtDuration(duration)).toBe('500.00ms');
    });

    test('should format milliseconds under 1 second', () => {
      const duration = { secs: 0, nanos: 123456789 };
      expect(fmtDuration(duration)).toBe('123.46ms');
    });

    test('should format seconds correctly', () => {
      const duration = { secs: 5, nanos: 0 };
      expect(fmtDuration(duration)).toBe('5.00s');
    });

    test('should format seconds with milliseconds', () => {
      const duration = { secs: 5, nanos: 500000000 };
      expect(fmtDuration(duration)).toBe('5.50s');
    });

    test('should handle zero duration', () => {
      const duration = { secs: 0, nanos: 0 };
      expect(fmtDuration(duration)).toBe('0.00ms');
    });

    test('should format large durations', () => {
      const duration = { secs: 123, nanos: 456000000 };
      expect(fmtDuration(duration)).toBe('123.46s');
    });
  });

  describe('escapeHtml', () => {
    test('should escape ampersands', () => {
      expect(escapeHtml('A & B')).toBe('A &amp; B');
    });

    test('should escape less than signs', () => {
      expect(escapeHtml('5 < 10')).toBe('5 &lt; 10');
    });

    test('should escape greater than signs', () => {
      expect(escapeHtml('10 > 5')).toBe('10 &gt; 5');
    });

    test('should escape double quotes', () => {
      expect(escapeHtml('Say "hello"')).toBe('Say &quot;hello&quot;');
    });

    test('should escape single quotes', () => {
      expect(escapeHtml("It's working")).toBe('It&#039;s working');
    });

    test('should escape all special characters together', () => {
      expect(escapeHtml('<script>alert("XSS & stuff\'s")</script>')).toBe(
        '&lt;script&gt;alert(&quot;XSS &amp; stuff&#039;s&quot;)&lt;/script&gt;'
      );
    });

    test('should handle empty string', () => {
      expect(escapeHtml('')).toBe('');
    });

    test('should handle plain text without special characters', () => {
      expect(escapeHtml('Hello World')).toBe('Hello World');
    });

    test('should prevent XSS attacks', () => {
      const maliciousInput = '<' + 'script' + '>alert(1)<' + '/' + 'script' + '>';
      const escaped = escapeHtml(maliciousInput);
      // Verify no actual HTML tags remain
      expect(escaped).not.toContain('<script');
      // Verify the dangerous characters are escaped
      expect(escaped).toContain('&lt;');
      expect(escaped).toContain('&gt;');
      // Verify the full expected output
      expect(escaped).toBe('&lt;script&gt;alert(1)&lt;/script&gt;');
    });
  });
});
