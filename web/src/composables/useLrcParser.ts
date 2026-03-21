import type { ParsedLyricsLine, ParsedWord } from '@/types'

const METADATA_PREFIXES = ['[ti:', '[ar:', '[al:', '[by:', '[offset:']

function isMetadataLine(line: string): boolean {
  const trimmed = line.trim().toLowerCase()
  return METADATA_PREFIXES.some((prefix) => trimmed.startsWith(prefix))
}

export function useLrcParser() {
  // Standard LRC: [mm:ss.xx] — minutes can be any number of digits
  const standardRegex = /\[(\d+):(\d{2})\.(\d{2,3})\](.*)/
  // Offset LRC: [offset_ms,duration_ms] — common from NetEase/QQ Music
  const offsetRegex = /\[(\d+),(\d+)\](.*)/

  function parseStandardTimestamp(match: RegExpExecArray): number {
    const mins = parseInt(match[1]!)
    const secs = parseInt(match[2]!)
    const frac = match[3]!
    const ms = frac.length === 2 ? parseInt(frac) * 10 : parseInt(frac)
    return mins * 60 + secs + ms / 1000
  }

  function parseOffsetTimestamp(match: RegExpExecArray): number {
    const offsetMs = parseInt(match[1]!)
    return offsetMs / 1000
  }

  /**
   * Try to parse a line's timestamp. Returns [time, rest] or null if no match.
   */
  function parseLineTimestamp(line: string): [number, string] | null {
    // Try standard LRC first
    let match = standardRegex.exec(line)
    if (match) {
      return [parseStandardTimestamp(match), match[4] ?? '']
    }
    // Try offset format
    match = offsetRegex.exec(line)
    if (match) {
      return [parseOffsetTimestamp(match), match[3] ?? '']
    }
    return null
  }

  function parseLrc(content: string): ParsedLyricsLine[] {
    const lines: ParsedLyricsLine[] = []

    for (const line of content.split('\n')) {
      if (isMetadataLine(line)) continue

      const parsed = parseLineTimestamp(line)
      if (parsed) {
        const [time, text] = parsed
        const trimmed = text.trim()
        // Strip word-level timing if present, keeping only text
        const hasWordTiming = /\(\d+,\d+\)/.test(trimmed)
        const cleanText = hasWordTiming
          ? trimmed.replace(/(.+?)\((\d+),(\d+)\)/g, '$1').replace(/\s+/g, ' ').trim()
          : trimmed
        if (cleanText) {
          lines.push({ time, text: cleanText })
        }
      }
    }

    return lines.sort((a, b) => a.time - b.time)
  }

  function parseWordLevel(content: string): ParsedLyricsLine[] {
    const lines: ParsedLyricsLine[] = []

    for (const line of content.split('\n')) {
      if (isMetadataLine(line)) continue

      const parsed = parseLineTimestamp(line)
      if (parsed) {
        const [time, rest] = parsed
        const words = parseWordsWithTiming(rest)
        const text = words.map((w) => w.text).join('')
        if (text.trim()) {
          lines.push({ time, text, words })
        }
      }
    }

    return lines.sort((a, b) => a.time - b.time)
  }

  function parseWordsWithTiming(text: string): ParsedWord[] {
    const words: ParsedWord[] = []
    // Non-greedy match: word(offset_ms,duration_ms)
    const wordRegex = /(.+?)\((\d+),(\d+)\)/g
    let match: RegExpExecArray | null

    while ((match = wordRegex.exec(text)) !== null) {
      words.push({
        text: match[1]!,
        // Absolute time in seconds (matching legacy behavior)
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
