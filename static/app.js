// API base URL - adjust if server is running on different host/port
const API_BASE = window.location.origin;

let tracks = [];
let currentEditTrackId = null;

// Load tracks on page load
document.addEventListener('DOMContentLoaded', () => {
    loadTracks();
    setupEventListeners();
});

function setupEventListeners() {
    document.getElementById('refreshBtn').addEventListener('click', loadTracks);
    document.getElementById('editForm').addEventListener('submit', handleEditSubmit);
}

async function loadTracks() {
    showLoading();
    hideError();

    try {
        const response = await fetch(`${API_BASE}/tracks`);
        
        if (!response.ok) {
            throw new Error(`Server returned ${response.status}: ${response.statusText}`);
        }

        tracks = await response.json();
        renderTracks();
        updateTrackCount();
    } catch (error) {
        showError(`Failed to load tracks: ${error.message}`);
        console.error('Error loading tracks:', error);
    } finally {
        hideLoading();
    }
}

function renderTracks() {
    const trackList = document.getElementById('trackList');
    
    if (tracks.length === 0) {
        trackList.innerHTML = `
            <div style="text-align: center; padding: 40px; background: white; border-radius: 8px;">
                <p style="font-size: 1.2em; color: #666;">No tracks found in the library.</p>
                <p style="color: #999; margin-top: 10px;">Make sure your server is configured with a music folder containing FLAC files.</p>
            </div>
        `;
        return;
    }

    trackList.innerHTML = tracks.map(track => createTrackCard(track)).join('');
}

function createTrackCard(track) {
    const title = track.title || 'Unknown Title';
    const artist = track.artist || 'Unknown Artist';
    const album = track.album || 'Unknown Album';
    const duration = track.duration_secs ? formatDuration(track.duration_secs) : 'Unknown';
    const fileSize = formatFileSize(track.file_size);
    const streamUrl = `${API_BASE}/stream/${track.id}`;

    return `
        <div class="track-card" data-track-id="${track.id}">
            <div class="track-info">
                <div class="track-title">${escapeHtml(title)}</div>
                <div class="track-artist">🎤 ${escapeHtml(artist)}</div>
                <div class="track-album">💿 ${escapeHtml(album)}</div>
                <div class="track-meta">
                    <span>⏱️ ${duration}</span>
                    <span>📦 ${fileSize}</span>
                    <span style="font-family: monospace; font-size: 0.8em;">ID: ${track.id.substring(0, 8)}...</span>
                </div>
            </div>
            <div class="track-actions">
                <button class="btn btn-primary btn-small" onclick="openEditModal('${track.id}')">
                    ✏️ Edit
                </button>
                <a href="${streamUrl}" target="_blank" class="btn btn-secondary btn-small" style="text-decoration: none; text-align: center;">
                    ▶️ Play
                </a>
            </div>
        </div>
    `;
}

function openEditModal(trackId) {
    const track = tracks.find(t => t.id === trackId);
    if (!track) return;

    currentEditTrackId = trackId;
    
    document.getElementById('editTrackId').value = trackId;
    document.getElementById('editTitle').value = track.title || '';
    document.getElementById('editArtist').value = track.artist || '';
    document.getElementById('editAlbum').value = track.album || '';
    
    document.getElementById('editModal').style.display = 'flex';
}

function closeEditModal() {
    document.getElementById('editModal').style.display = 'none';
    currentEditTrackId = null;
}

async function handleEditSubmit(event) {
    event.preventDefault();
    
    const trackId = document.getElementById('editTrackId').value;
    const title = document.getElementById('editTitle').value.trim();
    const artist = document.getElementById('editArtist').value.trim();
    const album = document.getElementById('editAlbum').value.trim();

    // Prepare update payload (only send fields that are not empty)
    const update = {};
    if (title) update.title = title;
    if (artist) update.artist = artist;
    if (album) update.album = album;

    try {
        const response = await fetch(`${API_BASE}/tracks/${trackId}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(update),
        });

        if (!response.ok) {
            throw new Error(`Server returned ${response.status}: ${response.statusText}`);
        }

        const updatedTrack = await response.json();
        
        // Update local tracks array
        const index = tracks.findIndex(t => t.id === trackId);
        if (index !== -1) {
            tracks[index] = updatedTrack;
        }

        // Re-render tracks
        renderTracks();
        
        // Close modal
        closeEditModal();
        
        // Show success message (could add a toast notification here)
        console.log('Track updated successfully:', updatedTrack);
        
    } catch (error) {
        showError(`Failed to update track: ${error.message}`);
        console.error('Error updating track:', error);
    }
}

function formatDuration(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function formatFileSize(bytes) {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(2)} MB`;
}

function updateTrackCount() {
    const countElement = document.getElementById('trackCount');
    countElement.textContent = `${tracks.length} track${tracks.length !== 1 ? 's' : ''} in library`;
}

function showLoading() {
    document.getElementById('loading').style.display = 'block';
    document.getElementById('trackList').style.display = 'none';
}

function hideLoading() {
    document.getElementById('loading').style.display = 'none';
    document.getElementById('trackList').style.display = 'block';
}

function showError(message) {
    const errorElement = document.getElementById('error');
    errorElement.textContent = message;
    errorElement.style.display = 'block';
}

function hideError() {
    document.getElementById('error').style.display = 'none';
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Close modal when clicking outside
window.addEventListener('click', (event) => {
    const modal = document.getElementById('editModal');
    if (event.target === modal) {
        closeEditModal();
    }
});

// Close modal with Escape key
document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
        closeEditModal();
    }
});
