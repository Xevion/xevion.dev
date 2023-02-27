/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/**/*.{js,ts,jsx,tsx}"],
    theme: {
        extend: {
            colors: {
              zinc: {
                  850: "#1D1D20"
              }
            },
            fontFamily: {
                manrope: ["\"Manrope\"", "sans-serif"],
                inter: ["\"Inter\"", "sans-serif"],
                roboto: ["\"Roboto\"", "sans-serif"],
                mono: ["\"Roboto Mono\"", "monospace"],
                hanken: ["\"Hanken Grotesk\"", "sans-serif"],
            },
        },
    },
plugins: [require('@tailwindcss/typography')],
};
