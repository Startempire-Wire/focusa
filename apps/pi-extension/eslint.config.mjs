import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import prettier from "eslint-config-prettier";

export default [
  {
    ignores: ["node_modules/**", "*.d.ts", "dist/**", ".beads/**"],
  },
  {
    files: ["src/**/*.ts"],
    languageOptions: {
      parser: tsParser,
      parserOptions: { ecmaVersion: 2024, sourceType: "module" },
      globals: {
        process: "readonly",
        console: "readonly",
        Buffer: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        global: "readonly",
      },
    },
    plugins: { "@typescript-eslint": tsPlugin },
    rules: {
      // Permissive on pre-existing style; tighten over time.
      "no-unused-vars": "off",
      // Root-cause fix: `vars: "local"` makes @typescript-eslint/no-unused-vars
      // skip EXPORTED variables (which are public API for cross-file consumers).
      // Without this, every `export function foo()` in state.ts was flagged as
      // "defined but never used" because the rule's default `vars: "all"`
      // checks exports too. State.ts has 120+ exports consumed by other modules.
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          vars: "never",
          args: "after-used",
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-non-null-assertion": "off",
      "no-empty": ["error", { allowEmptyCatch: true }],
      "no-constant-condition": ["error", { checkLoops: false }],
      eqeqeq: ["error", "smart"],
      "prefer-const": "error",
      "no-var": "error",
    },
  },
  prettier,
];
