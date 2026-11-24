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
 * The rail: the wide arrangement's own ground, and the ink on it.
 *
 * The rail is the one piece of chrome that is not the page's colour — it
 * is the app's own dark ground, with its own ink, so that the
 * destinations read as a place of their own. Every palette answers the
 * rail's roles, because a palette that left the rail white would put the
 * app's navigation in another product's colours.
 */
export interface Rail {
  /** The ground the rail is drawn on. */
  ground: string
  /** A destination at rest. */
  ink: string
  /** The destination that is open, and the one under the pointer. */
  inkStrong: string
  /** The ground of the destination that is open. */
  active: string
  /** The ground of a destination under the pointer. */
  hover: string
  /** The note at the foot of the rail. */
  foot: string
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
  /**
   * The three series a chart draws on one plot: the athlete profile's
   * three windows, each one a hue of its own.
   *
   * **They are chosen, not derived.** A chart that draws more than one
   * series on the same axes needs hues that separate at a glance, and
   * each of them has to hold against the card the chart is drawn on: a
   * pale tint that reads well as a fill is invisible as a line. So each
   * one here is a step deep enough to be seen against the card, and the
   * first is the palette's own, the second the palette's warm other, and
   * the third the palette's own third hue.
   */
  series: string[]
}

/** A whole set of roles: one palette. */
export interface Palette {
  /** The azure ramp in `stride`; the orange-red ramp in `dreeve`. */
  brand: Ramp
  /** The warm ramp: the athlete's own data, the accent of the app. */
  accent: Ramp
  ink: Ink
  surface: Surface
  /** The wide arrangement's own ground, and the ink on it. */
  rail: Rail
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
  rail: Rail
  edge: string
  better: string
  usual: string
  could: string
  ramp: string[]
  chart: { grid: string; axis: string; series: string[] }
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
    rail: input.rail,
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
      series: input.chart.series,
    },
  }
}

/**
 * `stride` — today's colours.
 *
 * **These are the mock's colours.** The approved design lives in
 * `stride-mocks/mock.css`, and this palette is that file's `stride`
 * block, lifted into the app's own ramp: the deep azure for the
 * structure, the warm amber for the athlete's own data, the white cards
 * on a cool ground, and the hairline between them.
 *
 * The mock's inks are three — the ink, the soft ink and the faint ink —
 * and the app writes in four roles, so the middle one is shared: `soft`
 * and `muted` are both the mock's soft ink. That is the mock's own
 * hierarchy, not a fourth colour the mock never had.
 */
export const stride: Palette = palette({
  brand: {
    50: '#F2F7FC',
    100: '#E6EFF9',
    200: '#CFE0F3',
    300: '#A9C8E8',
    400: '#6EA4DE',
    500: '#2B7FE3',
    600: '#1F6BC4',
    700: '#14548F',
    800: '#104A7D',
    900: '#0D3A63',
  },
  accent: {
    50: '#FEF9F0',
    100: '#FDF1DC',
    200: '#F8E2BD',
    300: '#F3D19A',
    400: '#EFB86A',
    500: '#E8A33D',
    600: '#D1872A',
    700: '#B57312',
    800: '#8D5610',
    900: '#6B410C',
  },
  ink: {
    DEFAULT: '#0F1B2A',
    soft: '#5A6B7E',
    muted: '#5A6B7E',
    faint: '#8B9AAB',
  },
  surface: {
    DEFAULT: '#FFFFFF',
    soft: '#F1F4F7',
    raised: '#F8FAFC',
    sunken: '#EEF2F6',
  },
  rail: {
    ground: '#0D3A63',
    ink: '#FFFFFFC7',
    inkStrong: '#FFFFFF',
    active: '#FFFFFF29',
    hover: '#FFFFFF14',
    foot: '#FFFFFF8C',
  },
  edge: '#E3E8EF',
  better: '#1F9D6B',
  usual: '#8B9AAB',
  could: '#D98814',
  ramp: ['#2B7FE3', '#E8A33D', '#0E9C8A', '#7C5CD6', '#E2607A', '#6B7F96'],
  chart: {
    grid: '#EEF2F6',
    axis: '#8B9AAB',
    // The three series of a chart that draws more than one: the deep
    // azure, the deep amber and the teal. Each holds against the white
    // card (5.3, 3.9 and 3.4 to one) and the three hues are far apart.
    series: ['#1F6BC4', '#B57312', '#0E9C8A'],
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
  rail: {
    ground: '#FFFFFF',
    ink: '#6B7280',
    inkStrong: '#111827',
    active: '#FFF1E9',
    hover: '#F9FAFB',
    foot: '#9CA3AF',
  },
  edge: '#E5E7EB',
  better: '#15803D',
  usual: '#6B7280',
  could: '#B45309',
  ramp: ['#FC5200', '#BD2D22', '#6B7280', '#374151', '#942319', '#9CA3AF'],
  chart: {
    grid: '#ECEEF0',
    axis: '#6B7280',
    // The three series of a chart that draws more than one: the
    // palette's own orange-red, its slate, and its deep red. Each holds
    // against the white card (4.8, 4.8 and 8.4 to one).
    series: ['#D03F00', '#6B7280', '#942319'],
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
  rail: {
    ground: '#14202F',
    ink: '#9FB0C4',
    inkStrong: '#E8EEF6',
    active: '#24344A',
    hover: '#1E2A38',
    foot: '#8497AC',
  },
  edge: '#24344A',
  better: '#34D399',
  usual: '#94A3B8',
  could: '#FBBF24',
  ramp: ['#60A5FA', '#FBBF24', '#2DD4BF', '#A78BFA', '#FB7185', '#94A3B8'],
  chart: {
    grid: '#24344A',
    axis: '#8497AC',
    // The three series of a chart that draws more than one: the light
    // blue, the gold and the teal. Each holds against the dark card
    // (6.5, 9.9 and 8.8 to one).
    series: ['#60A5FA', '#FBBF24', '#2DD4BF'],
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
 * **The card is the mock's card:** fourteen pixels — soft enough to sit on
 * a page and square enough to stack. The small radius is the mock's nine,
 * and the small radius is what a control, a mark and a chip take.
 */
export const radii = {
  none: '0px',
  sm: '2px',
  md: '9px',
  lg: '12px',
  card: '14px',
  xl: '16px',
  '2xl': '20px',
  '3xl': '24px',
  full: '9999px',
} as const

/**
 * The shadows.
 *
 * **The card's shadow is the mock's shadow:** a soft two-part shadow — a
 * hairline under the card and a wide, shallow lift off the page. It says
 * "this is a card", and it is the only thing in the app that says it.
 */
export const shadows = {
  card: '0 1px 2px rgba(15,27,42,0.05), 0 6px 20px -12px rgba(15,27,42,0.22)',
  lift: '0 20px 60px -20px rgba(15,27,42,0.35)',
  nav: '0 1px 0 rgba(15,27,42,0.08)',
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

/**
 * The two families of type the app is set in.
 *
 * **The sans is Inter**, the mock's face — and it is the app's *own* copy
 * of it: the files live in `frontend/public/fonts/`, are loaded by the
 * app's own CSS, and are never fetched from a CDN. The app is a
 * self-hosted desktop program and the phone is a webview: both must draw
 * the same face with no network at all.
 */
export const fonts = {
  sans: [
    'Inter',
    'ui-sans-serif',
    'system-ui',
    '-apple-system',
    '"Segoe UI"',
    'Roboto',
    'sans-serif',
  ],
  mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'Consolas', 'monospace'],
} as const
