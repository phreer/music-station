import { get } from './client'
import type { LibraryStats } from '@/types'

export async function fetchStats(signal?: AbortSignal): Promise<LibraryStats> {
  return get<LibraryStats>('/stats', signal)
}
