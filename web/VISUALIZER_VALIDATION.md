# Player visualization validation

The visualizer is opt-in (`player-visualizer` in local storage). Clicking the cover opens the immersive player even when visualization is disabled. Both layouts operate the existing audio element, queue, progress, volume, and lyrics store. The immersive view is a Naive UI modal with focus trapping, an inert application background, Escape support, and focus restoration.

The audio service owns one source and one analyser for the player's lifetime. It waits for the first context resume before rerouting native playback, so a denied or pending first resume leaves native output intact. Disabling visualization disconnects only the analyser branch. A context that has taken over media playback stays open until player teardown. Initialization or analysis failures produce one warning per load; the graph is not recreated automatically. Explicit playback can still resume its existing context after an analysis failure.

The renderer uses logarithmic 60 Hz–16 kHz bands (limited by Nyquist), 48/24 bars, `fftSize=1024`, 30 FPS, DPR at most 1.5, and at most one million canvas pixels. Two consecutive 60-frame groups above a 2 ms sampling/drawing P95 budget reduce the renderer to 20 FPS and DPR 1 until visualization is re-enabled. Canvas and analyser work runs outside Vue's reactive state. This budget does not include all browser audio-thread or GPU costs.

## Automated checks

Run from `web/`:

```sh
npm test
./node_modules/.bin/vue-tsc --noEmit
./node_modules/.bin/vite build --outDir /private/tmp/music-station-visualizer-build
```

All 26 Node tests, the TypeScript check, and the temporary-directory build passed. The tests use Vite's TypeScript module loader and cover graph reuse, direct sound routing, exclusive animation ownership, visibility/buffering suspension, pause decay, pending and rejected first resume, context interruption and recovery, drawing/sampling exceptions, resource release, logarithmic bands at seven sample rates, silence, canvas limits, FPS limiting and persistent performance downgrade. Lyrics tests cover canceled and deliberately delayed responses, old `finally` handlers, resetting to a song without lyrics, distinct HTTP failures, and forward/backward synchronization. Player tests cover immediate seeking, actual playback/waiting state, listener teardown, reduced-motion preferences, and superseded play rejection.

The verification build goes to a temporary directory and does not replace `static/`. Vite reports its existing large Naive UI chunk warning.

## Browser environment

Validation date: 2026-10-20. Device: MacBook Air (Mac15,12), Apple M3, 24 GB RAM, macOS 15.8.1. Browser: installed Chromium 156.0.0.0, headless, via Playwright CLI. The fixture serves a 240-second, 44.1 kHz PCM WAV containing modulated tones and 120 synchronized lyric lines; it does not touch the user's music library or databases.

The normal browser run stalled both media and AudioContext clocks with this machine's OS audio output. The test browser was restarted with `--disable-audio-output`. Chromium documents this as replacing the OS stream with a fake stream while exercising the audio output path ([Chromium change](https://chromium.googlesource.com/chromium/src/+/f29eb01290cd36a30177ecf8197f906c01088a0d)). Real media decoding, Web Audio nodes, FFT sampling, Canvas drawing, and playback timing still run. These measurements cannot establish audible output quality or real-device audio power use.

Performance instrumentation is injected only into the fixture browser, outside production code. Sampling/drawing P95 times the analyser read through the end of that animation callback. Browser CPU sums the isolated browser process tree's CDP `SystemInfo.getProcessInfo` CPU-time deltas, divided by wall time (100% means one fully occupied core). Main-thread busy percentage separately uses CDP `TaskDuration` divided by elapsed time. Heap figures use CDP JS heap after garbage collection and exclude GPU and native audio allocations. Instrumentation adds a small amount of overhead.

## Performance comparison

Each mode starts at the beginning of the same fixture track, in the same browser session, with the same synchronized lyrics. One sample is collected per mode, so these figures are a smoke test, not a statistically controlled device benchmark.

| Mode | Elapsed | Browser CPU | Main thread busy | Draw P95 | FPS | JS heap start → end |
| --- | --- | --- | --- | --- | --- | --- |
| Off | 60.12 s | 6.02% | 1.91% | — | 0 | 7.71 → 7.79 MiB |
| Mini | 60.13 s | 15.96% | 3.87% | 0.20 ms | 29.21 | 7.98 → 8.02 MiB |
| Immersive | 60.12 s | 15.23% | 3.78% | 0.20 ms | 29.06 | 8.08 → 8.12 MiB |

Off mode created zero sources/analysers and ran zero spectrum loops. Mini and immersive modes each retained exactly one source, one analyser, and one spectrum loop. The analyser operated at the context's 48 kHz sample rate. The desktop viewport was 1280 × 720 at DPR 1. The immersive keyboard-focus fix and safer first-resume/startup ordering were verified afterward; they do not change the drawing algorithm or canvas geometry measured here.

## Interaction and lifecycle checks

Chromium checks passed for:

- Enabling visualization, refreshing with the saved preference, changing songs, pausing/resuming, seeking through the UI, and explicitly resuming a suspended context. The context is created from enable/play gestures, including when restoring the saved preference.
- Cover entry with visualization disabled, mini expansion, shared lyrics with no extra requests on entry, one mounted lyrics display, Tab/Shift+Tab focus trapping, Escape/close buttons, focus restoration, artist navigation, sidebar-side restoration, and automatic exit when playback is stopped and cleared.
- Plain, LRC and word-level lyrics, songs without lyrics, a deliberately failed HTTP request, manual scrolling and return-to-current following. Seeking updates the active word immediately; regular highlighting still follows the existing roughly 4 Hz playback updates.
- Runtime reduced-motion changes, suppression of auto-enable on a reduced-motion reload without overwriting the saved preference, and deliberate enabling afterward. English/Chinese text and light/dark themes were checked.
- The 390 × 844 mobile viewport uses a vertical layout, a 56 px mini spectrum, an independently scrolling lyrics region, and controls within the bottom of the viewport. This is desktop viewport emulation, not a physical-phone result.

Twenty immersive round trips (40 transitions) left source/analyser/loop counts at 1/1/1, with unchanged tracked listeners and resize/intersection observers. Player teardown closed the context, disconnected its audio graph, removed its listeners, and left zero spectrum loops or intersection observers. The remaining document visibility listener belongs to Vue Router's history integration, outside the player lifecycle.

A separate 20-round-trip check explicitly tracked media-query listeners: two before and after switching, zero after unmount.

Hidden/visible and buffering/playing handlers were exercised using injected visibility state/events and media events. Context suspension/resumption used the real AudioContext. These checks do not establish behavior across OS sleep or physical mobile backgrounding.

The browser console's deliberate lyrics-error case returned HTTP 503 as expected; the final interaction run produced no unexpected application exceptions.

Previews: [desktop light](../output/playwright/immersive-desktop-light.png), [desktop dark](../output/playwright/immersive-desktop-dark.png), [mobile viewport dark](../output/playwright/immersive-mobile-dark.png), [mobile viewport Chinese](../output/playwright/immersive-mobile-zh.png).

## Remaining device coverage

Safari WebDriver was attempted, but session creation failed because Safari's “Allow remote automation” setting is disabled. No Safari settings were changed. Safari, iOS Safari, Android Chrome, physical-phone performance/power use, real audible output, and native/GPU memory remain unverified. Desktop viewport emulation is not used to claim coverage of those devices.
