/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["src/**/*.rs", "index.html"],
  theme: {
    extend: {},
  },
  plugins: [require("@tailwindcss/forms"), require("daisyui")],
};
