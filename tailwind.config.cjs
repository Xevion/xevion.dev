/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        manrope: ["\"Manrope\"", "sans-serif"],
        opensans: ["\"Open Sans\"", "sans-serif"],
        inter: ["\"Inter\"", "sans-serif"],
        mono: ["\"Roboto Mono\"", "monospace"],
        raleway: ["\"Raleway\"", "sans-serif"],
        roboto: ["\"Roboto\""],
      },
    },
  },
  plugins: [],
};
