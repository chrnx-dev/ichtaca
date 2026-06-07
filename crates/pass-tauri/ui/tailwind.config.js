/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{svelte,ts,js}',
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('daisyui'),
  ],
  daisyui: {
    themes: [
      {
        ichtaca: {
          // ── Obsidiana & Oro custom DaisyUI theme ─────────────────────────
          // Base / page backgrounds
          'base-100': '#1E1B26',   // surface (panels / blocks)
          'base-200': '#2A2533',   // surface_sel (slightly lighter)
          'base-300': '#332E3D',   // even lighter surface
          'base-content': '#E8E2D0', // cream amate — main text

          // Brand colours
          'primary': '#E0A436',          // gold
          'primary-content': '#15131A',  // dark text on gold
          'secondary': '#2FB6A8',        // turquoise
          'secondary-content': '#15131A',
          'accent': '#46D0C0',           // turquoise-bright / jade accent
          'accent-content': '#15131A',
          'neutral': '#6B6478',          // muted (borders, hints)
          'neutral-content': '#E8E2D0',

          // Semantic
          'success': '#3FA66A',          // jade
          'success-content': '#15131A',
          'warning': '#E0A436',          // gold as warning
          'warning-content': '#15131A',
          'error': '#C8443B',            // cochineal
          'error-content': '#E8E2D0',
          'info': '#2FB6A8',             // turquoise as info
          'info-content': '#15131A',

          // Page (body) background — obsidian
          '--rounded-box': '0.5rem',
          '--rounded-btn': '0.375rem',
          '--rounded-badge': '0.25rem',
          '--color-scheme': 'dark',
        },
      },
    ],
    darkTheme: 'ichtaca',
    base: true,
    styled: true,
    utils: true,
    logs: false,
  },
}
