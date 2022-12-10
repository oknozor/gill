/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
      '**/*.{html,js}',
  ],
  plugins: [
    require('@tailwindcss/typography'),
    require('@tailwindcss/forms'),
    require('@tailwindcss/line-clamp'),
    require('@tailwindcss/aspect-ratio'),
  ],
  theme: {
    fontFamily: {
      'body': ['"Open Sans"'],
    }
  }
}