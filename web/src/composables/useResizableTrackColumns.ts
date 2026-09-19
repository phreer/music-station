import { onBeforeUnmount, ref, watch } from 'vue'

type ColumnKey = string

interface ResizableColumnConfig {
  key: ColumnKey
  defaultWidth: number
  minWidth: number
  maxWidth?: number
}

interface ResizeState {
  key: ColumnKey
  startX: number
  startWidth: number
}

function clampWidth(width: number, config: ResizableColumnConfig): number {
  const limitedWidth = config.maxWidth == null ? width : Math.min(width, config.maxWidth)
  return Math.max(config.minWidth, Math.round(limitedWidth))
}

export function useResizableTrackColumns(
  storageKey: string,
  configs: ResizableColumnConfig[],
) {
  const configByKey = new Map(configs.map((config) => [config.key, config]))

  const loadInitialWidths = (): Record<ColumnKey, number> => {
    const defaults = Object.fromEntries(configs.map((config) => [config.key, config.defaultWidth]))
    if (typeof window === 'undefined') return defaults

    try {
      const raw = window.localStorage.getItem(storageKey)
      if (!raw) return defaults

      const parsed = JSON.parse(raw) as Record<ColumnKey, unknown>
      return Object.fromEntries(
        configs.map((config) => {
          const candidate = parsed[config.key]
          return [
            config.key,
            typeof candidate === 'number'
              ? clampWidth(candidate, config)
              : config.defaultWidth,
          ]
        }),
      )
    } catch {
      return defaults
    }
  }

  const widths = ref<Record<ColumnKey, number>>(loadInitialWidths())

  const persist = () => {
    if (typeof window === 'undefined') return
    window.localStorage.setItem(storageKey, JSON.stringify(widths.value))
  }

  watch(widths, persist, { deep: true })

  const setWidth = (key: ColumnKey, nextWidth: number) => {
    const config = configByKey.get(key)
    if (!config) return
    widths.value = {
      ...widths.value,
      [key]: clampWidth(nextWidth, config),
    }
  }

  const getWidth = (key: ColumnKey): number => {
    const config = configByKey.get(key)
    return widths.value[key] ?? config?.defaultWidth ?? 0
  }

  let resizeState: ResizeState | null = null
  let removeListeners: (() => void) | null = null
  let previousCursor = ''
  let previousUserSelect = ''

  const stopResize = () => {
    removeListeners?.()
    removeListeners = null
    resizeState = null
    document.body.style.cursor = previousCursor
    document.body.style.userSelect = previousUserSelect
  }

  const startResize = (key: ColumnKey, event: MouseEvent) => {
    const config = configByKey.get(key)
    if (!config) return

    stopResize()
    resizeState = {
      key,
      startX: event.clientX,
      startWidth: widths.value[key] ?? config.defaultWidth,
    }

    previousCursor = document.body.style.cursor
    previousUserSelect = document.body.style.userSelect
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!resizeState) return
      setWidth(
        resizeState.key,
        resizeState.startWidth + moveEvent.clientX - resizeState.startX,
      )
    }

    const handleMouseUp = () => {
      stopResize()
    }

    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('mouseup', handleMouseUp)
    removeListeners = () => {
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseup', handleMouseUp)
    }
  }

  onBeforeUnmount(() => {
    if (typeof document !== 'undefined') stopResize()
  })

  return {
    widths,
    getWidth,
    setWidth,
    startResize,
  }
}
