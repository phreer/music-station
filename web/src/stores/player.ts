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
  const currentTime = ref(0)
  const duration = ref(0)
  const volume = ref(parseFloat(localStorage.getItem('player-volume') ?? '0.8'))
  const audioElement = ref<HTMLAudioElement | null>(null)

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

  function initAudio(el: HTMLAudioElement) {
    audioElement.value = el
    el.volume = volume.value

    el.addEventListener('timeupdate', () => {
      currentTime.value = el.currentTime
    })
    el.addEventListener('loadedmetadata', () => {
      duration.value = el.duration
    })
    el.addEventListener('ended', () => {
      playNext()
    })
    el.addEventListener('play', () => {
      isPlaying.value = true
    })
    el.addEventListener('pause', () => {
      isPlaying.value = false
    })
  }

  async function playTrack(trackId: string) {
    const audio = audioElement.value
    if (!audio) return

    currentTrackId.value = trackId
    audio.src = streamUrl(trackId)
    await audio.play()

    // Increment play count (fire and forget)
    incrementPlayCount(trackId).then(() => {
      library.updateTrackLocally(trackId, {
        play_count: (library.findTrack(trackId)?.play_count ?? 0) + 1,
      })
    })
  }

  function togglePlayPause() {
    const audio = audioElement.value
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
      audio.play()
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
    const audio = audioElement.value
    if (!audio) return
    audio.pause()
    audio.src = ''
    currentTrackId.value = null
    currentTime.value = 0
    duration.value = 0
    isPlaying.value = false
  }

  function seek(percent: number) {
    const audio = audioElement.value
    if (!audio || !duration.value) return
    audio.currentTime = (percent / 100) * duration.value
  }

  function setVolume(val: number) {
    volume.value = val
    localStorage.setItem('player-volume', String(val))
    if (audioElement.value) {
      audioElement.value.volume = val
    }
  }

  return {
    currentTrackId,
    isPlaying,
    currentTime,
    duration,
    volume,
    currentTrack,
    progress,
    initAudio,
    playTrack,
    togglePlayPause,
    playNext,
    playPrevious,
    stop,
    seek,
    setVolume,
  }
})
