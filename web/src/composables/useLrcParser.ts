import type { ParsedLyricsLine, ParsedWord } from '@/types'

export function useLrcParser() {
  function parseLrc(content: string): ParsedLyricsLine[] {
    const lines: ParsedLyricsLine[] = []
    const regex = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/

    for (const line of content.split('\n')) {
      const match = regex.exec(line)
      if (match) {
        const mins = parseInt(match[1]!)
        const secs = parseInt(match[2]!)
        const ms = match[3]!.length === 2 ? parseInt(match[3]!) * 10 : parseInt(match[3]!)
        const time = mins * 60 + secs + ms / 1000
        const text = match[4]?.trim() ?? ''
        if (text) {
          lines.push({ time, text })
        }
      }
    }

    return lines.sort((a, b) => a.time - b.time)
  }

  function parseWordLevel(content: string): ParsedLyricsLine[] {
    const lines: ParsedLyricsLine[] = []
    const lineRegex = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/

    for (const line of content.split('\n')) {
      const match = lineRegex.exec(line)
      if (match) {
        const mins = parseInt(match[1]!)
        const secs = parseInt(match[2]!)
        const ms = match[3]!.length === 2 ? parseInt(match[3]!) * 10 : parseInt(match[3]!)
        const time = mins * 60 + secs + ms / 1000
        const rest = match[4] ?? ''
        const words = parseWordsWithTiming(rest, time)
        const text = words.map((w) => w.text).join('')
        if (text.trim()) {
          lines.push({ time, text, words })
        }
      }
    }

    return lines.sort((a, b) => a.time - b.time)
  }

  function parseWordsWithTiming(text: string, _lineTime: number): ParsedWord[] {
    const words: ParsedWord[] = []
    const wordRegex = /([^(]+)\((\d+),(\d+)\)/g
    let match: RegExpExecArray | null

    while ((match = wordRegex.exec(text)) !== null) {
      words.push({
        text: match[1]!,
        offset: parseInt(match[2]!) / 1000,
        duration: parseInt(match[3]!) / 1000,
      })
    }

    // Fallback: if no word-level timing found, treat as single word
    if (words.length === 0 && text.trim()) {
      words.push({ text: text.trim(), offset: 0, duration: 0 })
    }

    return words
  }

  return { parseLrc, parseWordLevel }
}
