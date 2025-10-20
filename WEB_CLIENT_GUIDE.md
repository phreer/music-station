# Web Client Usage Guide

## Accessing the Web Client

1. **Start the server** with your music library:
   ```bash
   cargo run -- --library /path/to/music/folder
   ```

2. **Open your browser** to:
   ```
   http://localhost:3000/web/index.html
   ```

## Features

### Browse Your Music Library

- **Automatic Loading**: The web client automatically loads all tracks from the server
- **Track Cards**: Each track is displayed in a card with:
  - Title, Artist, Album
  - Duration and file size
  - Track ID (truncated)
  - Action buttons

### Edit Track Metadata

1. Click the **"✏️ Edit"** button on any track
2. A modal dialog opens with the current metadata
3. Edit any of the fields:
   - **Title**: Song title
   - **Artist**: Artist name
   - **Album**: Album name
4. Click **"💾 Save Changes"** to persist to the FLAC file
5. Changes are immediately saved to the file and reflected in the UI

**Notes:**
- Metadata is written directly to the FLAC file using Vorbis comments
- Changes persist across server restarts
- Only non-empty fields are updated

### Play Music

Click the **"▶️ Play"** button to:
- Open the track in a new browser tab
- Stream the FLAC audio directly from the server
- Use browser's native audio player

### Refresh Library

Click the **"🔄 Refresh Library"** button to reload all tracks from the server (useful if you've added new files to the library folder).

## User Interface

### Header
- Shows the application title and subtitle
- Quick stats about your library

### Controls Bar
- **Refresh Button**: Reload tracks from server
- **Track Count**: Shows total number of tracks

### Track List
- Grid layout of track cards
- Responsive design (works on mobile, tablet, desktop)
- Each card shows full track information
- Hover effects for better UX

### Edit Modal
- Clean, centered modal dialog
- Form with labeled inputs
- Cancel and Save buttons
- Click outside or press ESC to close

## Keyboard Shortcuts

- **ESC**: Close the edit modal

## Styling

The web client uses:
- **Primary Color**: Brown/orange theme matching VS Code workspace
- **Clean, Modern Design**: Card-based layout
- **Responsive**: Works on all screen sizes
- **Professional**: Rounded corners, shadows, hover effects

## Technical Details

### API Calls

The web client makes these API calls:

- `GET /tracks` - Load all tracks
- `GET /tracks/:id` - Get track details (implicit)
- `PUT /tracks/:id` - Update track metadata
- `GET /stream/:id` - Stream audio (via link)

### Error Handling

- Shows error messages if server is unreachable
- Displays helpful messages when library is empty
- Console logging for debugging

### Data Flow

1. Page loads → Fetch tracks from API
2. Render tracks in UI
3. User clicks Edit → Open modal with current data
4. User submits form → PUT request to API
5. Server updates FLAC file → Returns updated track
6. Client updates UI with new data

## Troubleshooting

### "Failed to load tracks"
- Ensure the server is running
- Check server URL (default: `http://localhost:3000`)
- Check browser console for errors

### "Failed to update track"
- Check file permissions on FLAC files
- Ensure FLAC files are valid
- Check server logs for errors

### Blank page or "No tracks found"
- Verify your music library path contains FLAC files
- Check server logs for scan errors
- Try clicking "🔄 Refresh Library"

### Changes not persisting
- Ensure FLAC files are writable
- Check server has write permissions
- Verify the FLAC files are valid

## Browser Compatibility

The web client works in modern browsers:
- Chrome/Edge (recommended)
- Firefox
- Safari
- Opera

Requires JavaScript enabled.

## Development

### File Structure
```
static/
├── index.html    # Main HTML structure
├── styles.css    # All styling (CSS variables, responsive)
└── app.js        # All JavaScript (API calls, UI logic)
```

### Customization

You can customize the appearance by editing `styles.css`:
- Change `--primary-color` for different theme
- Adjust card sizes and spacing
- Modify responsive breakpoints

### Adding Features

To add new features:
1. Add API endpoint in `src/server.rs`
2. Add UI elements in `index.html`
3. Add styling in `styles.css`
4. Add logic in `app.js`
