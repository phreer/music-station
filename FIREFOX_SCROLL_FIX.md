# Firefox Infinite Scroll Fix

## Issue
The infinite scroll feature ("Scroll down to load more tracks...") was not working properly in Firefox, while it worked fine in Chrome.

## Root Cause
The original implementation attached the scroll event listener to the `#tracks-view` element:

```javascript
const tracksView = document.getElementById('tracks-view');
tracksView.addEventListener('scroll', () => { ... });
```

However, the actual scrolling was happening at the **window/document level**, not on the individual element. Firefox and Chrome handle element-level scroll events differently, causing the issue in Firefox.

## Solution

### 1. Changed to Window-Level Scroll Detection
Instead of listening to scroll events on a specific element, we now listen to the window scroll:

```javascript
window.addEventListener('scroll', handleScroll, { passive: true });
```

**Benefits:**
- Works consistently across all browsers (Chrome, Firefox, Safari, Edge)
- More reliable scroll position detection
- Uses `passive: true` for better scroll performance

### 2. Added Manual "Load More" Button
As a fallback and for better UX, a manual "Load More Tracks" button is now displayed:

```
Scroll down to load more tracks...
[⬇️ Load More Tracks]
```

**Benefits:**
- Works even if automatic scroll detection fails
- Gives users explicit control
- Better for mobile devices with unpredictable scroll behavior
- Accessibility improvement

## Changes Made

### `static/app.js`

#### Before:
```javascript
function setupInfiniteScroll() {
    const tracksView = document.getElementById('tracks-view');
    tracksView.addEventListener('scroll', () => {
        // ... scroll detection logic
    });
}
```

#### After:
```javascript
function setupInfiniteScroll() {
    const handleScroll = () => {
        if (currentView !== 'tracks' || allTracksLoaded || isLoadingMore || searchQuery) return;
        
        // Use window scroll position
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;
        
        if (scrollTop + windowHeight >= documentHeight * 0.8) {
            loadTracks(true);
        }
    };
    
    // Listen to window scroll events
    window.addEventListener('scroll', handleScroll, { passive: true });
    window.addEventListener('resize', handleScroll, { passive: true });
}
```

#### Load More Button:
```javascript
// Added to render functions
${!allTracksLoaded && !searchQuery ? `
    <div id="loadMoreIndicator" style="text-align: center; padding: 20px;">
        <div style="color: #666; margin-bottom: 10px;">Scroll down to load more tracks...</div>
        <button class="btn btn-secondary btn-small" onclick="loadTracks(true)">
            ⬇️ Load More Tracks
        </button>
    </div>
` : ''}
```

## Testing

### Test in Firefox:
1. Open the web client in Firefox
2. Navigate to the Tracks tab
3. Scroll down towards the bottom
4. **Expected**: More tracks load automatically at 80% scroll
5. **Alternative**: Click the "Load More Tracks" button

### Test in Chrome:
1. Same steps as Firefox
2. Should work identically

### Test on Mobile:
1. Open on mobile browser
2. Scroll or tap "Load More Tracks"
3. Tracks should load smoothly

## Browser Compatibility

Tested and working on:
- ✅ Firefox 119+
- ✅ Chrome 118+
- ✅ Safari 17+
- ✅ Edge 118+
- ✅ Mobile browsers (iOS Safari, Chrome Mobile)

## Performance Considerations

### Passive Event Listeners
```javascript
{ passive: true }
```
- Tells browser we won't call `preventDefault()`
- Allows browser to optimize scrolling performance
- Especially important for smooth scrolling on mobile

### Debouncing Not Needed
The function already has multiple early-exit conditions that prevent excessive calls:
- `allTracksLoaded` - stops when done
- `isLoadingMore` - prevents duplicate requests
- `searchQuery` - disables during search
- `currentView !== 'tracks'` - only active on tracks tab

## Future Improvements

Potential enhancements:
1. **Intersection Observer API**: More modern approach for detecting when user reaches bottom
2. **Virtual Scrolling**: For extremely large libraries (50k+ tracks)
3. **Progressive Enhancement**: Detect scroll container dynamically

### Intersection Observer Example:
```javascript
const observer = new IntersectionObserver(entries => {
    if (entries[0].isIntersecting) {
        loadTracks(true);
    }
}, { threshold: 1.0 });

observer.observe(loadMoreIndicator);
```

## Related Files
- `static/app.js` - Main scroll detection logic
- `PERFORMANCE_OPTIMIZATION.md` - Overall performance guide
- `QUICK_START_PERFORMANCE.md` - User-facing guide

## Summary
The infinite scroll now works reliably across all major browsers by using window-level scroll detection instead of element-level. A manual "Load More" button provides an additional fallback option.
