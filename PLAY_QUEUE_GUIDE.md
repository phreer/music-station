# Play Queue Feature Guide

## Overview
The Play Queue is a temporary playlist that allows you to build and manage a custom playback sequence on-the-fly. Unlike saved playlists, the queue is session-based and resets when you refresh the page.

## Key Features

### 🎵 **Queue Management**
- Add individual tracks to the queue
- Add entire albums to the queue at once
- View all queued tracks in order
- See queue statistics (track count, total duration)
- Remove tracks from the queue
- Clear the entire queue

### 🎮 **Playback Control**
- Queue takes priority over regular playlist playback
- Next/Previous buttons navigate through the queue
- Click any track in the queue to jump to it
- Visual indicator shows the currently playing track

### 💾 **Queue Persistence**
- Queue is temporary and session-based
- Resets when you refresh or close the browser
- Perfect for creating one-time listening sessions

## How to Use

### Adding Tracks to Queue

#### From Track List
1. Browse to the **Tracks** tab
2. Find a track you want to add
3. Click the **📋** button next to the track
4. Track is added to the end of the queue

#### From Albums
1. Go to the **Albums** tab
2. Find an album you want to add
3. Click the **📋 Add to Queue** button on the album card
4. All tracks from the album are added in order

### Viewing the Queue

#### Show Queue Panel
- Click the **📋** button (bottom-right corner)
- The queue panel slides in from the right side
- Shows all queued tracks with:
  - Track number in queue
  - Cover art (if available)
  - Title and artist
  - Duration
  - Remove button

#### Queue Information
At the top of the queue panel:
- **Track count**: Total number of tracks
- **Total duration**: Combined length of all tracks

### Managing Queue Tracks

#### Play a Track
- Click any track in the queue to start playing it
- The playing track is highlighted
- Playback continues through the queue in order

#### Remove a Track
1. Hover over a track in the queue
2. Click the **✖️** button that appears
3. Track is removed from queue
4. Queue indices adjust automatically

#### Clear Queue
1. Open the queue panel
2. Click the **🗑️** button in the header
3. Confirm the action
4. All tracks are removed from the queue

### Queue Playback Behavior

#### Priority System
- **If queue exists**: Next/Previous buttons navigate queue
- **If queue is empty**: Next/Previous buttons navigate all tracks
- **Play button**: Starts queue if it exists, otherwise plays all tracks

#### Auto-advance
- When a track finishes, the next track in queue plays automatically
- Loops back to the first track when reaching the end
- Works seamlessly with the music player controls

### Visual Indicators

#### Queue Toggle Button
- Fixed position button (bottom-right)
- Shows **📋** icon
- Red badge displays queue count
- Only visible when queue has tracks

#### Currently Playing
- Highlighted with orange border in queue
- Track number shows in orange
- Also highlighted in main track table

#### Queue Panel
- Slides in from right side
- Dark header with queue info
- Scrollable track list
- Hover effects for interactivity

## Tips & Tricks

### Building a Queue
1. **Start with an album**: Add your favorite album to queue
2. **Add singles**: Browse tracks and add standout songs
3. **Mix genres**: Create variety by adding from different albums
4. **Adjust as you go**: Remove tracks you want to skip

### Queue Workflows

#### Morning Playlist
1. Add a upbeat album to start the day
2. Add a few energizing singles
3. Let it play through while you work

#### Discovery Mode
1. Add one track from multiple albums
2. Explore different artists
3. Note which tracks you love

#### Party Mode
1. Add several popular albums
2. Add crowd favorites as singles
3. Keep the music flowing

### Best Practices
- **Keep it manageable**: 10-30 tracks is ideal for most sessions
- **Use albums**: Faster than adding individual tracks
- **Clear when done**: Start fresh for each listening session
- **Combine with playlists**: Use saved playlists for permanent collections

## Keyboard & Mouse

### Queue Panel
- **Click track**: Play that track
- **Hover track**: Show remove button
- **Click remove**: Delete track from queue

### Toggle Button
- **Click**: Show/hide queue panel
- **Badge**: Shows current queue size

## Technical Details

### Storage
- Queue stored in browser memory (JavaScript variable)
- Not saved to localStorage or server
- Lost on page refresh/reload
- Independent of saved playlists

### Limitations
- Maximum: No hard limit (practical limit ~1000 tracks)
- No drag-and-drop reordering (yet)
- No shuffle option (yet)
- No export/import functionality

### Performance
- Instant add/remove operations
- Smooth scrolling for large queues
- Minimal memory footprint
- No impact on server

## Troubleshooting

**Q: Queue button not showing?**
- Add a track to queue first
- Button appears automatically when queue has tracks

**Q: Queue disappeared?**
- Queue is temporary and resets on page refresh
- Use saved playlists for permanent collections

**Q: Can't see remove button?**
- Hover over the track in the queue panel
- Button appears on hover

**Q: Next button not playing queue?**
- Ensure tracks are in the queue
- Queue takes priority over regular playback

**Q: How to save my queue?**
- Queue is temporary by design
- Create a playlist for permanent storage
- You can manually add queue tracks to a playlist

## Differences from Playlists

| Feature | Play Queue | Playlists |
|---------|-----------|-----------|
| **Persistence** | Temporary | Saved permanently |
| **Storage** | Browser memory | Browser localStorage |
| **Purpose** | Current session | Long-term collections |
| **Creation** | Quick, on-the-fly | Deliberate organization |
| **Visibility** | Current session only | Available anytime |
| **Editing** | Add/remove only | Full management |

## Future Enhancements

Potential features being considered:
- Drag-and-drop reordering
- Shuffle queue option
- Repeat queue mode
- Save queue as playlist
- Import playlist to queue
- Queue history (recent queues)
- Auto-queue similar tracks

## Keyboard Shortcuts (Planned)

Future keyboard shortcuts for queue:
- `Q` - Toggle queue panel
- `Ctrl+Q` - Clear queue
- `Alt+Q` - Add current track to queue

---

**Pro Tip**: Use the queue for immediate playback needs and playlists for your curated collections. They work great together!
