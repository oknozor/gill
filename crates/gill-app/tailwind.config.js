/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        '**/*.{html,js,css}',
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
        },
        extend: {
            typography(theme) {
                return {
                    slate: {
                        css: {
                            '--tw-prose-bullets': theme('colors.slate[900]'),
                            '--tw-prose-pre-bg': theme('colors.slate[100]'),
                            '--tw-prose-pre-code': theme('colors.slate[900]'),
                            'p': {
                                'line-height': 1.5
                            },
                            'a': {
                                color: theme('colors.blue[600]'),
                                'text-decoration': 'none',
                            },
                            'a:hover': {
                                'text-decoration': 'underline',
                            },
                            'code::before': {
                                content: 'none',
                            },
                            'code::after': {
                                content: 'none'
                            },
                            'img': {
                                'display': 'inline-block',
                                'margin-top': '0em',
                                'margin-bottom': '0.25em',
                            },
                            'h1, h2': {
                                'padding-bottom': '20px',
                                'border-bottom-width': '1px',
                                'border-color': theme('colors.slate.300'),
                            },
                            'h1, h2, h3, h4': {
                                'margin-top': '0',
                            },
                            'li': {
                                'margin-top': '0.25em',
                                'margin-bottom': '0.25em',
                            },
                            'li *': {
                                'margin-top': 0,
                                'margin-bottom': 0,
                            },
                            ':not(pre) > code': {
                                color: theme('colors.slate.700'),
                                backgroundColor: theme('colors.stone.100'),
                                borderRadius: theme('borderRadius.DEFAULT'),
                                paddingLeft: theme('spacing[1]'),
                                paddingRight: theme('spacing[1]'),
                                paddingTop: '3px',
                                paddingBottom: '3px',
                            },
                        },
                    }
                }
            }
        }
    }
}
