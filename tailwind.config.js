/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: ["./src/**/*.rs"],
  },
  theme: {
    extend: {},
  },
  plugins: [
      require("daisyui"),
      // require('@tailwindcss/typography'),
  ],
}

