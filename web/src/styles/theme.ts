import type { GlobalThemeOverrides } from 'naive-ui'

type ThemeName = 'light' | 'dark'

const palettes = {
  light: {
    bg: '#f7f8fa', text: '#20242e', 'text-muted': '#697180',
    surface: '#ffffff', 'surface-raised': '#ffffff', inset: '#f0f2f5',
    border: '#e7e9ee', primary: '#4062d8', 'primary-hover': '#3454c2',
    'on-primary': '#ffffff', 'focus-ring': '#4062d8',
    hover: '#f2f4fa', 'active-bg': '#edf1fc', 'placeholder-bg': '#e9edf5',
    danger: '#c83850', 'danger-bg': '#fceef0', success: '#188454', error: '#c83850',
    shadow: 'rgba(30, 40, 65, 0.07)',
    'scrollbar-track': 'rgba(30, 40, 65, 0.04)',
    'scrollbar-thumb': 'rgba(30, 40, 65, 0.24)',
    'scrollbar-thumb-hover': 'rgba(30, 40, 65, 0.4)',
  },
  dark: {
    bg: '#14161b', text: '#eceef3', 'text-muted': '#9da5b5',
    surface: '#1c1f26', 'surface-raised': '#252932', inset: '#14161b',
    border: '#30343f', primary: '#9badff', 'primary-hover': '#bac6ff',
    'on-primary': '#18203b', 'focus-ring': '#9badff',
    hover: '#272c38', 'active-bg': '#2c334a', 'placeholder-bg': '#2d3340',
    danger: '#ff8e9d', 'danger-bg': '#402830', success: '#6fd5ac', error: '#ff8e9d',
    shadow: 'rgba(0, 0, 0, 0.2)',
    'scrollbar-track': 'rgba(255, 255, 255, 0.04)',
    'scrollbar-thumb': 'rgba(255, 255, 255, 0.24)',
    'scrollbar-thumb-hover': 'rgba(255, 255, 255, 0.4)',
  },
}

// Native elements and Naive UI use the same palette to keep surface and state colors aligned.
export function applyTheme(name: ThemeName) {
  const root = document.documentElement
  root.dataset.theme = name
  root.style.colorScheme = name
  for (const [key, value] of Object.entries(palettes[name])) {
    root.style.setProperty(`--app-${key}`, value)
  }
}

function createOverrides(name: ThemeName): GlobalThemeOverrides {
  const p = palettes[name]
  return {
    common: {
      borderRadius: '9px', borderRadiusSmall: '6px', fontSize: '14px',
      primaryColor: p.primary, primaryColorHover: p['primary-hover'],
      primaryColorPressed: p.primary, primaryColorSuppl: p['primary-hover'],
      bodyColor: p.bg, cardColor: p.surface, modalColor: p['surface-raised'],
      popoverColor: p['surface-raised'], tableColor: p.surface, tableHeaderColor: p.surface,
      tableColorHover: p.hover, inputColor: p.surface, borderColor: p.border, dividerColor: p.border,
      textColor1: p.text, textColor2: p.text, textColor3: p['text-muted'],
    },
    Card: { borderRadius: '14px' },
    Input: { borderRadius: '10px', heightMedium: '38px', fontSizeMedium: '13px' },
    Button: {
      borderRadiusMedium: '9px',
      textColorPrimary: p['on-primary'], textColorHoverPrimary: p['on-primary'],
      textColorPressedPrimary: p['on-primary'], textColorFocusPrimary: p['on-primary'],
    },
    Slider: { railHeight: '3px', handleSize: '12px' },
    DataTable: { thPaddingSmall: '12px 12px', tdPaddingSmall: '10px 12px', borderRadius: '12px' },
  }
}

export const themeOverrides = { light: createOverrides('light'), dark: createOverrides('dark') }
