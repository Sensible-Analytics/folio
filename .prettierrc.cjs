module.exports = {
  // Core formatting options
  printWidth: 100,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  quoteProps: "as-needed",
  trailingComma: "all",

  // JSX/React specific
  jsxSingleQuote: false,
  bracketSpacing: true,
  bracketSameLine: false,
  singleAttributePerLine: false,

  // Arrow functions
  arrowParens: "always",

  // Line endings and whitespace
  endOfLine: "lf",
  insertPragma: false,
  requirePragma: false,
  proseWrap: "always",
  htmlWhitespaceSensitivity: "css",
  embeddedLanguageFormatting: "auto",

  // File-specific overrides
  overrides: [
    {
      files: ["**/*.json", "**/*.jsonc"],
      options: {
        tabWidth: 2,
        useTabs: false,
      },
    },
    {
      files: ["**/*.md", "**/*.mdx"],
      options: {
        proseWrap: "always",
        printWidth: 80,
      },
    },
    {
      files: ["**/*.yml", "**/*.yaml"],
      options: {
        tabWidth: 2,
        singleQuote: false,
      },
    },
    {
      files: ["packages/ui/src/components/**/*.tsx"],
      options: {
        printWidth: 120,
      },
    },
    {
      files: ["packages/addon-sdk/src/**/*.ts"],
      options: {
        printWidth: 90,
        singleQuote: true,
      },
    },
  ],

  // Plugins (Tailwind CSS plugin for class sorting)
  plugins: ["prettier-plugin-tailwindcss"],
};
