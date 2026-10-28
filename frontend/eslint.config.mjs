import coreWebVitals from "eslint-config-next/core-web-vitals";

// `next lint` was removed in Next 16; the shared config is a flat-config array and ESLint
// runs directly against it.
export default [
  ...coreWebVitals,
  {
    rules: {
      // Pre-existing provider/shell patterns that React's compiler rules flag. Kept visible as
      // warnings rather than silenced: they are real, and the compiler migration removes them.
      "react-hooks/set-state-in-effect": "warn",
    },
  },
  {
    ignores: [".next/**", "node_modules/**", "next-env.d.ts"],
  },
];
