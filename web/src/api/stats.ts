import { get } from './client'
import type { LibraryStats } from '@/types'

export async function fetchStats(): Promise<LibraryStats> {
  return get<LibraryStats>('/stats')
}
