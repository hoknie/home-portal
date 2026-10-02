import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

const RAW_ELEMENTS = "button|input|select|textarea|table|thead|tbody|tr|th|td|h1|h2|h3|h4|h5|h6";

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  {
    files: ["src/**/*.tsx"],
    ignores: ["src/**/*.test.tsx"],
    rules: {
      "react/jsx-no-literals": ["error", { noStrings: true, ignoreProps: true, noAttributeStrings: false }],
    },
  },
  {
    files: ["src/**/*.tsx", "src/**/*.ts"],
    ignores: ["src/shared/ui/kit/**", "src/**/*.test.ts", "src/**/*.test.tsx"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: `JSXOpeningElement[name.name=/^(${RAW_ELEMENTS})$/]`,
          message: "Take this element from @/shared/ui/kit instead of rendering it raw.",
        },
      ],
      "no-restricted-imports": [
        "error",
        { paths: [{ name: "radix-ui", message: "Radix is used only inside @/shared/ui/kit." }], patterns: [{ group: ["@radix-ui/*"], message: "Radix is used only inside @/shared/ui/kit." }] },
      ],
    },
  },
  globalIgnores([".next/**", "out/**", "build/**", "next-env.d.ts", "coverage/**"]),
]);

export default eslintConfig;
