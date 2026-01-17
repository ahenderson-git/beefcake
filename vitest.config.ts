import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'happy-dom',
    globals: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'dist/',
        'test/',
        '**/*.config.ts',
        '**/*.d.ts',
      ],
    },
    include: ['src-frontend/**/*.{test,spec}.ts'],
    exclude: ['node_modules', 'dist', 'build'],
  },
});
