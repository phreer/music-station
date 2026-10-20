import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Track } from '@/types'
import { streamUrl } from '@/api/client'
import { incrementPlayCount } from '@/api/tracks'
import { useQueueStore } from './queue'
import { useLibraryStore } from './library'

export const usePlayerStore = defineStore('player', () => {
  const currentTrackId = ref<string | null>(null)
  const isPlaying = ref(false)
  const isActuallyPlaying = ref(false)
  const isWaiting = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)
  const volume = ref(parseFloat(localStorage.getItem('player-volume') ?? '0.8'))
  let audioElement: HTMLAudioElement | null = null
  let audioCleanup: (() => void) | null = null
  let beforePlay: (() => void) | null = null
  let playRequest = 0

  const library = useLibraryStore()
  const queue = useQueueStore()

  const currentTrack = computed<Track | null>(() => {
    if (!currentTrackId.value) return null
    return library.findTrack(currentTrackId.value) ?? null
  })

  const progress = computed(() => {
    if (duration.value === 0) return 0
    return (currentTime.value / duration.value) * 100
  })

  function initAudio(el: HTMLAudioElement, prepare?: () => void) {
    audioCleanup?.()
    audioElement = el
    beforePlay = prepare ?? null
    el.volume = volume.value
    let lastTimeUpdate = 0
    const syncTime = () => { currentTime.value = el.currentTime }
    const halted = () => { isActuallyPlaying.value = false }
    const listeners: Record<string, EventListener> = {
      timeupdate: () => {
        const now = performance.now()
        if (now - lastTimeUpdate < 250) return
        lastTimeUpdate = now
        syncTime()
      },
      loadedmetadata: () => { duration.value = Number.isFinite(el.duration) ? el.duration : 0 },
      seeked: () => { syncTime(); isActuallyPlaying.value = !el.paused && !el.ended && el.readyState >= 3 },
      playing: () => { isActuallyPlaying.value = true; isWaiting.value = false; isPlaying.value = true },
      waiting: () => { halted(); isWaiting.value = true },
      seeking: () => { halted(); syncTime() },
      play: () => { isPlaying.value = true },
      pause: () => { halted(); isPlaying.value = false; isWaiting.value = false },
      ended: () => { halted(); isPlaying.value = false; isWaiting.value = false; playNext() },
      emptied: () => { halted(); isWaiting.value = false },
      error: () => { halted(); isPlaying.value = false; isWaiting.value = false },
    }
    for (const [event, listener] of Object.entries(listeners)) el.addEventListener(event, listener)
    audioCleanup = () => {
      for (const [event, listener] of Object.entries(listeners)) el.removeEventListener(event, listener)
    }
  }

  function destroyAudio() {
    playRequest++
    audioElement?.pause()
    audioCleanup?.()
    audioCleanup = null
    audioElement = null
    beforePlay = null
    isPlaying.value = false
    isActuallyPlaying.value = false
    isWaiting.value = false
  }

  function startPlayback(audio: HTMLAudioElement) {
    const request = ++playRequest
    beforePlay?.()
    // Call play in the originating gesture, without awaiting AudioContext.resume.
    return audio.play().then(() => request === playRequest, () => {
      if (request !== playRequest) return false
      isPlaying.value = false
      isActuallyPlaying.value = false
      isWaiting.value = false
      return false
    })
  }

  async function playTrack(trackId: string) {
    const audio = audioElement
    if (!audio) return

    currentTrackId.value = trackId
    currentTime.value = 0
    duration.value = 0
    isActuallyPlaying.value = false
    isWaiting.value = true
    audio.src = streamUrl(trackId)
    if (!await startPlayback(audio)) return

    // Increment play count (fire and forget)
    incrementPlayCount(trackId).then(() => {
      library.updateTrackLocally(trackId, {
        play_count: (library.findTrack(trackId)?.play_count ?? 0) + 1,
      })
    }).catch(() => {})
  }

  function togglePlayPause() {
    const audio = audioElement
    if (!audio) return

    if (!currentTrackId.value) {
      // Nothing loaded, play first from queue or library
      const firstId = queue.queue[0] ?? library.filteredTracks[0]?.id
      if (firstId) {
        if (queue.queue.length > 0) queue.playIndex(0)
        playTrack(firstId)
      }
      return
    }

    if (audio.paused) {
      void startPlayback(audio)
    } else {
      audio.pause()
    }
  }

  function playNext() {
    const nextId = queue.next()
    if (nextId) {
      playTrack(nextId)
    }
  }

  function playPrevious() {
    const prevId = queue.previous()
    if (prevId) {
      playTrack(prevId)
    }
  }

  function stop() {
    playRequest++
    const audio = audioElement
    if (!audio) return
    audio.pause()
    audio.removeAttribute('src')
    audio.load()
    currentTrackId.value = null
    currentTime.value = 0
    duration.value = 0
    isPlaying.value = false
    isActuallyPlaying.value = false
    isWaiting.value = false
  }

  function seek(percent: number) {
    const audio = audioElement
    if (!audio || !duration.value) return
    audio.currentTime = (Math.max(0, Math.min(100, percent)) / 100) * duration.value
    currentTime.value = audio.currentTime
  }

  function setVolume(val: number) {
    volume.value = val
    localStorage.setItem('player-volume', String(val))
    if (audioElement) {
      audioElement.volume = val
    }
  }

  return {
    currentTrackId,
    isPlaying,
    isActuallyPlaying,
    isWaiting,
    currentTime,
    duration,
    volume,
    currentTrack,
    progress,
    initAudio,
    destroyAudio,
    playTrack,
    togglePlayPause,
    playNext,
    playPrevious,
    stop,
    seek,
    setVolume,
  }
})
