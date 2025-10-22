// API base URL - adjust if server is running on different host/port
const API_BASE = window.location.origin;

let tracks = [];
let currentEditTrackId = null;
let currentView = 'tracks';

// Music player state
let currentTrackIndex = -1;
let playlist = [];
let isPlaying = false;

// Play queue management
let playQueue = [];
let queueIndex = -1;
let isQueueVisible = false;

// Playlist management
let playlists = [];
let currentPlaylistId = null;
let trackToAdd = null;

// Load tracks on page load
document.addEventListener('DOMContentLoaded', () => {
    loadTracks();
    loadPlaylists();
    setupEventListeners();
    setupMusicPlayer();
    setupPlaylistEventListeners();
    updateQueueDisplay();
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

function setupPlaylistEventListeners() {
    document.getElementById('createPlaylistBtn').addEventListener('click', openCreatePlaylistModal);
    document.getElementById('createPlaylistForm').addEventListener('submit', handleCreatePlaylist);
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

function playTrack(trackId, skipQueueUpdate = false) {
    const track = tracks.find(t => t.id === trackId);
    if (!track) return;
    
    // Add track to queue if not already there and not skipping queue update
    if (!skipQueueUpdate) {
        const existingQueueIndex = playQueue.indexOf(trackId);
        if (existingQueueIndex >= 0) {
            // Track already in queue, just update the index
            queueIndex = existingQueueIndex;
        } else {
            // Add track to queue
            playQueue.push(trackId);
            queueIndex = playQueue.length - 1;
            // Show queue button
            document.getElementById('queueToggleBtn').style.display = 'flex';
        }
    }
    
    // Update playlist to match queue if queue exists
    if (playQueue.length > 0) {
        playlist = playQueue.slice();
        currentTrackIndex = queueIndex;
    } else {
        playlist = tracks.map(t => t.id);
        currentTrackIndex = playlist.indexOf(trackId);
    }
    
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
    updateQueueDisplay();
}

function togglePlayPause() {
    const audio = document.getElementById('audioPlayer');
    
    if (audio.src) {
        if (isPlaying) {
            audio.pause();
        } else {
            audio.play();
        }
    } else if (playQueue.length > 0) {
        // Play first track in queue
        playTrack(playQueue[0]);
    } else if (tracks.length > 0) {
        // Start playing first track if none loaded
        playTrack(tracks[0].id);
    }
}

function playNext() {
    // Prioritize queue over playlist
    if (playQueue.length > 0) {
        queueIndex = (queueIndex + 1) % playQueue.length;
        playTrack(playQueue[queueIndex]);
    } else if (playlist.length > 0) {
        currentTrackIndex = (currentTrackIndex + 1) % playlist.length;
        playTrack(playlist[currentTrackIndex]);
    }
}

function playPrevious() {
    // Prioritize queue over playlist
    if (playQueue.length > 0) {
        queueIndex = (queueIndex - 1 + playQueue.length) % playQueue.length;
        playTrack(playQueue[queueIndex]);
    } else if (playlist.length > 0) {
        currentTrackIndex = (currentTrackIndex - 1 + playlist.length) % playlist.length;
        playTrack(playlist[currentTrackIndex]);
    }
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
        case 'playlists':
            displayPlaylists();
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
                        <th style="width: 50px;">Cover</th>
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
    const coverUrl = track.has_cover ? `${API_BASE}/cover/${track.id}` : null;

    const coverCell = coverUrl 
        ? `<img src="${coverUrl}" alt="Cover" class="track-cover-thumb" onerror="this.style.display='none'; this.parentElement.querySelector('.track-cover-placeholder').style.display='flex';">
           <div class="track-cover-placeholder" style="display: none;">📀</div>`
        : `<div class="track-cover-placeholder">📀</div>`;

    return `
        <tr class="track-row" data-track-id="${track.id}">
            <td class="track-cover-cell">
                ${coverCell}
            </td>
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
                <button class="btn-add-to-queue" onclick="addToQueue('${track.id}')" title="Add to queue">
                    📋
                </button>
                <button class="btn-add-to-playlist" onclick="openAddToPlaylistModal('${track.id}')" title="Add to playlist">
                    ➕
                </button>
                <a href="${streamUrl}" target="_blank" class="btn btn-secondary btn-small" style="text-decoration: none;" title="Download track" download>
                    💾
                </a>
            </td>
        </tr>
    `;
}

let selectedCoverFile = null;
let customFieldCounter = 0;

function openEditModal(trackId) {
    const track = tracks.find(t => t.id === trackId);
    if (!track) return;

    currentEditTrackId = trackId;
    selectedCoverFile = null;
    
    // Basic fields
    document.getElementById('editTrackId').value = trackId;
    document.getElementById('editTitle').value = track.title || '';
    document.getElementById('editArtist').value = track.artist || '';
    document.getElementById('editAlbum').value = track.album || '';
    document.getElementById('editAlbumArtist').value = track.album_artist || '';
    
    // Additional fields
    document.getElementById('editGenre').value = track.genre || '';
    document.getElementById('editYear').value = track.year || '';
    document.getElementById('editTrackNumber').value = track.track_number || '';
    document.getElementById('editDiscNumber').value = track.disc_number || '';
    document.getElementById('editComposer').value = track.composer || '';
    document.getElementById('editComment').value = track.comment || '';
    
    // Cover art
    const currentCover = document.getElementById('currentCover');
    const removeCoverBtn = document.getElementById('removeCoverBtn');
    if (track.has_cover) {
        currentCover.innerHTML = `<img src="${API_BASE}/cover/${trackId}" alt="Cover">`;
        currentCover.classList.remove('empty');
        removeCoverBtn.style.display = 'inline-block';
    } else {
        currentCover.innerHTML = '';
        currentCover.classList.add('empty');
        removeCoverBtn.style.display = 'none';
    }
    
    // Custom fields
    const customFieldsList = document.getElementById('customFieldsList');
    customFieldsList.innerHTML = '';
    customFieldCounter = 0;
    if (track.custom_fields && Object.keys(track.custom_fields).length > 0) {
        for (const [key, value] of Object.entries(track.custom_fields)) {
            addCustomFieldWithData(key, value);
        }
    }
    
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
    const album_artist = document.getElementById('editAlbumArtist').value.trim();
    const genre = document.getElementById('editGenre').value.trim();
    const year = document.getElementById('editYear').value.trim();
    const track_number = document.getElementById('editTrackNumber').value.trim();
    const disc_number = document.getElementById('editDiscNumber').value.trim();
    const composer = document.getElementById('editComposer').value.trim();
    const comment = document.getElementById('editComment').value.trim();

    // Prepare update payload (only send fields that are not empty)
    const update = {};
    if (title) update.title = title;
    if (artist) update.artist = artist;
    if (album) update.album = album;
    if (album_artist) update.album_artist = album_artist;
    if (genre) update.genre = genre;
    if (year) update.year = year;
    if (track_number) update.track_number = track_number;
    if (disc_number) update.disc_number = disc_number;
    if (composer) update.composer = composer;
    if (comment) update.comment = comment;

    // Collect custom fields
    const customFields = {};
    document.querySelectorAll('.custom-field-item').forEach(item => {
        const keyInput = item.querySelector('.custom-field-key');
        const valueInput = item.querySelector('.custom-field-value');
        if (keyInput && valueInput && keyInput.value.trim() && valueInput.value.trim()) {
            customFields[keyInput.value.trim()] = valueInput.value.trim();
        }
    });
    if (Object.keys(customFields).length > 0) {
        update.custom_fields = customFields;
    }

    try {
        // Update metadata
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

        let updatedTrack = await response.json();

        // Upload cover art if selected
        if (selectedCoverFile) {
            const formData = new FormData();
            formData.append('image', selectedCoverFile);

            const coverResponse = await fetch(`${API_BASE}/cover/${trackId}`, {
                method: 'POST',
                body: formData,
            });

            if (!coverResponse.ok) {
                throw new Error(`Failed to upload cover: ${coverResponse.status}`);
            }

            updatedTrack = await coverResponse.json();
        }
        
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

// Cover art functions
function selectCoverImage() {
    document.getElementById('coverImageInput').click();
}

document.addEventListener('DOMContentLoaded', () => {
    // ... existing code ...
    
    // Cover image input handler
    document.getElementById('coverImageInput').addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (file && file.type.startsWith('image/')) {
            selectedCoverFile = file;
            
            // Preview the image
            const reader = new FileReader();
            reader.onload = (e) => {
                const currentCover = document.getElementById('currentCover');
                currentCover.innerHTML = `<img src="${e.target.result}" alt="Cover preview">`;
                currentCover.classList.remove('empty');
                document.getElementById('removeCoverBtn').style.display = 'inline-block';
            };
            reader.readAsDataURL(file);
        }
    });
});

async function removeCover() {
    if (!currentEditTrackId) return;
    
    if (!confirm('Are you sure you want to remove the cover art?')) {
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/cover/${currentEditTrackId}`, {
            method: 'DELETE',
        });

        if (!response.ok) {
            throw new Error(`Failed to remove cover: ${response.status}`);
        }

        const updatedTrack = await response.json();
        
        // Update local track
        const index = tracks.findIndex(t => t.id === currentEditTrackId);
        if (index !== -1) {
            tracks[index] = updatedTrack;
        }

        // Update UI
        const currentCover = document.getElementById('currentCover');
        currentCover.innerHTML = '';
        currentCover.classList.add('empty');
        document.getElementById('removeCoverBtn').style.display = 'none';
        selectedCoverFile = null;

        console.log('Cover removed successfully');
    } catch (error) {
        showError(`Failed to remove cover: ${error.message}`);
        console.error('Error removing cover:', error);
    }
}

// Custom field functions
function addCustomField() {
    const customFieldsList = document.getElementById('customFieldsList');
    const fieldId = `custom-field-${customFieldCounter++}`;
    
    const fieldHtml = `
        <div class="custom-field-item" id="${fieldId}">
            <input type="text" class="custom-field-key" placeholder="Field name (e.g., LABEL)">
            <input type="text" class="custom-field-value" placeholder="Field value">
            <button type="button" onclick="removeCustomField('${fieldId}')">❌</button>
        </div>
    `;
    
    customFieldsList.insertAdjacentHTML('beforeend', fieldHtml);
}

function addCustomFieldWithData(key, value) {
    const customFieldsList = document.getElementById('customFieldsList');
    const fieldId = `custom-field-${customFieldCounter++}`;
    
    const fieldHtml = `
        <div class="custom-field-item" id="${fieldId}">
            <input type="text" class="custom-field-key" placeholder="Field name" value="${escapeHtml(key)}">
            <input type="text" class="custom-field-value" placeholder="Field value" value="${escapeHtml(value)}">
            <button type="button" onclick="removeCustomField('${fieldId}')">❌</button>
        </div>
    `;
    
    customFieldsList.insertAdjacentHTML('beforeend', fieldHtml);
}

function removeCustomField(fieldId) {
    const field = document.getElementById(fieldId);
    if (field) {
        field.remove();
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
    
    albumList.innerHTML = albums.map(album => {
        const albumTrackIds = album.tracks.map(t => t.id);
        return `
        <div class="album-card" onclick="toggleAlbum(this)">
            <div class="album-header">
                <div style="display: flex; align-items: center; flex: 1;">
                    <div class="album-icon">💿</div>
                    <div class="album-info">
                        <h3>${escapeHtml(album.name)}</h3>
                        <div class="artist-name">🎤 ${escapeHtml(album.artist)}</div>
                    </div>
                </div>
                <button class="btn btn-primary btn-small" onclick="event.stopPropagation(); playAlbum('${escapeHtml(album.name)}', '${escapeHtml(album.artist)}')" title="Play album" style="margin-right: 8px;">
                    ▶️ Play
                </button>
                <button class="btn-add-to-queue" onclick="event.stopPropagation(); addMultipleToQueue(${JSON.stringify(albumTrackIds)})" title="Add album to queue" style="margin-right: 8px;">
                    📋 Add to Queue
                </button>
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
        `;
    }).join('');
}

function toggleAlbum(element) {
    element.classList.toggle('expanded');
}

// Play entire album
function playAlbum(albumName, artistName) {
    // Find all tracks for this album
    const albumTracks = tracks.filter(t => 
        t.album === albumName && t.artist === artistName
    );
    
    if (albumTracks.length === 0) return;
    
    // Sort by track number if available
    albumTracks.sort((a, b) => {
        const aNum = parseInt(a.track_number) || 0;
        const bNum = parseInt(b.track_number) || 0;
        return aNum - bNum;
    });
    
    const albumTrackIds = albumTracks.map(t => t.id);
    
    // Add all album tracks to queue
    playQueue = albumTrackIds.slice();
    queueIndex = 0;
    playlist = albumTrackIds.slice();
    currentTrackIndex = 0;
    document.getElementById('queueToggleBtn').style.display = 'flex';
    updateQueueDisplay();
    
    // Play first track (skip queue update since we already set it up)
    playTrack(albumTrackIds[0], true);
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
        closeCreatePlaylistModal();
        closeAddToPlaylistModal();
    }
});

// ========== PLAYLIST MANAGEMENT ==========

// Load playlists from localStorage
function loadPlaylists() {
    const saved = localStorage.getItem('music_station_playlists');
    if (saved) {
        try {
            playlists = JSON.parse(saved);
        } catch (e) {
            console.error('Failed to parse playlists:', e);
            playlists = [];
        }
    } else {
        playlists = [];
    }
}

// Save playlists to localStorage
function savePlaylists() {
    localStorage.setItem('music_station_playlists', JSON.stringify(playlists));
}

// Display all playlists
function displayPlaylists() {
    const playlistList = document.getElementById('playlist-list');
    
    if (playlists.length === 0) {
        playlistList.innerHTML = `
            <div class="empty-playlist">
                <div class="empty-playlist-icon">🎵</div>
                <p>No playlists yet. Create your first playlist!</p>
            </div>
        `;
        return;
    }
    
    playlistList.innerHTML = playlists.map(pl => {
        const trackCount = pl.tracks.length;
        const totalDuration = pl.tracks.reduce((sum, trackId) => {
            const track = tracks.find(t => t.id === trackId);
            return sum + (track ? track.duration_secs : 0);
        }, 0);
        
        return `
            <div class="playlist-card" data-playlist-id="${pl.id}">
                <div class="playlist-card-header">
                    <div class="playlist-icon">🎼</div>
                    <div class="playlist-info">
                        <h3 class="playlist-name">${escapeHtml(pl.name)}</h3>
                        ${pl.description ? `<p class="playlist-description">${escapeHtml(pl.description)}</p>` : ''}
                    </div>
                </div>
                <div class="playlist-meta">
                    <span>📀 ${trackCount} track${trackCount !== 1 ? 's' : ''}</span>
                    <span>⏱️ ${formatDuration(totalDuration)}</span>
                </div>
                <div class="playlist-actions">
                    <button class="btn btn-primary btn-small" onclick="playPlaylist('${pl.id}')" ${trackCount === 0 ? 'disabled' : ''}>
                        ▶️ Play
                    </button>
                    <button class="btn btn-secondary btn-small" onclick="togglePlaylistTracks('${pl.id}')">
                        👁️ View
                    </button>
                    <button class="btn btn-secondary btn-small" onclick="deletePlaylist('${pl.id}')">
                        🗑️ Delete
                    </button>
                </div>
                <div class="playlist-tracks">
                    ${trackCount === 0 ? '<p style="text-align: center; color: #b8b8b8; padding: 12px;">No tracks in this playlist</p>' : pl.tracks.map(trackId => {
                        const track = tracks.find(t => t.id === trackId);
                        if (!track) return '';
                        return `
                            <div class="playlist-track-item">
                                <div class="playlist-track-info">
                                    <div class="playlist-track-title">${escapeHtml(track.title || 'Unknown Title')}</div>
                                    <div class="playlist-track-artist">${escapeHtml(track.artist || 'Unknown Artist')}</div>
                                </div>
                                <div class="playlist-track-actions">
                                    <button onclick="playTrackFromPlaylist('${pl.id}', '${trackId}')" title="Play">▶️</button>
                                    <button onclick="removeFromPlaylist('${pl.id}', '${trackId}')" title="Remove">❌</button>
                                </div>
                            </div>
                        `;
                    }).join('')}
                </div>
            </div>
        `;
    }).join('');
}

// Toggle playlist tracks visibility
function togglePlaylistTracks(playlistId) {
    const card = document.querySelector(`[data-playlist-id="${playlistId}"]`);
    if (card) {
        card.classList.toggle('expanded');
    }
}

// Play entire playlist
function playPlaylist(playlistId) {
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl || pl.tracks.length === 0) return;
    
    // Set current playlist
    currentPlaylistId = playlistId;
    playlist = pl.tracks.slice(); // Copy array
    currentTrackIndex = 0;
    
    // Add all playlist tracks to queue
    playQueue = pl.tracks.slice();
    queueIndex = 0;
    document.getElementById('queueToggleBtn').style.display = 'flex';
    updateQueueDisplay();
    
    // Play first track (skip queue update since we already set it up)
    playTrack(playlist[0], true);
}

// Play track from playlist
function playTrackFromPlaylist(playlistId, trackId) {
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;
    
    currentPlaylistId = playlistId;
    playlist = pl.tracks.slice();
    currentTrackIndex = playlist.indexOf(trackId);
    
    // Add all playlist tracks to queue
    playQueue = pl.tracks.slice();
    queueIndex = playlist.indexOf(trackId);
    document.getElementById('queueToggleBtn').style.display = 'flex';
    updateQueueDisplay();
    
    // Play the selected track (skip queue update since we already set it up)
    playTrack(trackId, true);
}

// Open create playlist modal
function openCreatePlaylistModal() {
    document.getElementById('createPlaylistModal').style.display = 'flex';
    document.getElementById('playlistName').value = '';
    document.getElementById('playlistDescription').value = '';
    document.getElementById('playlistName').focus();
}

// Close create playlist modal
function closeCreatePlaylistModal() {
    document.getElementById('createPlaylistModal').style.display = 'none';
}

// Handle create playlist form submission
function handleCreatePlaylist(event) {
    event.preventDefault();
    
    const name = document.getElementById('playlistName').value.trim();
    const description = document.getElementById('playlistDescription').value.trim();
    
    if (!name) {
        alert('Please enter a playlist name');
        return;
    }
    
    // Create new playlist
    const newPlaylist = {
        id: generateId(),
        name: name,
        description: description,
        tracks: [],
        createdAt: new Date().toISOString()
    };
    
    playlists.push(newPlaylist);
    savePlaylists();
    
    // Close modal and refresh display
    closeCreatePlaylistModal();
    displayPlaylists();
    
    // Show success message
    console.log('Playlist created:', newPlaylist);
}

// Open add to playlist modal
function openAddToPlaylistModal(trackId) {
    trackToAdd = trackId;
    
    const modal = document.getElementById('addToPlaylistModal');
    const listContainer = document.getElementById('playlistSelectionList');
    
    if (playlists.length === 0) {
        listContainer.innerHTML = '<p style="text-align: center; color: #999; padding: 20px;">No playlists available. Create one first!</p>';
    } else {
        listContainer.innerHTML = playlists.map(pl => `
            <div class="playlist-selection-item" onclick="addTrackToPlaylist('${pl.id}')">
                <span class="playlist-icon">🎼</span>
                <span class="playlist-name">${escapeHtml(pl.name)}</span>
                <span class="playlist-track-count">${pl.tracks.length} tracks</span>
            </div>
        `).join('');
    }
    
    modal.style.display = 'flex';
}

// Close add to playlist modal
function closeAddToPlaylistModal() {
    document.getElementById('addToPlaylistModal').style.display = 'none';
    trackToAdd = null;
}

// Add track to playlist
function addTrackToPlaylist(playlistId) {
    if (!trackToAdd) return;
    
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;
    
    // Check if track already in playlist
    if (pl.tracks.includes(trackToAdd)) {
        alert('Track is already in this playlist');
        return;
    }
    
    // Add track
    pl.tracks.push(trackToAdd);
    savePlaylists();
    
    // Close modal and refresh if on playlists view
    closeAddToPlaylistModal();
    if (currentView === 'playlists') {
        displayPlaylists();
    }
    
    // Show success message
    const track = tracks.find(t => t.id === trackToAdd);
    console.log(`Added "${track?.title}" to playlist "${pl.name}"`);
}

// Remove track from playlist
function removeFromPlaylist(playlistId, trackId) {
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;
    
    const index = pl.tracks.indexOf(trackId);
    if (index > -1) {
        pl.tracks.splice(index, 1);
        savePlaylists();
        displayPlaylists();
    }
}

// Delete playlist
function deletePlaylist(playlistId) {
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;
    
    if (confirm(`Are you sure you want to delete the playlist "${pl.name}"?`)) {
        const index = playlists.findIndex(p => p.id === playlistId);
        if (index > -1) {
            playlists.splice(index, 1);
            savePlaylists();
            displayPlaylists();
        }
    }
}

// Open create playlist from add modal
function openCreatePlaylistFromAdd() {
    closeAddToPlaylistModal();
    openCreatePlaylistModal();
}

// Generate unique ID
function generateId() {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// ========== PLAY QUEUE MANAGEMENT ==========

// Add track to queue
function addToQueue(trackId) {
    const track = tracks.find(t => t.id === trackId);
    if (!track) return;
    
    playQueue.push(trackId);
    updateQueueDisplay();
    
    // Show queue button if first item
    if (playQueue.length === 1) {
        document.getElementById('queueToggleBtn').style.display = 'flex';
    }
    
    console.log(`Added "${track.title}" to queue`);
}

// Add multiple tracks to queue
function addMultipleToQueue(trackIds) {
    trackIds.forEach(id => {
        if (!playQueue.includes(id)) {
            playQueue.push(id);
        }
    });
    updateQueueDisplay();
    
    if (playQueue.length > 0) {
        document.getElementById('queueToggleBtn').style.display = 'flex';
    }
}

// Remove track from queue
function removeFromQueue(index) {
    if (index < 0 || index >= playQueue.length) return;
    
    // Adjust queue index if needed
    if (index === queueIndex) {
        // Removing currently playing track
        queueIndex = -1;
    } else if (index < queueIndex) {
        queueIndex--;
    }
    
    playQueue.splice(index, 1);
    updateQueueDisplay();
    
    // Hide queue button if empty
    if (playQueue.length === 0) {
        document.getElementById('queueToggleBtn').style.display = 'none';
        queueIndex = -1;
    }
}

// Clear entire queue
function clearQueue() {
    if (playQueue.length === 0) return;
    
    if (confirm(`Clear all ${playQueue.length} tracks from queue?`)) {
        playQueue = [];
        queueIndex = -1;
        updateQueueDisplay();
        document.getElementById('queueToggleBtn').style.display = 'none';
    }
}

// Play track from queue
function playFromQueue(index) {
    if (index < 0 || index >= playQueue.length) return;
    
    queueIndex = index;
    playTrack(playQueue[index]);
}

// Toggle queue visibility
function toggleQueue() {
    isQueueVisible = !isQueueVisible;
    const panel = document.getElementById('playQueuePanel');
    
    if (isQueueVisible) {
        panel.style.display = 'flex';
        document.body.style.paddingRight = '350px';
    } else {
        panel.style.display = 'none';
        document.body.style.paddingRight = '0';
    }
}

// Update queue display
function updateQueueDisplay() {
    const queueList = document.getElementById('queueList');
    const queueCount = document.getElementById('queueCount');
    const queueDuration = document.getElementById('queueDuration');
    const queueBadge = document.getElementById('queueBadge');
    
    // Update badge
    queueBadge.textContent = playQueue.length;
    
    if (playQueue.length === 0) {
        queueList.innerHTML = `
            <div class="queue-empty">
                <div class="queue-empty-icon">🎵</div>
                <p>Queue is empty</p>
                <p class="queue-empty-hint">Add tracks to start playing</p>
            </div>
        `;
        queueCount.textContent = '0 tracks';
        queueDuration.textContent = '0:00';
        return;
    }
    
    // Calculate total duration
    let totalDuration = 0;
    playQueue.forEach(trackId => {
        const track = tracks.find(t => t.id === trackId);
        if (track && track.duration_secs) {
            totalDuration += track.duration_secs;
        }
    });
    
    // Update info
    queueCount.textContent = `${playQueue.length} track${playQueue.length !== 1 ? 's' : ''}`;
    queueDuration.textContent = formatDuration(totalDuration);
    
    // Render queue items
    queueList.innerHTML = playQueue.map((trackId, index) => {
        const track = tracks.find(t => t.id === trackId);
        if (!track) return '';
        
        const isCurrentTrack = index === queueIndex;
        const coverUrl = track.has_cover ? `${API_BASE}/cover/${trackId}` : null;
        
        return `
            <div class="queue-item ${isCurrentTrack ? 'playing' : ''}" onclick="playFromQueue(${index})">
                <div class="queue-item-number">${index + 1}</div>
                <div class="queue-item-cover">
                    ${coverUrl 
                        ? `<img src="${coverUrl}" alt="Cover" onerror="this.style.display='none';">` 
                        : '📀'}
                </div>
                <div class="queue-item-info">
                    <div class="queue-item-title">${escapeHtml(track.title || 'Unknown Title')}</div>
                    <div class="queue-item-artist">${escapeHtml(track.artist || 'Unknown Artist')}</div>
                </div>
                <div class="queue-item-duration">${track.duration_secs ? formatDuration(track.duration_secs) : '--:--'}</div>
                <button class="queue-item-remove" onclick="event.stopPropagation(); removeFromQueue(${index})" title="Remove from queue">
                    ✖️
                </button>
            </div>
        `;
    }).join('');
}

// Move track up in queue
function moveQueueItemUp(index) {
    if (index <= 0 || index >= playQueue.length) return;
    
    [playQueue[index - 1], playQueue[index]] = [playQueue[index], playQueue[index - 1]];
    
    // Adjust current index if needed
    if (queueIndex === index) {
        queueIndex--;
    } else if (queueIndex === index - 1) {
        queueIndex++;
    }
    
    updateQueueDisplay();
}

// Move track down in queue
function moveQueueItemDown(index) {
    if (index < 0 || index >= playQueue.length - 1) return;
    
    [playQueue[index], playQueue[index + 1]] = [playQueue[index + 1], playQueue[index]];
    
    // Adjust current index if needed
    if (queueIndex === index) {
        queueIndex++;
    } else if (queueIndex === index + 1) {
        queueIndex--;
    }
    
    updateQueueDisplay();
}
