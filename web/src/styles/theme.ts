import type { GlobalThemeOverrides } from 'naive-ui'

const base = {
  borderRadius: '8px',
  borderRadiusSmall: '6px',
  fontSize: '14px',
}

export const themeOverrides: Record<'light' | 'dark', GlobalThemeOverrides> = {
  light: {
    common: {
      ...base,
      primaryColor: '#356ae6',
      primaryColorHover: '#2457d3',
      primaryColorPressed: '#1c48b5',
      primaryColorSuppl: '#2457d3',
    },
    Card: { borderRadius: '12px' },
    Input: { borderRadius: '8px' },
    Button: { borderRadiusMedium: '8px' },
  },
  dark: {
    common: {
      ...base,
      primaryColor: '#82aaff',
      primaryColorHover: '#a5c2ff',
      primaryColorPressed: '#6392f3',
      primaryColorSuppl: '#a5c2ff',
    },
    Card: { borderRadius: '12px' },
    Input: { borderRadius: '8px' },
    Button: { borderRadiusMedium: '8px' },
  },
}
