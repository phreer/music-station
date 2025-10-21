// API base URL - adjust if server is running on different host/port
const API_BASE = window.location.origin;

let tracks = [];
let currentEditTrackId = null;
let currentView = 'tracks';

// Music player state
let currentTrackIndex = -1;
let playlist = [];
let isPlaying = false;

// Load tracks on page load
document.addEventListener('DOMContentLoaded', () => {
    loadTracks();
    setupEventListeners();
    setupMusicPlayer();
});

function setupEventListeners() {
    document.getElementById('refreshBtn').addEventListener('click', loadTracks);
    document.getElementById('editForm').addEventListener('submit', handleEditSubmit);
    
    // Tab switching
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const tabName = btn.dataset.tab;
            switchTab(tabName);
        });
    });
}

function setupMusicPlayer() {
    const audio = document.getElementById('audioPlayer');
    const playPauseBtn = document.getElementById('playPauseBtn');
    const prevBtn = document.getElementById('prevBtn');
    const nextBtn = document.getElementById('nextBtn');
    const stopBtn = document.getElementById('stopBtn');
    const progressBar = document.getElementById('progressBar');
    const volumeControl = document.getElementById('volumeControl');
    
    // Play/Pause
    playPauseBtn.addEventListener('click', togglePlayPause);
    
    // Previous track
    prevBtn.addEventListener('click', playPrevious);
    
    // Next track
    nextBtn.addEventListener('click', playNext);
    
    // Stop
    stopBtn.addEventListener('click', stopPlayback);
    
    // Progress bar
    progressBar.addEventListener('input', (e) => {
        const time = (audio.duration * e.target.value) / 100;
        audio.currentTime = time;
    });
    
    // Volume control
    volumeControl.addEventListener('input', (e) => {
        audio.volume = e.target.value / 100;
    });
    
    // Audio events
    audio.addEventListener('timeupdate', updateProgress);
    audio.addEventListener('ended', playNext);
    audio.addEventListener('loadedmetadata', updateDuration);
    audio.addEventListener('play', () => {
        isPlaying = true;
        updatePlayPauseIcon();
    });
    audio.addEventListener('pause', () => {
        isPlaying = false;
        updatePlayPauseIcon();
    });
}

function playTrack(trackId) {
    const track = tracks.find(t => t.id === trackId);
    if (!track) return;
    
    // Update playlist if not set
    if (playlist.length === 0) {
        playlist = tracks.map(t => t.id);
    }
    
    currentTrackIndex = playlist.indexOf(trackId);
    if (currentTrackIndex === -1) return;
    
    const audio = document.getElementById('audioPlayer');
    const streamUrl = `${API_BASE}/stream/${trackId}`;
    
    audio.src = streamUrl;
    audio.load();
    audio.play();
    
    // Update UI
    document.getElementById('playerTitle').textContent = track.title || 'Unknown Title';
    document.getElementById('playerArtist').textContent = track.artist || 'Unknown Artist';
    document.getElementById('musicPlayer').style.display = 'block';
    
    // Add visual feedback to current track
    highlightCurrentTrack(trackId);
}

function togglePlayPause() {
    const audio = document.getElementById('audioPlayer');
    
    if (audio.src) {
        if (isPlaying) {
            audio.pause();
        } else {
            audio.play();
        }
    } else if (tracks.length > 0) {
        // Start playing first track if none loaded
        playTrack(tracks[0].id);
    }
}

function playNext() {
    if (playlist.length === 0) return;
    
    currentTrackIndex = (currentTrackIndex + 1) % playlist.length;
    playTrack(playlist[currentTrackIndex]);
}

function playPrevious() {
    if (playlist.length === 0) return;
    
    currentTrackIndex = (currentTrackIndex - 1 + playlist.length) % playlist.length;
    playTrack(playlist[currentTrackIndex]);
}

function stopPlayback() {
    const audio = document.getElementById('audioPlayer');
    audio.pause();
    audio.currentTime = 0;
    document.getElementById('musicPlayer').style.display = 'none';
    isPlaying = false;
    updatePlayPauseIcon();
    highlightCurrentTrack(null);
}

function updateProgress() {
    const audio = document.getElementById('audioPlayer');
    const progressBar = document.getElementById('progressBar');
    const progressFill = document.getElementById('progressFill');
    const currentTime = document.getElementById('currentTime');
    
    if (audio.duration) {
        const progress = (audio.currentTime / audio.duration) * 100;
        progressBar.value = progress;
        progressFill.style.width = progress + '%';
        currentTime.textContent = formatDuration(Math.floor(audio.currentTime));
    }
}

function updateDuration() {
    const audio = document.getElementById('audioPlayer');
    const totalTime = document.getElementById('totalTime');
    
    if (audio.duration) {
        totalTime.textContent = formatDuration(Math.floor(audio.duration));
    }
}

function updatePlayPauseIcon() {
    const icon = document.getElementById('playPauseIcon');
    icon.textContent = isPlaying ? '⏸️' : '▶️';
}

function highlightCurrentTrack(trackId) {
    // Remove previous highlight
    document.querySelectorAll('.track-row').forEach(row => {
        row.classList.remove('playing');
    });
    
    // Add highlight to current track
    if (trackId) {
        const row = document.querySelector(`tr[data-track-id="${trackId}"]`);
        if (row) {
            row.classList.add('playing');
        }
    }
}

function switchTab(tabName) {
    currentView = tabName;
    
    // Update tab buttons
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tabName);
    });
    
    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    document.getElementById(`${tabName}-view`).classList.add('active');
    
    // Load content for the selected tab
    switch(tabName) {
        case 'tracks':
            if (tracks.length === 0) {
                loadTracks();
            }
            break;
        case 'albums':
            loadAlbums();
            break;
        case 'artists':
            loadArtists();
            break;
        case 'stats':
            loadStats();
            break;
    }
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

    // Update playlist
    playlist = tracks.map(t => t.id);

    trackList.innerHTML = `
        <div class="track-table-container">
            <table class="track-table">
                <thead>
                    <tr>
                        <th style="width: 40px;"></th>
                        <th>Title</th>
                        <th>Artist</th>
                        <th>Album</th>
                        <th>Duration</th>
                        <th>Size</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${tracks.map(track => createTrackRow(track)).join('')}
                </tbody>
            </table>
        </div>
    `;
}

function createTrackRow(track) {
    const title = track.title || 'Unknown Title';
    const artist = track.artist || 'Unknown Artist';
    const album = track.album || 'Unknown Album';
    const duration = track.duration_secs ? formatDuration(track.duration_secs) : '--:--';
    const fileSize = formatFileSize(track.file_size);
    const streamUrl = `${API_BASE}/stream/${track.id}`;

    return `
        <tr class="track-row" data-track-id="${track.id}">
            <td class="track-play-cell">
                <button class="play-track-btn" onclick="playTrack('${track.id}')" title="Play this track">
                    ▶️
                </button>
            </td>
            <td class="track-title-cell">${escapeHtml(title)}</td>
            <td class="track-artist-cell">${escapeHtml(artist)}</td>
            <td class="track-album-cell">${escapeHtml(album)}</td>
            <td class="track-duration-cell">${duration}</td>
            <td class="track-size-cell">${fileSize}</td>
            <td class="track-actions-cell">
                <button class="btn btn-primary btn-small" onclick="openEditModal('${track.id}')" title="Edit metadata">
                    ✏️
                </button>
                <a href="${streamUrl}" target="_blank" class="btn btn-secondary btn-small" style="text-decoration: none;" title="Download track" download>
                    💾
                </a>
            </td>
        </tr>
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

// Load albums
async function loadAlbums() {
    try {
        const response = await fetch(`${API_BASE}/albums`);
        const albums = await response.json();
        displayAlbums(albums);
    } catch (error) {
        console.error('Error loading albums:', error);
        document.getElementById('album-list').innerHTML = 
            '<p style="text-align: center; color: #ff6b6b;">Failed to load albums</p>';
    }
}

function displayAlbums(albums) {
    const albumList = document.getElementById('album-list');
    
    if (albums.length === 0) {
        albumList.innerHTML = '<p style="text-align: center; color: #b8b8b8;">No albums found</p>';
        return;
    }
    
    albumList.innerHTML = albums.map(album => `
        <div class="album-card" onclick="toggleAlbum(this)">
            <div class="album-header">
                <div style="display: flex; align-items: center; flex: 1;">
                    <div class="album-icon">💿</div>
                    <div class="album-info">
                        <h3>${escapeHtml(album.name)}</h3>
                        <div class="artist-name">🎤 ${escapeHtml(album.artist)}</div>
                    </div>
                </div>
                <div class="expand-indicator">▼</div>
            </div>
            <div class="album-meta">
                <span>📀 ${album.track_count} track${album.track_count !== 1 ? 's' : ''}</span>
                <span>⏱️ ${formatDuration(album.total_duration_secs)}</span>
            </div>
            <div class="album-tracks">
                ${album.tracks.map(track => `
                    <div class="album-track-item">
                        <span class="track-title-mini">${escapeHtml(track.title)}</span>
                        <span class="track-duration-mini">${formatDuration(track.duration_secs)}</span>
                    </div>
                `).join('')}
            </div>
        </div>
    `).join('');
}

function toggleAlbum(element) {
    element.classList.toggle('expanded');
}

// Load artists
async function loadArtists() {
    try {
        const response = await fetch(`${API_BASE}/artists`);
        const artists = await response.json();
        displayArtists(artists);
    } catch (error) {
        console.error('Error loading artists:', error);
        document.getElementById('artist-list').innerHTML = 
            '<p style="text-align: center; color: #ff6b6b;">Failed to load artists</p>';
    }
}

function displayArtists(artists) {
    const artistList = document.getElementById('artist-list');
    
    if (artists.length === 0) {
        artistList.innerHTML = '<p style="text-align: center; color: #b8b8b8;">No artists found</p>';
        return;
    }
    
    artistList.innerHTML = artists.map(artist => `
        <div class="artist-card" onclick="toggleArtist(this)">
            <div class="artist-header">
                <div class="artist-icon">🎤</div>
                <div style="flex: 1;">
                    <div class="artist-info">
                        <h3>${escapeHtml(artist.name)}</h3>
                        <div class="artist-meta">
                            <span>💿 ${artist.album_count} album${artist.album_count !== 1 ? 's' : ''}</span>
                            <span>📀 ${artist.track_count} track${artist.track_count !== 1 ? 's' : ''}</span>
                        </div>
                    </div>
                </div>
                <div class="expand-indicator">▼</div>
            </div>
            <div class="artist-albums">
                ${artist.albums.map(album => `
                    <div class="artist-album-item">
                        <h4>💿 ${escapeHtml(album.name)}</h4>
                        <div class="artist-album-meta">
                            📀 ${album.track_count} tracks · ⏱️ ${formatDuration(album.total_duration_secs)}
                        </div>
                    </div>
                `).join('')}
            </div>
        </div>
    `).join('');
}

function toggleArtist(element) {
    element.classList.toggle('expanded');
}

// Load stats
async function loadStats() {
    try {
        const response = await fetch(`${API_BASE}/stats`);
        const stats = await response.json();
        displayStats(stats);
    } catch (error) {
        console.error('Error loading stats:', error);
        document.getElementById('stats-display').innerHTML = 
            '<p style="text-align: center; color: #ff6b6b;">Failed to load statistics</p>';
    }
}

function displayStats(stats) {
    const statsDisplay = document.getElementById('stats-display');
    
    const totalDuration = formatDuration(stats.total_duration_secs);
    const totalSize = formatFileSizeBytes(stats.total_size_bytes);
    const avgTrackSize = formatFileSizeBytes(stats.total_size_bytes / stats.total_tracks);
    
    statsDisplay.innerHTML = `
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-icon">📀</div>
                <div class="stat-value">${stats.total_tracks}</div>
                <div class="stat-label">Tracks</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon">💿</div>
                <div class="stat-value">${stats.total_albums}</div>
                <div class="stat-label">Albums</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon">🎤</div>
                <div class="stat-value">${stats.total_artists}</div>
                <div class="stat-label">Artists</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon">⏱️</div>
                <div class="stat-value">${totalDuration}</div>
                <div class="stat-label">Total Duration</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon">💾</div>
                <div class="stat-value">${totalSize}</div>
                <div class="stat-label">Total Size</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon">📊</div>
                <div class="stat-value">${avgTrackSize}</div>
                <div class="stat-label">Avg Track Size</div>
            </div>
        </div>
    `;
}

function formatFileSizeBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
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
