import base from "../../../oxfmt.config.mjs";
export default {
  ...base,
  sortTailwindcss: {
    stylesheet: "./app/styles/global.css",
    functions: ["cn", "cva"],
  },
};
