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
      bodyColor: '#f5f7fb',
      cardColor: '#ffffff',
      modalColor: '#ffffff',
      popoverColor: '#ffffff',
      tableColor: '#ffffff',
      tableHeaderColor: '#ffffff',
      inputColor: '#ffffff',
      borderColor: '#e3e8f0',
      dividerColor: '#e3e8f0',
      textColor1: '#202938',
      textColor2: '#202938',
      textColor3: '#647084',
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
      bodyColor: '#12151c',
      cardColor: '#1b202a',
      modalColor: '#232a36',
      popoverColor: '#232a36',
      tableColor: '#1b202a',
      tableHeaderColor: '#1b202a',
      inputColor: '#232a36',
      borderColor: '#303948',
      dividerColor: '#303948',
      textColor1: '#e8ecf3',
      textColor2: '#e8ecf3',
      textColor3: '#a0aabc',
    },
    Card: { borderRadius: '12px' },
    Input: { borderRadius: '8px' },
    Button: {
      borderRadiusMedium: '8px',
      textColorPrimary: '#101622',
      textColorHoverPrimary: '#101622',
      textColorPressedPrimary: '#101622',
      textColorFocusPrimary: '#101622',
    },
  },
}
