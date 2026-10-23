/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Strava-grade palette.
        brand: {
          DEFAULT: '#fc5200',
          50: '#fff4ef',
          100: '#ffe3d6',
          200: '#ffc7ab',
          300: '#ffa077',
          400: '#ff7943',
          500: '#fc5200',
          600: '#e04700',
          700: '#b83a00',
          800: '#8f2e00',
          900: '#6b2300',
        },
        ink: {
          DEFAULT: '#121212',
          soft: '#3c4043',
          muted: '#6d7176',
          faint: '#9aa0a6',
        },
        surface: {
          DEFAULT: '#ffffff',
          soft: '#f7f7f7',
          raised: '#fbfbfb',
          sunken: '#f0f0f0',
        },
        edge: '#e6e6e6',
        positive: '#1a9c5a',
        negative: '#d93a2b',
        warn: '#e6a700',
        // Chart ramp: orange -> magenta, as Strava uses for activity plots.
        ramp: ['#fc5200', '#fd6f31', '#f58a58', '#e9a08a', '#d977a6', '#c14fb0'],
      },
      fontFamily: {
        sans: [
          'Inter',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          'Helvetica Neue',
          'Arial',
          'sans-serif',
        ],
        mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'Consolas', 'monospace'],
      },
      boxShadow: {
        card: '0 1px 2px rgba(0,0,0,0.04), 0 1px 3px rgba(0,0,0,0.06)',
        lift: '0 4px 16px rgba(0,0,0,0.10)',
        nav: '0 1px 0 rgba(0,0,0,0.08)',
      },
      borderRadius: {
        card: '10px',
      },
      keyframes: {
        'fade-up': {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        shimmer: {
          '100%': { transform: 'translateX(100%)' },
        },
      },
      animation: {
        'fade-up': 'fade-up 0.28s ease-out both',
      },
    },
  },
  plugins: [],
}
