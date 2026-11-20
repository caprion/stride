/**
 * The design tokens — the one source for the whole app.
 *
 * **Everything that is a colour, a size or a shadow is here, and nowhere
 * else.** The Tailwind theme is generated from this file; the phone's own
 * chrome is generated from this file; the palette the athlete chose is one
 * attribute on the root element, and this file holds every palette it can
 * be set to.
 *
 * Three palettes live here, not one set of colours:
 *
 *   * `stride` — today's: the deep azure, the warm amber, the white cards.
 *   * `dreeve` — the one the athlete prefers: a saturated orange-red
 *     accent, a warm ground, a hairline edge, the ink `#111827`, the muted
 *     `#6b7280`, and white cards.
 *   * `night` — the dark variant.
 *
 * The values came from the prototype (`stride-mocks/mock.css`), which
 * carried them as CSS custom properties; the dreeve ones were sampled from
 * the running instance on this machine, so they are the real ones.
 *
 * ## The rules the file holds
 *
 *   * **A component never names a colour.** It names a token — a Tailwind
 *     class (`bg-brand-600`) or a CSS variable (`var(--chart-line)`). The
 *     gate scans the app's source for raw hex values and names the
 *     offenders.
 *   * **A palette is a whole set of roles, not a hue.** Every palette
 *     answers every role, so a swap can never leave a screen half
 *     painted.
 *   * **Nothing here is a verdict.** `better`, `usual` and `could` are the
 *     three directions the app may say, and there is no "good", no "bad",
 *     no score and no grade anywhere in this file — because there is none
 *     anywhere in the app.
 *
 * ## The ramps
 *
 * A ramp runs from the faintest trace of its colour to its strongest
 * expression: `50` is the faintest, `900` the strongest. On the light
 * ground that means a pale tint at `50` and a deep blue at `900`; on the
 * dark ground it is the other way round — `50` is a tint that barely
 * lifts off the page, and `900` is a pale blue.
 *
 * One step is deliberate in every palette: `brand.600` is the fill that
 * carries white text, so it stays dark enough to carry it.
 */

/** A ramp: ten steps, from the faintest trace to the deepest. */
export interface Ramp {
  50: string
  100: string
  200: string
  300: string
  400: string
  500: string
  600: string
  700: string
  800: string
  900: string
}

/** The inks: the four weights of text the app writes in. */
export interface Ink {
  DEFAULT: string
  soft: string
  muted: string
  faint: string
}

/** The surfaces: the four planes the app is drawn on. */
export interface Surface {
  DEFAULT: string
  soft: string
  raised: string
  sunken: string
}

/**
 * The chart's own colours.
 *
 * A chart is not a card with a line on it: it has a grid behind it, a band
 * the athlete's usual is measured against, a series, and a marker. Those
 * are roles, and they are tokens, so a palette can answer them without a
 * component being rewritten.
 */
export interface Chart {
  /** The grid lines behind a plot. */
  grid: string
  /** The axis labels, and the quiet guide lines beside them. */
  axis: string
  /** The band behind a line: the athlete's usual, or a zone. */
  band: string
  /** The series itself. */
  line: string
  /** The second series, or the quiet line that accompanies the first. */
  lineAlt: string
  /** The marker: the selected date, the pace beside the heart rate. */
  reference: string
}

/** A whole set of roles: one palette. */
export interface Palette {
  /** The azure ramp in `stride`; the orange-red ramp in `dreeve`. */
  brand: Ramp
  /** The warm ramp: the athlete's own data, the accent of the app. */
  accent: Ramp
  ink: Ink
  surface: Surface
  /** The hairline between two things. */
  edge: string
  /** The three directions the app may say. Never red, never a grade. */
  better: string
  usual: string
  could: string
  /** The series ramp: six steps, for a chart that draws more than one. */
  ramp: string[]
  chart: Chart
}

/** The palettes the app can be drawn in. */
export type PaletteName = 'stride' | 'dreeve' | 'night'

/**
 * The parts of a palette that are not derived.
 *
 * The chart's colours are derived from the ramps — a chart's series is the
 * brand's own colour, its band is the brand's faintest tint — except for
 * the two that cannot be: the grid and the axis, which are the quietest
 * marks on the plot and belong to the ground, not to the brand.
 */
interface PaletteInput {
  brand: Ramp
  accent: Ramp
  ink: Ink
  surface: Surface
  edge: string
  better: string
  usual: string
  could: string
  ramp: string[]
  chart: { grid: string; axis: string }
}

/**
 * A palette, with its chart colours derived from its own ramps.
 *
 * Deriving them is the point: a chart cannot be a colour the palette does
 * not have, so a new palette cannot come out half painted.
 */
function palette(input: PaletteInput): Palette {
  return {
    brand: input.brand,
    accent: input.accent,
    ink: input.ink,
    surface: input.surface,
    edge: input.edge,
    better: input.better,
    usual: input.usual,
    could: input.could,
    ramp: input.ramp,
    chart: {
      grid: input.chart.grid,
      axis: input.chart.axis,
      band: input.brand[100],
      line: input.brand[500],
      lineAlt: input.brand[200],
      reference: input.accent[500],
    },
  }
}

/**
 * `stride` — today's colours.
 *
 * The deep azure for the structure, the warm amber for the athlete's own
 * data, and the white cards on a cool ground.
 */
export const stride: Palette = palette({
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
  ink: {
    DEFAULT: '#112B40',
    soft: '#31465A',
    muted: '#4E6577',
    faint: '#586F82',
  },
  surface: {
    DEFAULT: '#FFFFFF',
    soft: '#F1F5F8',
    raised: '#FAFCFD',
    sunken: '#E9EFF4',
  },
  edge: '#CEDAE4',
  better: '#2E8B7A',
  usual: '#4A82B4',
  could: '#8A5E05',
  ramp: ['#F2B01E', '#E08A3C', '#B4557F', '#6E6BA8', '#3F6E9E', '#1E5E96'],
  chart: {
    grid: '#E4EBF0',
    axis: '#61788D',
  },
})

/**
 * `dreeve` — the palette the athlete prefers.
 *
 * The saturated orange-red accent `#fc5200`, the warm ground `#f9fafb`,
 * the hairline edge `#e5e7eb`, the ink `#111827`, the muted `#6b7280`, and
 * white cards. The navigation is white with an orange current page, which
 * is what `brand.50` and `brand.700` are for.
 */
export const dreeve: Palette = palette({
  brand: {
    50: '#FFF6F2',
    100: '#FFF1E9',
    200: '#FFD9C9',
    300: '#FFB99B',
    400: '#FF8A5C',
    500: '#FC5200',
    600: '#D03F00',
    700: '#C23C00',
    800: '#9E3100',
    900: '#7A2600',
  },
  accent: {
    50: '#FDF6F3',
    100: '#FDF3EE',
    200: '#FCEFE8',
    300: '#F0CFC2',
    400: '#DE8A72',
    500: '#BD2D22',
    600: '#A32319',
    700: '#942319',
    800: '#6D1810',
    900: '#4A100B',
  },
  ink: {
    DEFAULT: '#111827',
    soft: '#374151',
    muted: '#6B7280',
    faint: '#9CA3AF',
  },
  surface: {
    DEFAULT: '#FFFFFF',
    soft: '#F9FAFB',
    raised: '#FCFCFD',
    sunken: '#F3F4F6',
  },
  edge: '#E5E7EB',
  better: '#15803D',
  usual: '#6B7280',
  could: '#B45309',
  ramp: ['#FC5200', '#BD2D22', '#6B7280', '#374151', '#942319', '#9CA3AF'],
  chart: {
    grid: '#ECEEF0',
    axis: '#6B7280',
  },
})

/**
 * `night` — the dark variant.
 *
 * The same roles on a dark ground: the azure becomes a light blue, the
 * amber a bright gold, the ink a pale grey, and the cards a deep blue
 * rather than white.
 */
export const night: Palette = palette({
  brand: {
    50: '#17293F',
    100: '#1E3A5F',
    200: '#24486F',
    300: '#2F5C8F',
    400: '#3B82F6',
    500: '#60A5FA',
    600: '#2F6FE0',
    700: '#93C5FD',
    800: '#BFDBFE',
    900: '#DBEAFE',
  },
  accent: {
    50: '#2A2312',
    100: '#4A3A13',
    200: '#6B5524',
    300: '#8F7434',
    400: '#C08A1E',
    500: '#FBBF24',
    600: '#FCD34D',
    700: '#FDE68A',
    800: '#FEF3C7',
    900: '#FFFBEB',
  },
  ink: {
    DEFAULT: '#E8EEF6',
    soft: '#9FB0C4',
    muted: '#8497AC',
    faint: '#7A8EA4',
  },
  surface: {
    DEFAULT: '#14202F',
    soft: '#0D1724',
    raised: '#1B2839',
    sunken: '#0F1A28',
  },
  edge: '#24344A',
  better: '#34D399',
  usual: '#94A3B8',
  could: '#FBBF24',
  ramp: ['#60A5FA', '#FBBF24', '#2DD4BF', '#A78BFA', '#FB7185', '#94A3B8'],
  chart: {
    grid: '#24344A',
    axis: '#8497AC',
  },
})

/** Every palette, by name. */
export const palettes: Record<PaletteName, Palette> = { stride, dreeve, night }

/** The palette the app is drawn in when the athlete has not chosen one. */
export const DEFAULT_PALETTE: PaletteName = 'stride'

/**
 * The phone's launcher backdrop.
 *
 * The launcher icon is a fixed drawable — it cannot follow a palette — so
 * this is one colour in every palette. It is a token because the phone's
 * own chrome is a surface of this app.
 */
export const launcherBackground = '#0B0F14'

/**
 * The phone's own chrome, and the token each colour comes from.
 *
 * The five colours in `android/.../values/colors.xml` are generated from
 * this map, so the phone's chrome and the web bundle it embeds cannot
 * drift apart.
 */
export const androidColors = {
  /** The surface the screens are drawn on. */
  surface: 'surface.soft',
  /** The ink the native screens are written in. */
  onSurface: 'ink.DEFAULT',
  /** The muted ink of the native screens. */
  muted: 'ink.muted',
  /** The accent colour a link may take. */
  brand: 'brand.600',
} as const

/**
 * The type scale.
 *
 * The size, and the line height that goes with it. The app's own text is
 * small and close — it is read at a run's end, on a phone, in daylight.
 */
export const typeScale = {
  '2xs': { size: '11px', lineHeight: '16px' },
  xs: { size: '12px', lineHeight: '18px' },
  sm: { size: '13px', lineHeight: '19px' },
  base: { size: '14px', lineHeight: '20px' },
  md: { size: '15px', lineHeight: '22px' },
  lg: { size: '16px', lineHeight: '24px' },
  xl: { size: '18px', lineHeight: '26px' },
  '2xl': { size: '22px', lineHeight: '28px' },
  '3xl': { size: '26px', lineHeight: '32px' },
} as const

/**
 * The spacing rhythm.
 *
 * Four pixels, and the half steps between them. The rhythm is what makes
 * two screens look like one app; a number that is not on it is a number
 * somebody chose at random.
 */
export const spacing = {
  0: '0px',
  px: '1px',
  0.5: '2px',
  1: '4px',
  1.5: '6px',
  2: '8px',
  2.5: '10px',
  3: '12px',
  3.5: '14px',
  4: '16px',
  5: '20px',
  6: '24px',
  7: '28px',
  8: '32px',
  9: '36px',
  10: '40px',
  11: '44px',
  12: '48px',
  14: '56px',
  16: '64px',
  20: '80px',
  24: '96px',
} as const

/**
 * The radii.
 *
 * The card is the app's own: nine pixels, soft enough to sit on a page and
 * square enough to stack.
 */
export const radii = {
  none: '0px',
  sm: '2px',
  md: '6px',
  lg: '8px',
  card: '9px',
  xl: '12px',
  '2xl': '16px',
  '3xl': '24px',
  full: '9999px',
} as const

/**
 * The shadows.
 *
 * The app is flat by design: a shadow says "this is above that", and
 * almost nothing in it is.
 */
export const shadows = {
  card: '0 1px 2px rgba(11,27,39,0.04), 0 1px 3px rgba(11,27,39,0.06)',
  lift: '0 4px 16px rgba(11,27,39,0.10)',
  nav: '0 1px 0 rgba(11,27,39,0.08)',
} as const

/**
 * The motion.
 *
 * The app moves once: a block that arrives. It does not loop, it does not
 * bounce, and nothing slides in from the side.
 */
export const motion = {
  keyframes: {
    'fade-up': {
      '0%': { opacity: '0', transform: 'translateY(6px)' },
      '100%': { opacity: '1', transform: 'translateY(0)' },
    },
  },
  animation: {
    'fade-up': 'fade-up 0.28s ease-out both',
  },
} as const

/** The two families of type the app is set in. */
export const fonts = {
  sans: [
    'system-ui',
    '-apple-system',
    'BlinkMacSystemFont',
    '"Segoe UI"',
    'Roboto',
    '"Helvetica Neue"',
    'Arial',
    'sans-serif',
  ],
  mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'Consolas', 'monospace'],
} as const
