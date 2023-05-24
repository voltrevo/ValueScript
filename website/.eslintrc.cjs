// eslint-disable-next-line no-undef
module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  extends: [
    "eslint:recommended",
    "plugin:react/recommended",
    "plugin:@typescript-eslint/recommended",
    "prettier",
  ],
  overrides: [],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
  },
  plugins: ["react", "@typescript-eslint", "prettier"],
  settings: {
    react: {
      version: "detect",
    },
  },
  rules: {
    "linebreak-style": ["error", "unix"],
    "quotes": ["error", "double"],
    "semi": ["error", "always"],
    "comma-dangle": ["error", "always-multiline"],
    "react/react-in-jsx-scope": "off",
    "quote-props": ["error", "consistent-as-needed"],
    "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
    "prettier/prettier": "error",
    "no-constant-condition": "off",
    "no-empty": ["error", { allowEmptyCatch: true }],
  },
};
