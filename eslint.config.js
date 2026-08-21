import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'

/**
 * Type-aware linting (recommendedTypeChecked) — the TypeScript checker
 * feeds ESLint, so rules like no-floating-promises / no-misused-promises
 * run with full type information. Project service is scoped to `src` so
 * tooling configs (vite/tsconfig.node) stay out of the lint graph.
 * لینت آگاه به نوع (recommendedTypeChecked) — چک‌کنندهٔ TypeScript به
 * ESLint خوراک می‌دهد تا قوانینی مانند no-floating-promises با اطلاعات
 * کامل نوع اجرا شوند. سرویس پروژه به `src` محدود است تا کانفیگ ابزارها
 * (vite/tsconfig.node) خارج از گراف لینت بماند.
 */
export default tseslint.config(
  { ignores: ['dist', 'coverage', 'src-tauri'] },
  {
    // Root-level build config is not part of the app's `tsconfig.json`
    // project; lint it with the non-type-aware rule set instead.
    // کانفیگ‌های سطح ریشه عضو پروژهٔ `tsconfig.json` برنامه نیستند؛ با
    // مجموعه‌قوانین غیر type-aware لینت می‌شوند.
    files: ['vite.config.ts'],
    extends: [js.configs.recommended, tseslint.configs.disableTypeChecked],
    languageOptions: {
      ecmaVersion: 2020,
      globals: { ...globals.browser, ...globals.node },
    },
  },
  {
    extends: [
      js.configs.recommended,
      ...tseslint.configs.recommendedTypeChecked,
    ],
    // Build config lives outside the `tsconfig.json` project; it is linted
    // by the dedicated non-type-aware block above.
    // کانفیگ بیلد خارج از پروژهٔ `tsconfig.json` است و در بلاک غیر
    // type-aware بالا لینت می‌شود.
    ignores: ['vite.config.ts'],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        project: ['./tsconfig.json'],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true },
      ],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      // Test files deliberately use non-null assertions on mock arguments;
      // banning them globally would only add noise to already-covered code.
      // فایل‌های تست آگاهانه از assertion غیر-null روی آرگومان‌های mock
      // استفاده می‌کنند؛ ممنوعیت سراسری فقط نویز اضافه می‌کند.
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  }
)
