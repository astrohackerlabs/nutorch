import base from "../../../prettier.config.mjs";
export default {
  ...base,
  plugins: ["prettier-plugin-tailwindcss"],
  tailwindStylesheet: "./app/styles/global.css",
  tailwindFunctions: ["cn", "cva"],
};
