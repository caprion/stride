/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Brand — deep azure. Structure, gravity, primary actions, the frame.
        brand: {
          50: '#EEF5FB',
          100: '#D6E6F3',
          200: '#A9C9E2',
          300: '#7BA8CD',
          400: '#4A82B4',
          500: '#1E5E96',
          600: '#124A7C',
          700: '#0D3A63',
          800: '#082B49',
          900: '#051D33',
        },
        // Accent — warm amber. Excitement, energy, the athlete's own data.
        accent: {
          50: '#FEF8EA',
          100: '#FDEDC7',
          200: '#FBDB93',
          300: '#F8C55C',
          400: '#F2B01E',
          500: '#D99A0B',
          600: '#B57C06',
          700: '#8A5E05',
          800: '#5F4103',
          900: '#3D2A02',
        },
        // Ink — a blue-black, so text sits in the same world as the chrome.
        ink: {
          DEFAULT: '#0B1B27',
          soft: '#31465A',
          muted: '#61788D',
          faint: '#93A6B7',
        },
        surface: {
          DEFAULT: '#FFFFFF',
          soft: '#F4F7FA',
          raised: '#FAFCFD',
          sunken: '#E9EFF4',
        },
        edge: '#DEE7EE',
        positive: '#2E8B7A',
        negative: '#C0392B',
        warn: '#D99A0B',
        // Chart ramp: sun through earth to deep water.
        ramp: ['#F2B01E', '#E08A3C', '#B4557F', '#6E6BA8', '#3F6E9E', '#1E5E96'],
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
        card: '0 1px 2px rgba(11,27,39,0.04), 0 1px 3px rgba(11,27,39,0.06)',
        lift: '0 4px 16px rgba(11,27,39,0.10)',
        nav: '0 1px 0 rgba(11,27,39,0.08)',
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
