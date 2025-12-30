// API base URL - adjust if server is running on different host/port
const API_BASE = window.location.origin;

let fullTracks = [];
let tracks = [];
let currentEditTrackId = null;
let currentView = 'tracks';

// Pagination state
let currentPage = 1;
let pageSize = 100; // Number of tracks to load per page
let totalTracks = 0;
let isLoadingMore = false;
let allTracksLoaded = false;
let searchQuery = '';
let filteredTracks = [];

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
    initTheme();
    loadTracks();
    loadPlaylists();
    setupEventListeners();
    setupMusicPlayer();
    setupPlaylistEventListeners();
    updateQueueDisplay();
    setupInfiniteScroll();
    setupResizableSidebars();
});

function setupEventListeners() {
    document.getElementById('refreshBtn').addEventListener('click', () => {
        // Reset pagination and reload from scratch
        currentPage = 1;
        allTracksLoaded = false;
        loadTracks(false);
    });
    document.getElementById('autoFetchLyricsBtn').addEventListener('click', startAutoFetchLyrics);
    document.getElementById('editForm').addEventListener('submit', handleEditSubmit);
    document.getElementById('themeToggle').addEventListener('click', toggleTheme);

    // Search input with debouncing
    let searchTimeout;
    document.getElementById('searchInput').addEventListener('input', (e) => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            searchQuery = e.target.value.toLowerCase().trim();
            filterAndRenderTracks();
        }, 300); // 300ms debounce
    });

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
    const track = fullTracks.find(t => t.id === trackId);
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
        const displayTracks = searchQuery ? filteredTracks : tracks;
        playlist = displayTracks.map(t => t.id);
        currentTrackIndex = playlist.indexOf(trackId);
    }

    const audio = document.getElementById('audioPlayer');
    const streamUrl = `${API_BASE}/stream/${trackId}`;

    audio.src = streamUrl;
    audio.load();
    audio.play();

    // Increment play count on server
    fetch(`${API_BASE}/tracks/${trackId}/play`, { method: 'POST' })
        .then(response => {
            if (response.ok) {
                return response.json();
            }
        })
        .then(newCount => {
            if (newCount !== undefined) {
                // Update local track object in both arrays
                const fullIdx = fullTracks.findIndex(t => t.id === trackId);
                if (fullIdx !== -1) {
                    fullTracks[fullIdx].play_count = newCount;

                    // Also update in tracks if present
                    const trackIdx = tracks.findIndex(t => t.id === trackId);
                    if (trackIdx !== -1) {
                        tracks[trackIdx].play_count = newCount;
                    }

                    // If we are in stats view, we might want to refresh it
                    if (currentView === 'stats') {
                        loadStats();
                    }

                    // Update UI if visible
                    const row = document.querySelector(`tr[data-track-id="${trackId}"] .track-plays-cell`);
                    if (row) {
                        row.textContent = newCount;
                    }
                }
            }
        })
        .catch(err => console.error('Failed to increment play count:', err));

    // Update UI
    document.getElementById('playerTitle').textContent = track.title || 'Unknown Title';
    document.getElementById('playerArtist').textContent = track.artist || 'Unknown Artist';
    document.getElementById('musicPlayer').style.display = 'flex';
    document.body.classList.add('player-visible');

    // Add visual feedback to current track
    highlightCurrentTrack(trackId);
    updateQueueDisplay();

    // Load lyrics if available
    if (track.has_lyrics) {
        loadLyricsForPlayer(trackId);
    } else {
        hidePlayerLyrics();
    }
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
    document.body.classList.remove('player-visible');
    isPlaying = false;
    updatePlayPauseIcon();
    highlightCurrentTrack(null);
    hidePlayerLyrics();
    hasLyricsAvailable = false;
    document.getElementById('lyricsToggleBtn').style.display = 'none';
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

        // Update synchronized lyrics
        updateSynchronizedLyrics(audio.currentTime);
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
    icon.innerHTML = isPlaying ? '<i data-lucide="pause"></i>' : '<i data-lucide="play"></i>';
    lucide.createIcons();
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
    switch (tabName) {
        case 'tracks':
            if (tracks.length === 0) {
                loadTracks(false);
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

async function loadTracks(append = false) {
    if (isLoadingMore || (allTracksLoaded && append)) return;

    if (!append) {
        showLoading();
        tracks = [];
        fullTracks = [];
        currentPage = 1;
        allTracksLoaded = false;
    } else {
        isLoadingMore = true;
    }

    hideError();

    try {
        let allTracks;
        if (fullTracks.length === 0) {
            const response = await fetch(`${API_BASE}/tracks`);

            if (!response.ok) {
                throw new Error(`Server returned ${response.status}: ${response.statusText}`);
            }

            allTracks = await response.json();
            fullTracks = allTracks;
        } else {
            allTracks = fullTracks;
        }

        totalTracks = allTracks.length;

        // Calculate pagination
        const startIndex = (currentPage - 1) * pageSize;
        const endIndex = Math.min(startIndex + pageSize, allTracks.length);
        const newTracks = allTracks.slice(startIndex, endIndex);

        if (append) {
            tracks = [...tracks, ...newTracks];
        } else {
            tracks = newTracks;
        }

        // Check if all tracks are loaded
        if (tracks.length >= totalTracks) {
            allTracksLoaded = true;
        }

        renderTracks(append);
        updateTrackCount();

        currentPage++;
    } catch (error) {
        showError(`Failed to load tracks: ${error.message}`);
        console.error('Error loading tracks:', error);
    } finally {
        hideLoading();
        isLoadingMore = false;
    }
}

// Setup infinite scroll
function setupInfiniteScroll() {
    // Use window scroll instead of element scroll for better cross-browser compatibility
    const handleScroll = () => {
        if (currentView !== 'tracks' || allTracksLoaded || isLoadingMore || searchQuery) return;

        // Get the scroll position relative to the document
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // Load more when scrolled to 80% of the content
        if (scrollTop + windowHeight >= documentHeight * 0.8) {
            loadTracks(true);
        }
    };

    // Add scroll listener to window for cross-browser compatibility
    window.addEventListener('scroll', handleScroll, { passive: true });

    // Also check on resize in case content changes
    window.addEventListener('resize', handleScroll, { passive: true });
}

function setupResizableSidebars() {
    const lyricsSidebar = document.getElementById('playerLyricsContainer');
    const lyricsHandle = document.getElementById('lyricsResizeHandle');

    // Load saved width
    const savedWidth = localStorage.getItem('lyrics_sidebar_width');
    if (savedWidth) {
        lyricsSidebar.style.width = savedWidth;
    }

    if (lyricsHandle) {
        let isResizing = false;

        lyricsHandle.addEventListener('mousedown', (e) => {
            isResizing = true;
            document.body.classList.add('resizing');
            lyricsHandle.classList.add('resizing');
            e.preventDefault();
        });

        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;

            let newWidth = e.clientX;
            // Constraints
            if (newWidth < 250) newWidth = 250;
            if (newWidth > 800) newWidth = 800;

            lyricsSidebar.style.width = `${newWidth}px`;
            if (lyricsVisible) {
                document.body.style.paddingLeft = `${newWidth}px`;
            }
        });

        document.addEventListener('mouseup', () => {
            if (isResizing) {
                isResizing = false;
                document.body.classList.remove('resizing');
                lyricsHandle.classList.remove('resizing');
                // Save preference
                localStorage.setItem('lyrics_sidebar_width', lyricsSidebar.style.width);
            }
        });
    }
}

// Filter and render tracks based on search query
function filterAndRenderTracks() {
    if (!searchQuery) {
        // No search query, render all loaded tracks
        filteredTracks = tracks;
        renderTracks(false);
        updateTrackCount();
        return;
    }

    // Filter tracks based on search query
    filteredTracks = fullTracks.filter(track => {
        const title = (track.title || '').toLowerCase();
        const artist = (track.artist || '').toLowerCase();
        const album = (track.album || '').toLowerCase();
        const genre = (track.genre || '').toLowerCase();

        return title.includes(searchQuery) ||
            artist.includes(searchQuery) ||
            album.includes(searchQuery) ||
            genre.includes(searchQuery);
    });

    // Render filtered tracks
    renderFilteredTracks();
    updateTrackCount();
}

// Render filtered tracks (for search)
function renderFilteredTracks() {
    const trackList = document.getElementById('trackList');

    if (filteredTracks.length === 0) {
        trackList.innerHTML = `
            <div style="text-align: center; padding: 40px; background: white; border-radius: 8px;">
                <p style="font-size: 1.2em; color: #666;">No tracks found matching "${escapeHtml(searchQuery)}"</p>
                <p style="color: #999; margin-top: 10px;">Try a different search term</p>
            </div>
        `;
        return;
    }

    trackList.innerHTML = `
        <div class="track-table-container">
            <table class="track-table">
                <thead>
                    <tr>
                        <th style="width: 50px;">Cover</th>
                        <th style="width: 40px;"></th>
                        <th>Track</th>
                        <th>Album</th>
                        <th>Duration</th>
                        <th>Plays</th>
                        <th>Size</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody id="trackTableBody">
                    ${filteredTracks.map(track => createTrackRow(track)).join('')}
                </tbody>
            </table>
        </div>
    `;
    lucide.createIcons();
}

function renderTracks(append = false) {
    const trackList = document.getElementById('trackList');

    // Use filtered tracks if search is active
    const displayTracks = searchQuery ? filteredTracks : tracks;

    if (displayTracks.length === 0 && !append) {
        trackList.innerHTML = `
            <div style="text-align: center; padding: 40px; background: white; border-radius: 8px;">
                <p style="font-size: 1.2em; color: #666;">No tracks found in the library.</p>
                <p style="color: #999; margin-top: 10px;">Make sure your server is configured with a music folder containing FLAC files.</p>
            </div>
        `;
        return;
    }

    if (!append || searchQuery) {
        // Initial render or search mode - create table structure
        trackList.innerHTML = `
            <div class="track-table-container">
                <table class="track-table">
                    <thead>
                        <tr>
                            <th style="width: 50px;">Cover</th>
                            <th style="width: 40px;"></th>
                            <th>Track</th>
                            <th>Album</th>
                            <th>Duration</th>
                            <th>Plays</th>
                            <th>Size</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="trackTableBody">
                        ${displayTracks.map(track => createTrackRow(track)).join('')}
                    </tbody>
                </table>
                ${!allTracksLoaded && !searchQuery ? `
                    <div id="loadMoreIndicator" style="text-align: center; padding: 20px;">
                        <div style="color: #666; margin-bottom: 10px;">Scroll down to load more tracks...</div>
                        <button class="btn btn-secondary btn-small" onclick="loadTracks(true)" style="margin-top: 10px;">
                            <i data-lucide="chevron-down"></i> Load More Tracks
                        </button>
                    </div>
                ` : ''}
            </div>
        `;
    } else {
        // Append new tracks to existing table (infinite scroll mode)
        const tbody = document.getElementById('trackTableBody');
        if (tbody) {
            const startIndex = tracks.length - pageSize;
            const newTracks = tracks.slice(Math.max(0, startIndex));

            // Create a temporary tbody element to properly parse <tr> tags
            const tempTbody = document.createElement('tbody');
            tempTbody.innerHTML = newTracks.map(track => createTrackRow(track)).join('');

            // Use DocumentFragment for better performance
            const fragment = document.createDocumentFragment();
            while (tempTbody.firstChild) {
                fragment.appendChild(tempTbody.firstChild);
            }
            tbody.appendChild(fragment);

            // Update or remove load more indicator
            const indicator = document.getElementById('loadMoreIndicator');
            if (indicator) {
                if (allTracksLoaded) {
                    indicator.innerHTML = `<div style="color: #999; padding: 20px;"><i data-lucide="check"></i> All ${totalTracks} tracks loaded</div>`;
                } else {
                    indicator.innerHTML = `
                        <div style="color: #666; margin-bottom: 10px;">Loaded ${tracks.length} of ${totalTracks} tracks...</div>
                        <button class="btn btn-secondary btn-small" onclick="loadTracks(true)">
                            <i data-lucide="chevron-down"></i> Load More Tracks
                        </button>
                    `;
                }
            }
        }
    }
    lucide.createIcons();
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
           <div class="track-cover-placeholder" style="display: none;"><i data-lucide="disc"></i></div>`
        : `<div class="track-cover-placeholder"><i data-lucide="disc"></i></div>`;

    return `
        <tr class="track-row" data-track-id="${track.id}">
            <td class="track-cover-cell">
                ${coverCell}
            </td>
            <td class="track-play-cell">
                <button class="play-track-btn" onclick="playTrack('${track.id}')" title="Play this track">
                    <i data-lucide="play"></i>
                </button>
            </td>
            <td class="track-info-cell">
                <div class="track-title-main">${escapeHtml(title)}</div>
                <div class="track-artist-sub">${escapeHtml(artist)}</div>
            </td>
            <td class="track-album-cell">${escapeHtml(album)}</td>
            <td class="track-duration-cell">${duration}</td>
            <td class="track-plays-cell">${track.play_count || 0}</td>
            <td class="track-size-cell">${fileSize}</td>
            <td class="track-actions-cell">
                <button class="btn-action btn-edit" onclick="openEditModal('${track.id}')" title="Edit metadata">
                    <i data-lucide="edit-3"></i>
                </button>
                <button class="btn-action ${track.has_lyrics ? 'btn-lyrics-active' : 'btn-lyrics'}" onclick="openLyricsModal('${track.id}')" title="${track.has_lyrics ? 'View/Edit lyrics' : 'Add lyrics'}">
                    <i data-lucide="${track.has_lyrics ? 'file-text' : 'file-plus'}"></i>
                </button>
                <button class="btn-action btn-queue" onclick="addToQueue('${track.id}')" title="Add to queue">
                    <i data-lucide="list-plus"></i>
                </button>
                <button class="btn-action btn-playlist" onclick="openAddToPlaylistModal('${track.id}')" title="Add to playlist">
                    <i data-lucide="plus"></i>
                </button>
                <a href="${streamUrl}" target="_blank" class="btn-action btn-download" title="Download track" download>
                    <i data-lucide="download"></i>
                </a>
            </td>
        </tr>
    `;
}

let selectedCoverFile = null;
let customFieldCounter = 0;

function openEditModal(trackId) {
    const track = fullTracks.find(t => t.id === trackId);
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

        // Update local tracks arrays
        const fullIndex = fullTracks.findIndex(t => t.id === trackId);
        if (fullIndex !== -1) {
            fullTracks[fullIndex] = updatedTrack;
        }

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

        // Update local tracks arrays
        const fullIndex = fullTracks.findIndex(t => t.id === currentEditTrackId);
        if (fullIndex !== -1) {
            fullTracks[fullIndex] = updatedTrack;
        }

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
            <button type="button" onclick="removeCustomField('${fieldId}')" title="Remove field"><i data-lucide="x"></i></button>
        </div>
    `;

    customFieldsList.insertAdjacentHTML('beforeend', fieldHtml);
    lucide.createIcons();
}

function addCustomFieldWithData(key, value) {
    const customFieldsList = document.getElementById('customFieldsList');
    const fieldId = `custom-field-${customFieldCounter++}`;

    const fieldHtml = `
        <div class="custom-field-item" id="${fieldId}">
            <input type="text" class="custom-field-key" placeholder="Field name" value="${escapeHtml(key)}">
            <input type="text" class="custom-field-value" placeholder="Field value" value="${escapeHtml(value)}">
            <button type="button" onclick="removeCustomField('${fieldId}')" title="Remove field"><i data-lucide="x"></i></button>
        </div>
    `;

    customFieldsList.insertAdjacentHTML('beforeend', fieldHtml);
    lucide.createIcons();
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

    if (searchQuery) {
        countElement.textContent = `${filteredTracks.length} of ${totalTracks} track${totalTracks !== 1 ? 's' : ''} match`;
    } else if (allTracksLoaded) {
        countElement.textContent = `${totalTracks} track${totalTracks !== 1 ? 's' : ''} in library`;
    } else {
        countElement.textContent = `${tracks.length} of ${totalTracks} track${totalTracks !== 1 ? 's' : ''} loaded`;
    }
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
        const firstTrack = album.tracks[0];
        const coverUrl = firstTrack && firstTrack.has_cover ? `${API_BASE}/cover/${firstTrack.id}` : null;

        return `
        <div class="album-card" onclick="toggleAlbum(this)">
            <div class="album-cover-wrapper">
                ${coverUrl
                ? `<img src="${coverUrl}" alt="${escapeHtml(album.name)}" class="album-cover-img" onerror="this.style.display='none'; this.parentElement.querySelector('.album-cover-placeholder').style.display='flex';">`
                : ''}
                <div class="album-cover-placeholder" ${coverUrl ? 'style="display: none;"' : ''}>
                    <i data-lucide="disc"></i>
                </div>
                <div class="album-card-overlay">
                    <button class="album-overlay-btn" onclick="event.stopPropagation(); playAlbum('${escapeHtml(album.name)}', '${escapeHtml(album.artist)}')" title="Play album">
                        <i data-lucide="play"></i>
                    </button>
                    <button class="album-overlay-btn" onclick="event.stopPropagation(); addMultipleToQueue(${JSON.stringify(albumTrackIds)})" title="Add to queue">
                        <i data-lucide="list-plus"></i>
                    </button>
                </div>
            </div>
            <div class="album-card-content">
                <h3>${escapeHtml(album.name)}</h3>
                <div class="artist-name">${escapeHtml(album.artist)}</div>
                
                <div class="album-details-meta">
                    <span><i data-lucide="music"></i> ${album.track_count} tracks</span>
                    <span><i data-lucide="clock"></i> ${formatDuration(album.total_duration_secs)}</span>
                </div>

                <div class="album-tracks-list">
                    ${album.tracks.map((track, index) => `
                        <div class="album-track-row" onclick="event.stopPropagation(); playTrack('${track.id}')">
                            <span class="album-track-number">${index + 1}</span>
                            <span class="album-track-title">${escapeHtml(track.title)}</span>
                            <span class="album-track-duration">${formatDuration(track.duration_secs)}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>
        `;
    }).join('');
    lucide.createIcons();
}

function toggleAlbum(element) {
    const isExpanded = element.classList.contains('expanded');

    // Close all other expanded albums
    document.querySelectorAll('.album-card.expanded').forEach(card => {
        card.classList.remove('expanded');
    });

    if (!isExpanded) {
        element.classList.add('expanded');
        element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
}

// Play entire album
function playAlbum(albumName, artistName) {
    // Find all tracks for this album
    const albumTracks = fullTracks.filter(t =>
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

let currentArtists = [];

function displayArtists(artists) {
    const artistList = document.getElementById('artist-list');
    currentArtists = artists;

    if (artists.length === 0) {
        artistList.innerHTML = '<p style="text-align: center; color: #b8b8b8;">No artists found</p>';
        return;
    }

    artistList.innerHTML = artists.map((artist, index) => {
        const firstAlbum = artist.albums[0];
        const firstTrack = firstAlbum ? firstAlbum.tracks[0] : null;
        const artistImageUrl = firstTrack && firstTrack.has_cover ? `${API_BASE}/cover/${firstTrack.id}` : null;

        return `
        <div class="artist-card" id="artist-card-${index}" onclick="toggleArtist(this, ${index})">
            <div class="artist-image-wrapper">
                ${artistImageUrl 
                    ? `<img src="${artistImageUrl}" alt="${escapeHtml(artist.name)}" class="artist-profile-img" onerror="this.style.display='none'; this.parentElement.querySelector('.artist-icon-large').style.display='flex';">` 
                    : ''}
                <div class="artist-icon-large" ${artistImageUrl ? 'style="display: none;"' : ''}><i data-lucide="user"></i></div>
            </div>
            <div class="artist-details-content">
                <h3>${escapeHtml(artist.name)}</h3>
                <div class="artist-card-meta">
                    <span>${artist.album_count} albums</span> • 
                    <span>${artist.track_count} tracks</span>
                </div>
                
                <div class="artist-view-selector" style="display: none;">
                    <button class="view-btn active" onclick="event.stopPropagation(); switchArtistView(${index}, 'albums', this)">Albums</button>
                    <button class="view-btn" onclick="event.stopPropagation(); switchArtistView(${index}, 'tracks', this)">All Tracks</button>
                </div>

                <div id="artist-content-${index}" class="artist-expanded-content" style="display: none;">
                    <!-- Content will be rendered here -->
                </div>
            </div>
        </div>
    `}).join('');
    lucide.createIcons();
}

function toggleArtist(element, index) {
    const isExpanded = element.classList.contains('expanded');

    // Close all other expanded artists
    document.querySelectorAll('.artist-card.expanded').forEach(card => {
        if (card !== element) {
            card.classList.remove('expanded');
            card.querySelector('.artist-view-selector').style.display = 'none';
            card.querySelector('.artist-expanded-content').style.display = 'none';
        }
    });

    if (!isExpanded) {
        element.classList.add('expanded');
        element.querySelector('.artist-view-selector').style.display = 'flex';
        element.querySelector('.artist-expanded-content').style.display = 'block';
        
        // Default to albums view
        const albumsBtn = element.querySelector('.view-btn.active');
        switchArtistView(index, 'albums', albumsBtn);
        
        element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    } else {
        element.classList.remove('expanded');
        element.querySelector('.artist-view-selector').style.display = 'none';
        element.querySelector('.artist-expanded-content').style.display = 'none';
    }
}

function switchArtistView(index, mode, btnElement) {
    const artist = currentArtists[index];
    const contentContainer = document.getElementById(`artist-content-${index}`);
    
    // Update active button
    const selector = btnElement.parentElement;
    selector.querySelectorAll('.view-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');

    if (mode === 'albums') {
        renderArtistAlbums(index, contentContainer);
    } else {
        renderArtistAllTracks(index, contentContainer);
    }
    lucide.createIcons();
}

function renderArtistAlbums(index, container) {
    const artist = currentArtists[index];
    
    container.innerHTML = `
        <div class="artist-albums-grid">
            ${artist.albums.map((album, albumIndex) => {
                const firstTrack = album.tracks[0];
                const coverUrl = firstTrack && firstTrack.has_cover ? `${API_BASE}/cover/${firstTrack.id}` : null;
                
                return `
                <div class="artist-album-mini-card" onclick="event.stopPropagation(); toggleArtistAlbumTracks(${index}, ${albumIndex}, this)">
                    <div class="artist-album-mini-header">
                        <div class="artist-album-mini-icon">
                            ${coverUrl 
                                ? `<img src="${coverUrl}" alt="${escapeHtml(album.name)}" class="mini-album-cover" onerror="this.style.display='none'; this.parentElement.querySelector('i').style.display='block';">` 
                                : ''}
                            <i data-lucide="disc" ${coverUrl ? 'style="display: none;"' : ''}></i>
                        </div>
                        <div class="artist-album-mini-info">
                            <h4>${escapeHtml(album.name)}</h4>
                            <div class="artist-album-mini-meta">
                                ${album.track_count} tracks • ${formatDuration(album.total_duration_secs)}
                            </div>
                        </div>
                        <div class="artist-album-mini-actions">
                            <button class="btn-action btn-small" onclick="event.stopPropagation(); playAlbum('${escapeHtml(album.name)}', '${escapeHtml(artist.name)}')" title="Play album">
                                <i data-lucide="play"></i>
                            </button>
                        </div>
                        <div class="expand-indicator"><i data-lucide="chevron-down"></i></div>
                    </div>
                    <div id="artist-${index}-album-${albumIndex}-tracks" class="artist-album-tracks-container">
                        <div class="artist-album-tracks-inner">
                            <div class="loading-mini"><i data-lucide="refresh-cw" class="spin"></i> Loading tracks...</div>
                        </div>
                    </div>
                </div>
            `}).join('')}
        </div>
    `;
}

async function toggleArtistAlbumTracks(artistIndex, albumIndex, element) {
    const isExpanded = element.classList.contains('expanded');
    const tracksContainer = document.getElementById(`artist-${artistIndex}-album-${albumIndex}-tracks`);

    if (!isExpanded) {
        element.classList.add('expanded');
        
        const artist = currentArtists[artistIndex];
        const album = artist.albums[albumIndex];
        
        // Filter tracks from fullTracks
        const albumTracks = fullTracks.filter(t => 
            t.artist === artist.name && t.album === album.name
        );

        if (albumTracks.length > 0) {
            tracksContainer.innerHTML = `<div class="artist-album-tracks-inner">` + albumTracks.map((track, i) => `
                <div class="mini-track-item" onclick="event.stopPropagation(); playTrack('${track.id}')">
                    <div class="mini-track-num">${i + 1}</div>
                    <div class="mini-track-title">${escapeHtml(track.title)}</div>
                    <div class="mini-track-duration">${formatDuration(track.duration_secs)}</div>
                    <div class="mini-track-actions">
                        <button class="btn-action btn-small" onclick="event.stopPropagation(); addToQueue('${track.id}')" title="Add to queue">
                            <i data-lucide="list-plus"></i>
                        </button>
                    </div>
                </div>
            `).join('') + `</div>`;
        } else {
            tracksContainer.innerHTML = '<div class="artist-album-tracks-inner"><p class="no-tracks">No tracks found in library</p></div>';
        }
    } else {
        element.classList.remove('expanded');
    }
    lucide.createIcons();
}

function renderArtistAllTracks(index, container) {
    const artist = currentArtists[index];
    
    // Filter all tracks for this artist
    const artistTracks = fullTracks.filter(t => t.artist === artist.name);
    
    if (artistTracks.length === 0) {
        container.innerHTML = '<p style="text-align: center; color: #b8b8b8;">No tracks found for this artist</p>';
        return;
    }

    // Sort by album then track number if possible
    artistTracks.sort((a, b) => {
        if (a.album !== b.album) return a.album.localeCompare(b.album);
        return (a.track_number || 0) - (b.track_number || 0);
    });

    container.innerHTML = `
        <div class="artist-all-tracks-list">
            ${artistTracks.map((track, i) => `
                <div class="mini-track-item" onclick="event.stopPropagation(); playTrack('${track.id}')">
                    <div class="mini-track-num">${i + 1}</div>
                    <div class="mini-track-title">
                        ${escapeHtml(track.title)}
                        <span style="color: var(--text-light); font-size: 0.8em; margin-left: 8px;">• ${escapeHtml(track.album)}</span>
                    </div>
                    <div class="mini-track-duration">${formatDuration(track.duration_secs)}</div>
                    <div class="mini-track-actions">
                        <button class="btn-action btn-small" onclick="event.stopPropagation(); addToQueue('${track.id}')" title="Add to queue">
                            <i data-lucide="list-plus"></i>
                        </button>
                    </div>
                </div>
            `).join('')}
        </div>
    `;
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
                <div class="stat-icon"><i data-lucide="music"></i></div>
                <div class="stat-value">${stats.total_tracks}</div>
                <div class="stat-label">Tracks</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon"><i data-lucide="disc"></i></div>
                <div class="stat-value">${stats.total_albums}</div>
                <div class="stat-label">Albums</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon"><i data-lucide="mic-2"></i></div>
                <div class="stat-value">${stats.total_artists}</div>
                <div class="stat-label">Artists</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon"><i data-lucide="clock"></i></div>
                <div class="stat-value">${totalDuration}</div>
                <div class="stat-label">Total Duration</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon"><i data-lucide="hard-drive"></i></div>
                <div class="stat-value">${totalSize}</div>
                <div class="stat-label">Total Size</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon"><i data-lucide="bar-chart-3"></i></div>
                <div class="stat-value">${avgTrackSize}</div>
                <div class="stat-label">Avg Track Size</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon"><i data-lucide="play-circle"></i></div>
                <div class="stat-value">${stats.total_plays || 0}</div>
                <div class="stat-label">Total Plays</div>
            </div>
        </div>
    `;
    lucide.createIcons();
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
        closeLyricsModal();
    }
});

// ========== PLAYLIST MANAGEMENT ==========

// Load playlists from server
async function loadPlaylists() {
    try {
        const response = await fetch(`${API_BASE}/playlists`);
        if (response.ok) {
            playlists = await response.json();
            if (currentView === 'playlists') {
                displayPlaylists();
            }
        }
    } catch (e) {
        console.error('Failed to load playlists:', e);
    }
}

// Save playlists - now handled by server
function savePlaylists() {
    // No longer using localStorage for playlists
}

// Display all playlists
function displayPlaylists() {
    const playlistList = document.getElementById('playlist-list');

    if (playlists.length === 0) {
        playlistList.innerHTML = `
            <div class="empty-playlist" style="grid-column: 1/-1; text-align: center; padding: 60px; background: var(--card-bg); border-radius: 16px; border: 2px dashed var(--card-border);">
                <div class="empty-playlist-icon" style="font-size: 64px; color: var(--text-light); margin-bottom: 20px;"><i data-lucide="list-music"></i></div>
                <p style="color: var(--text-light); font-size: 18px;">No playlists yet. Create your first playlist!</p>
            </div>
        `;
        lucide.createIcons();
        return;
    }

    playlistList.innerHTML = playlists.map(pl => {
        const trackCount = pl.tracks.length;
        const playlistTracks = pl.tracks.map(id => fullTracks.find(t => t.id === id)).filter(Boolean);
        const totalDuration = playlistTracks.reduce((sum, t) => sum + (t.duration_secs || 0), 0);
        
        // Get up to 4 unique covers for the grid
        const covers = [];
        for (const track of playlistTracks) {
            if (track.has_cover && !covers.includes(track.id)) {
                covers.push(track.id);
                if (covers.length >= 4) break;
            }
        }

        return `
            <div class="playlist-card" onclick="togglePlaylist(this)">
                <div class="playlist-cover-wrapper">
                    ${covers.length > 0 ? `
                        <div class="playlist-cover-grid">
                            ${covers.map(id => `<img src="${API_BASE}/cover/${id}" class="playlist-cover-img" onerror="this.style.display='none';">`).join('')}
                            ${Array(Math.max(0, 4 - covers.length)).fill(0).map(() => `<div class="playlist-cover-placeholder" style="position: relative; font-size: 24px;"><i data-lucide="music"></i></div>`).join('')}
                        </div>
                    ` : `
                        <div class="playlist-cover-placeholder">
                            <i data-lucide="list-music"></i>
                        </div>
                    `}
                    <div class="playlist-card-overlay">
                        <button class="album-overlay-btn" onclick="event.stopPropagation(); playPlaylist('${pl.id}')" title="Play playlist" ${trackCount === 0 ? 'disabled' : ''}>
                            <i data-lucide="play"></i>
                        </button>
                        <button class="album-overlay-btn" onclick="event.stopPropagation(); deletePlaylist('${pl.id}')" title="Delete playlist">
                            <i data-lucide="trash-2"></i>
                        </button>
                    </div>
                </div>
                <div class="playlist-card-content">
                    <h3>${escapeHtml(pl.name)}</h3>
                    ${pl.description ? `<p class="playlist-description">${escapeHtml(pl.description)}</p>` : ''}
                    <div class="playlist-card-meta">
                        <span><i data-lucide="music" style="width: 12px; height: 12px; vertical-align: middle;"></i> ${trackCount} tracks</span>
                        <span><i data-lucide="clock" style="width: 12px; height: 12px; vertical-align: middle;"></i> ${formatDuration(totalDuration)}</span>
                    </div>

                    <div class="playlist-tracks-list">
                        <div style="min-height: 0;">
                            <div class="playlist-details-meta">
                                <span><i data-lucide="calendar"></i> Created: ${new Date(pl.createdAt || Date.now()).toLocaleDateString()}</span>
                                <span><i data-lucide="clock"></i> Total time: ${formatDuration(totalDuration)}</span>
                            </div>
                            <div class="tracks-table">
                                ${playlistTracks.map((track, i) => `
                                    <div class="playlist-track-row" onclick="event.stopPropagation(); playTrackFromPlaylist('${pl.id}', '${track.id}')">
                                        <div class="playlist-track-num">${i + 1}</div>
                                        <div class="playlist-track-info-cell">
                                            <div class="playlist-track-title-main">${escapeHtml(track.title || 'Unknown Title')}</div>
                                            <div class="playlist-track-artist-sub">${escapeHtml(track.artist || 'Unknown Artist')} • ${escapeHtml(track.album || 'Unknown Album')}</div>
                                        </div>
                                        <div class="playlist-track-duration">${formatDuration(track.duration_secs || 0)}</div>
                                        <div class="playlist-track-actions">
                                            <button class="btn-action btn-playlist" onclick="event.stopPropagation(); removeFromPlaylist('${pl.id}', '${track.id}')" title="Remove from playlist">
                                                <i data-lucide="x"></i>
                                            </button>
                                            <button class="btn-action btn-queue" onclick="event.stopPropagation(); addToQueue('${track.id}')" title="Add to queue">
                                                <i data-lucide="list-plus"></i>
                                            </button>
                                        </div>
                                    </div>
                                `).join('')}
                                ${trackCount === 0 ? '<p style="text-align: center; color: var(--text-light); padding: 20px;">No tracks in this playlist</p>' : ''}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }).join('');
    lucide.createIcons();
}

function togglePlaylist(element) {
    const isExpanded = element.classList.contains('expanded');

    // Close all other expanded playlists
    document.querySelectorAll('.playlist-card.expanded').forEach(card => {
        card.classList.remove('expanded');
    });

    if (!isExpanded) {
        element.classList.add('expanded');
        element.scrollIntoView({ behavior: 'smooth', block: 'center' });
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
async function handleCreatePlaylist(event) {
    event.preventDefault();

    const name = document.getElementById('playlistName').value.trim();
    const description = document.getElementById('playlistDescription').value.trim();

    if (!name) {
        alert('Please enter a playlist name');
        return;
    }

    try {
        const response = await fetch(`${API_BASE}/playlists`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                name: name,
                description: description
            }),
        });

        if (response.ok) {
            const newPlaylist = await response.json();
            playlists.push(newPlaylist);
            
            // Close modal and refresh display
            closeCreatePlaylistModal();
            displayPlaylists();
            console.log('Playlist created:', newPlaylist);
        } else {
            const error = await response.text();
            alert(`Failed to create playlist: ${error}`);
        }
    } catch (e) {
        console.error('Error creating playlist:', e);
        alert('Error creating playlist. See console for details.');
    }
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
                <span class="playlist-icon"><i data-lucide="list-music"></i></span>
                <span class="playlist-name">${escapeHtml(pl.name)}</span>
                <span class="playlist-track-count">${pl.tracks.length} tracks</span>
            </div>
        `).join('');
        lucide.createIcons();
    }

    modal.style.display = 'flex';
}

// Close add to playlist modal
function closeAddToPlaylistModal() {
    document.getElementById('addToPlaylistModal').style.display = 'none';
    trackToAdd = null;
}

// Add track to playlist
async function addTrackToPlaylist(playlistId) {
    if (!trackToAdd) return;

    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;

    // Check if track already in playlist
    if (pl.tracks.includes(trackToAdd)) {
        alert('Track is already in this playlist');
        return;
    }

    try {
        const response = await fetch(`${API_BASE}/playlists/${playlistId}/tracks/${trackToAdd}`, {
            method: 'POST',
        });

        if (response.ok) {
            // Update local state
            pl.tracks.push(trackToAdd);
            
            // Close modal and refresh if on playlists view
            closeAddToPlaylistModal();
            if (currentView === 'playlists') {
                displayPlaylists();
            }
            
            // Show success message
            const track = fullTracks.find(t => t.id === trackToAdd);
            console.log(`Added "${track?.title}" to playlist "${pl.name}"`);
        } else {
            const error = await response.text();
            alert(`Failed to add track to playlist: ${error}`);
        }
    } catch (e) {
        console.error('Error adding track to playlist:', e);
    }
}

// Remove track from playlist
async function removeFromPlaylist(playlistId, trackId) {
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;

    try {
        const response = await fetch(`${API_BASE}/playlists/${playlistId}/tracks/${trackId}`, {
            method: 'DELETE',
        });

        if (response.ok) {
            // Update local state
            pl.tracks = pl.tracks.filter(id => id !== trackId);
            displayPlaylists();
        } else {
            const error = await response.text();
            alert(`Failed to remove track: ${error}`);
        }
    } catch (e) {
        console.error('Error removing track from playlist:', e);
    }
}

// Delete playlist
async function deletePlaylist(playlistId) {
    const pl = playlists.find(p => p.id === playlistId);
    if (!pl) return;

    if (confirm(`Are you sure you want to delete the playlist "${pl.name}"?`)) {
        try {
            const response = await fetch(`${API_BASE}/playlists/${playlistId}`, {
                method: 'DELETE',
            });

            if (response.ok) {
                playlists = playlists.filter(p => p.id !== playlistId);
                displayPlaylists();
            } else {
                const error = await response.text();
                alert(`Failed to delete playlist: ${error}`);
            }
        } catch (e) {
            console.error('Error deleting playlist:', e);
        }
    }
}

// Open create playlist from add modal
function openCreatePlaylistFromAdd() {
    closeAddToPlaylistModal();
    openCreatePlaylistModal();
}

// Generate unique ID
// ========== PLAY QUEUE MANAGEMENT ==========

// Add track to queue
function addToQueue(trackId) {
    const track = fullTracks.find(t => t.id === trackId);
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
                <div class="queue-empty-icon"><i data-lucide="music"></i></div>
                <p>Queue is empty</p>
                <p class="queue-empty-hint">Add tracks to start playing</p>
            </div>
        `;
        queueCount.textContent = '0 tracks';
        queueDuration.textContent = '0:00';
        lucide.createIcons();
        return;
    }

    // Calculate total duration
    let totalDuration = 0;
    playQueue.forEach(trackId => {
        const track = fullTracks.find(t => t.id === trackId);
        if (track && track.duration_secs) {
            totalDuration += track.duration_secs;
        }
    });

    // Update info
    queueCount.textContent = `${playQueue.length} track${playQueue.length !== 1 ? 's' : ''}`;
    queueDuration.textContent = formatDuration(totalDuration);

    // Render queue items
    queueList.innerHTML = playQueue.map((trackId, index) => {
        const track = fullTracks.find(t => t.id === trackId);
        if (!track) return '';

        const isCurrentTrack = index === queueIndex;
        const coverUrl = track.has_cover ? `${API_BASE}/cover/${trackId}` : null;

        return `
            <div class="queue-item ${isCurrentTrack ? 'playing' : ''}" onclick="playFromQueue(${index})">
                <div class="queue-item-number">${index + 1}</div>
                <div class="queue-item-cover">
                    ${coverUrl
                ? `<img src="${coverUrl}" alt="Cover" onerror="this.style.display='none';">`
                : '<i data-lucide="disc"></i>'}
                </div>
                <div class="queue-item-info">
                    <div class="queue-item-title">${escapeHtml(track.title || 'Unknown Title')}</div>
                    <div class="queue-item-artist">${escapeHtml(track.artist || 'Unknown Artist')}</div>
                </div>
                <div class="queue-item-duration">${track.duration_secs ? formatDuration(track.duration_secs) : '--:--'}</div>
                <button class="queue-item-remove" onclick="event.stopPropagation(); removeFromQueue(${index})" title="Remove from queue">
                    <i data-lucide="x"></i>
                </button>
            </div>
        `;
    }).join('');
    lucide.createIcons();
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

// ========== LYRICS MANAGEMENT ==========

let currentLyricsTrackId = null;
let currentLyrics = null;
let isLyricsPanelVisible = false;

// Open lyrics modal
async function openLyricsModal(trackId) {
    const track = fullTracks.find(t => t.id === trackId);
    if (!track) return;

    currentLyricsTrackId = trackId;

    // Update track info
    document.getElementById('lyricsTrackTitle').textContent = track.title || 'Unknown Title';
    document.getElementById('lyricsTrackArtist').textContent = track.artist || 'Unknown Artist';

    // Load lyrics if available
    if (track.has_lyrics) {
        await loadLyricsForModal(trackId);
    } else {
        currentLyrics = null;
        clearLyricsModal();
    }

    // Show modal
    document.getElementById('lyricsModal').style.display = 'flex';
    switchLyricsTab('view');
}

// Close lyrics modal
function closeLyricsModal() {
    document.getElementById('lyricsModal').style.display = 'none';
    currentLyricsTrackId = null;
}

// Switch between view and edit tabs
function switchLyricsTab(tab) {
    // Update tab buttons
    document.querySelectorAll('.lyrics-tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tab);
    });

    // Update tab content
    document.querySelectorAll('.lyrics-tab-content').forEach(content => {
        content.classList.remove('active');
    });

    if (tab === 'view') {
        document.getElementById('lyricsViewTab').classList.add('active');
    } else {
        document.getElementById('lyricsEditTab').classList.add('active');
        // Populate edit form if lyrics exist
        if (currentLyrics) {
            document.getElementById('lyricsContent').value = currentLyrics.content;
            document.getElementById('lyricsFormat').value = currentLyrics.format || '';
            document.getElementById('lyricsLanguage').value = currentLyrics.language || '';
            document.getElementById('lyricsSource').value = currentLyrics.source || '';
            document.getElementById('deleteLyricsBtn').style.display = 'inline-block';
        } else {
            document.getElementById('lyricsContent').value = '';
            document.getElementById('lyricsFormat').value = '';
            document.getElementById('lyricsLanguage').value = '';
            document.getElementById('lyricsSource').value = '';
            document.getElementById('deleteLyricsBtn').style.display = 'none';
        }
    }
}

// Load lyrics for modal
async function loadLyricsForModal(trackId) {
    try {
        const response = await fetch(`${API_BASE}/lyrics/${trackId}`);

        if (response.ok) {
            currentLyrics = await response.json();
            displayLyricsInModal(currentLyrics);
        } else if (response.status === 404) {
            currentLyrics = null;
            clearLyricsModal();
        } else {
            throw new Error('Failed to load lyrics');
        }
    } catch (error) {
        console.error('Error loading lyrics:', error);
        clearLyricsModal();
    }
}

// Display lyrics in view tab
function displayLyricsInModal(lyrics) {
    const display = document.getElementById('lyricsDisplay');

    if (lyrics.format === 'lrc' || lyrics.format === 'lrc_word') {
        // Parse lyrics based on format
        const lines = lyrics.format === 'lrc_word'
            ? parseWordLevelLrcLyrics(lyrics.content)
            : parseLrcLyrics(lyrics.content);

        // Check if any line has word-level timing
        const hasWordTiming = lines.some(l => l.hasWordTiming);

        display.innerHTML = `
            ${hasWordTiming ? '<div class="lyrics-format-badge"><i data-lucide="mic"></i> Word-level synchronized lyrics (Karaoke mode)</div>' : ''}
            <div class="lyrics-content lrc-lyrics">
                ${lines.map((line, lineIndex) => {
            if (line.words && line.words.length > 0) {
                // Word-level lyrics rendering
                const wordsHtml = line.words.map((word, wordIndex) =>
                    `<span class="lyrics-word" data-time="${word.time}" data-duration="${word.duration}" data-line="${lineIndex}" data-word="${wordIndex}">${escapeHtml(word.word)}</span>`
                ).join('');

                return `
                            <div class="lyrics-line word-level-line" data-time="${line.time}" data-line="${lineIndex}">
                                ${line.timestamp ? `<span class="lyrics-timestamp">${line.timestamp}</span>` : ''}
                                <span class="lyrics-text word-level-text">${wordsHtml}</span>
                            </div>
                        `;
            } else {
                // Regular line-level lyrics
                return `
                            <div class="lyrics-line" data-time="${line.time}">
                                ${line.timestamp ? `<span class="lyrics-timestamp">${line.timestamp}</span>` : ''}
                                <span class="lyrics-text">${escapeHtml(line.text)}</span>
                            </div>
                        `;
            }
        }).join('')}
            </div>
        `;
    } else {
        // Display plain text
        const lines = lyrics.content.split('\n');
        display.innerHTML = `
            <div class="lyrics-content plain-lyrics">
                ${lines.map(line => `
                    <div class="lyrics-line">
                        <span class="lyrics-text">${escapeHtml(line)}</span>
                    </div>
                `).join('')}
            </div>
        `;
    }

    // Show metadata
    if (lyrics.language || lyrics.source) {
        const metadata = [];
        if (lyrics.language) metadata.push(`Language: ${lyrics.language}`);
        if (lyrics.source) metadata.push(`Source: ${lyrics.source}`);

        display.innerHTML += `
            <div class="lyrics-metadata">
                ${metadata.join(' • ')}
            </div>
        `;
    }
    lucide.createIcons();
}

// Clear lyrics modal
function clearLyricsModal() {
    document.getElementById('lyricsDisplay').innerHTML = `
        <div class="lyrics-empty">
            <div class="lyrics-empty-icon"><i data-lucide="file-text"></i></div>
            <p>No lyrics available for this track</p>
            <button class="btn btn-primary" onclick="switchLyricsTab('edit')"><i data-lucide="plus"></i> Add Lyrics</button>
        </div>
    `;
    lucide.createIcons();
}

// Parse LRC lyrics (line-level only)
function parseLrcLyrics(content) {
    const lines = [];
    // Support both standard LRC [mm:ss.xx] and extended format [offset,duration]
    const lrcRegex = /\[(\d+):(\d{2})\.(\d{2,3})\](.*)/;
    const extendedLrcRegex = /\[(\d+),(\d+)\](.*)/;

    content.split('\n').forEach(line => {
        // Try standard LRC format first
        let match = line.match(lrcRegex);
        if (match) {
            const minutes = parseInt(match[1]);
            const seconds = parseInt(match[2]);
            const centiseconds = parseInt(match[3]);
            const time = minutes * 60 + seconds + centiseconds / (match[3].length === 3 ? 1000 : 100);
            const timestamp = `${match[1]}:${match[2]}.${match[3]}`;
            let text = match[4].trim();

            // Check if this line has word-level timing (extended format)
            // Format: word(offset,duration)word(offset,duration)
            const hasWordTiming = /\((\d+),(\d+)\)/.test(text);

            if (hasWordTiming) {
                // Parse word-level timing and extract clean text
                text = parseWordLevelLyrics(text);
            }

            lines.push({ time, timestamp, text, hasWordTiming });
        } else {
            // Try extended format [offset,duration]
            match = line.match(extendedLrcRegex);
            if (match) {
                const offset = parseInt(match[1]);
                const duration = parseInt(match[2]);
                const time = offset / 1000; // Convert milliseconds to seconds
                const timestamp = formatTimestamp(time);
                let text = match[3].trim();

                // Check for word-level timing
                const hasWordTiming = /\((\d+),(\d+)\)/.test(text);
                if (hasWordTiming) {
                    text = parseWordLevelLyrics(text);
                }

                lines.push({ time, timestamp, text, hasWordTiming });
            } else if (line.trim() && !line.startsWith('[ti:') && !line.startsWith('[ar:') &&
                !line.startsWith('[al:') && !line.startsWith('[by:') && !line.startsWith('[offset:')) {
                // Non-LRC line (skip metadata tags)
                lines.push({ time: -1, timestamp: null, text: line.trim(), hasWordTiming: false });
            }
        }
    });

    return lines;
}

// Parse word-level LRC lyrics (preserves word timing data)
function parseWordLevelLrcLyrics(content) {
    const lines = [];
    // Support both standard LRC [mm:ss.xx] and extended format [offset,duration]
    const lrcRegex = /\[(\d+):(\d{2})\.(\d{2,3})\](.*)/;
    const extendedLrcRegex = /\[(\d+),(\d+)\](.*)/;

    content.split('\n').forEach(line => {
        // Try standard LRC format first
        let match = line.match(lrcRegex);
        if (match) {
            const minutes = parseInt(match[1]);
            const seconds = parseInt(match[2]);
            const centiseconds = parseInt(match[3]);
            const time = minutes * 60 + seconds + centiseconds / (match[3].length === 3 ? 1000 : 100);
            const timestamp = `${match[1]}:${match[2]}.${match[3]}`;
            const text = match[4].trim();

            // Parse word-level timing
            const words = parseWordsWithTiming(text, time);

            lines.push({ time, timestamp, text, words, hasWordTiming: words.length > 0 });
        } else {
            // Try extended format [offset,duration]
            match = line.match(extendedLrcRegex);
            if (match) {
                const offset = parseInt(match[1]);
                const duration = parseInt(match[2]);
                const time = offset / 1000; // Convert milliseconds to seconds
                const timestamp = formatTimestamp(time);
                const text = match[3].trim();

                // Parse word-level timing
                const words = parseWordsWithTiming(text, time);

                lines.push({ time, timestamp, text, words, hasWordTiming: words.length > 0 });
            } else if (line.trim() && !line.startsWith('[ti:') && !line.startsWith('[ar:') &&
                !line.startsWith('[al:') && !line.startsWith('[by:') && !line.startsWith('[offset:')) {
                // Non-LRC line (skip metadata tags)
                lines.push({ time: -1, timestamp: null, text: line.trim(), words: [], hasWordTiming: false });
            }
        }
    });

    return lines;
}

// Parse words with timing from text: word(offset,duration) -> {word, time}
function parseWordsWithTiming(text, lineTime) {
    const words = [];
    // Match word(offset,duration) pattern
    const wordRegex = /(.+?)\((\d+),(\d+)\)/g;
    let match;

    while ((match = wordRegex.exec(text)) !== null) {
        const word = match[1];
        const time = parseInt(match[2]) / 1000; // Convert to seconds
        const duration = parseInt(match[3]) / 1000; // Convert to seconds
        words.push({ word, time, duration });
    }

    return words;
}

// Parse word-level lyrics and extract clean text
function parseWordLevelLyrics(text) {
    // Remove word-level timing information: word(offset,duration) -> word
    return text.replace(/(\S+?)\((\d+),(\d+)\)/g, '$1')
        .replace(/\s+/g, ' ')
        .trim();
}

// Format timestamp from seconds
function formatTimestamp(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const cs = Math.floor((seconds % 1) * 100);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${cs.toString().padStart(2, '0')}`;
}

// Save lyrics
async function saveLyrics() {
    if (!currentLyricsTrackId) return;

    const content = document.getElementById('lyricsContent').value.trim();

    if (!content) {
        alert('Please enter lyrics content');
        return;
    }

    const format = document.getElementById('lyricsFormat').value || null;
    const language = document.getElementById('lyricsLanguage').value.trim() || null;
    const source = document.getElementById('lyricsSource').value.trim() || null;

    try {
        const response = await fetch(`${API_BASE}/lyrics/${currentLyricsTrackId}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                content,
                format,
                language,
                source
            })
        });

        if (!response.ok) {
            throw new Error('Failed to save lyrics');
        }

        currentLyrics = await response.json();

        // Update track's has_lyrics flag in both arrays
        const fullTrack = fullTracks.find(t => t.id === currentLyricsTrackId);
        if (fullTrack) {
            fullTrack.has_lyrics = true;
        }
        const track = tracks.find(t => t.id === currentLyricsTrackId);
        if (track) {
            track.has_lyrics = true;
        }

        // Refresh track list
        renderTracks();

        // Switch to view tab
        switchLyricsTab('view');

        console.log('Lyrics saved successfully');
    } catch (error) {
        console.error('Error saving lyrics:', error);
        alert('Failed to save lyrics. Please try again.');
    }
}

// Delete lyrics
async function deleteLyrics() {
    if (!currentLyricsTrackId) return;

    if (!confirm('Are you sure you want to delete the lyrics for this track?')) {
        return;
    }

    try {
        const response = await fetch(`${API_BASE}/lyrics/${currentLyricsTrackId}`, {
            method: 'DELETE'
        });

        if (response.status !== 204) {
            throw new Error('Failed to delete lyrics');
        }

        currentLyrics = null;

        // Update track's has_lyrics flag in both arrays
        const fullTrack = fullTracks.find(t => t.id === currentLyricsTrackId);
        if (fullTrack) {
            fullTrack.has_lyrics = false;
        }
        const track = tracks.find(t => t.id === currentLyricsTrackId);
        if (track) {
            track.has_lyrics = false;
        }

        // Refresh track list
        renderTracks();

        // Clear modal
        clearLyricsModal();
        switchLyricsTab('view');

        console.log('Lyrics deleted successfully');
    } catch (error) {
        console.error('Error deleting lyrics:', error);
        alert('Failed to delete lyrics. Please try again.');
    }
}

// ========== PLAYER INTEGRATED LYRICS ==========

let hasLyricsAvailable = false;
let lyricsVisible = true; // Default to showing lyrics when available

// Load lyrics for currently playing track
async function loadLyricsForPlayer(trackId) {
    try {
        const response = await fetch(`${API_BASE}/lyrics/${trackId}`);

        if (response.ok) {
            const lyrics = await response.json();
            displayLyricsInPlayer(lyrics);
            hasLyricsAvailable = true;

            // Show toggle button
            const toggleBtn = document.getElementById('lyricsToggleBtn');
            toggleBtn.style.display = 'flex';

            // Show lyrics if visibility preference is true
            if (lyricsVisible) {
                showPlayerLyrics();
                toggleBtn.classList.add('active');
            } else {
                hidePlayerLyrics();
                toggleBtn.classList.remove('active');
            }
        } else {
            hasLyricsAvailable = false;
            hidePlayerLyrics();
            document.getElementById('lyricsToggleBtn').style.display = 'none';
        }
    } catch (error) {
        console.error('Error loading lyrics for player:', error);
        hasLyricsAvailable = false;
        hidePlayerLyrics();
        document.getElementById('lyricsToggleBtn').style.display = 'none';
    }
}

// Display lyrics in player
function displayLyricsInPlayer(lyrics) {
    const content = document.getElementById('playerLyricsContent');

    if (lyrics.format === 'lrc' || lyrics.format === 'lrc_word') {
        // Parse lyrics based on format
        const lines = lyrics.format === 'lrc_word'
            ? parseWordLevelLrcLyrics(lyrics.content)
            : parseLrcLyrics(lyrics.content);

        content.innerHTML = `
            <div class="lyrics-content lrc-lyrics">
                ${lines.map((line, lineIndex) => {
            if (line.words && line.words.length > 0) {
                // Word-level lyrics rendering with karaoke effect
                const wordsHtml = line.words.map((word, wordIndex) =>
                    `<span class="lyrics-word" data-time="${word.time}" data-duration="${word.duration}" data-index="${lineIndex}" data-line="${lineIndex}" data-word="${wordIndex}">${escapeHtml(word.word)}</span>`
                ).join('');

                return `
                            <div class="lyrics-line word-level-line" data-time="${line.time}" data-index="${lineIndex}" data-line="${lineIndex}">
                                ${line.timestamp ? `<span class="lyrics-timestamp">${line.timestamp}</span>` : ''}
                                <span class="lyrics-text word-level-text">${wordsHtml}</span>
                            </div>
                        `;
            } else {
                // Regular line-level lyrics
                return `
                            <div class="lyrics-line" data-time="${line.time}" data-index="${lineIndex}" data-line="${lineIndex}">
                                ${line.timestamp ? `<span class="lyrics-timestamp">${line.timestamp}</span>` : ''}
                                <span class="lyrics-text">${escapeHtml(line.text)}</span>
                            </div>
                        `;
            }
        }).join('')}
            </div>
        `;

        // Enable synchronized scrolling with word-level support
        enableLyricsSynchronization(lines);
    } else {
        // Display plain text
        const lines = lyrics.content.split('\n');
        content.innerHTML = `
            <div class="lyrics-content plain-lyrics">
                ${lines.map(line => `
                    <div class="lyrics-line">
                        <span class="lyrics-text">${escapeHtml(line)}</span>
                    </div>
                `).join('')}
            </div>
        `;
    }
}

// Show player lyrics container
function showPlayerLyrics() {
    const panel = document.getElementById('playerLyricsContainer');
    panel.style.display = 'flex';
    document.body.style.paddingLeft = panel.style.width || '350px';
}

// Hide player lyrics container
function hidePlayerLyrics() {
    const panel = document.getElementById('playerLyricsContainer');
    panel.style.display = 'none';
    document.body.style.paddingLeft = '0';
    lyricsLines = [];
    currentLyricsIndex = -1;
}

// Toggle player lyrics visibility
function togglePlayerLyrics() {
    if (!hasLyricsAvailable) return;

    lyricsVisible = !lyricsVisible;
    const toggleBtn = document.getElementById('lyricsToggleBtn');

    if (lyricsVisible) {
        showPlayerLyrics();
        toggleBtn.classList.add('active');
    } else {
        hidePlayerLyrics();
        toggleBtn.classList.remove('active');
    }
}

// Enable lyrics synchronization with playback
let lyricsLines = [];
let currentLyricsIndex = -1;

function enableLyricsSynchronization(lines) {
    lyricsLines = lines.filter(l => l.time >= 0).sort((a, b) => a.time - b.time);
    currentLyricsIndex = -1;
}

// Update synchronized lyrics (call this from audio timeupdate)
function updateSynchronizedLyrics(currentTime) {
    if (lyricsLines.length === 0) return;

    // Find the current line
    let newIndex = -1;
    for (let i = lyricsLines.length - 1; i >= 0; i--) {
        if (currentTime >= lyricsLines[i].time) {
            newIndex = i;
            break;
        }
    }

    if (newIndex !== currentLyricsIndex) {
        // Remove previous highlight
        const prevLine = document.querySelector('.lyrics-line.active');
        if (prevLine) {
            prevLine.classList.remove('active');
        }

        // Highlight current line
        if (newIndex >= 0) {
            const currentLine = document.querySelector(`[data-index="${newIndex}"]`);
            if (currentLine) {
                currentLine.classList.add('active');
                // Scroll to center
                currentLine.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }

        currentLyricsIndex = newIndex;
    }

    // Handle word-level highlighting if available
    if (newIndex >= 0 && lyricsLines[newIndex].words && lyricsLines[newIndex].words.length > 0) {
        const currentLineWords = lyricsLines[newIndex].words;

        // Find and highlight current word
        for (let i = 0; i < currentLineWords.length; i++) {
            const word = currentLineWords[i];
            const wordElement = document.querySelector(`[data-line="${newIndex}"][data-word="${i}"]`);

            if (!wordElement) continue;

            if (currentTime >= word.time && currentTime < word.time + word.duration) {
                // Current word - highlight it
                if (!wordElement.classList.contains('active')) {
                    wordElement.classList.add('active');
                }
            } else if (currentTime >= word.time + word.duration) {
                // Past word - mark as sung
                if (!wordElement.classList.contains('sung')) {
                    wordElement.classList.remove('active');
                    wordElement.classList.add('sung');
                }
            } else {
                // Future word - no highlight
                wordElement.classList.remove('active', 'sung');
            }
        }

        // Clear highlighting from previous lines
        if (currentLyricsIndex > 0) {
            const prevWords = document.querySelectorAll(`[data-line="${currentLyricsIndex - 1}"] .lyrics-word`);
            prevWords.forEach(w => {
                w.classList.remove('active');
                w.classList.add('sung');
            });
        }
    }
}

// ========== LYRICS SEARCH ==========

let currentSearchTrackId = null;

// Open lyrics search modal
function openLyricsSearchModal() {
    if (!currentLyricsTrackId) return;

    // Store the track ID for search
    currentSearchTrackId = currentLyricsTrackId;

    // Get track info to pre-fill search
    const track = fullTracks.find(t => t.id === currentLyricsTrackId);
    if (track) {
        document.getElementById('lyricsSearchQuery').value = track.title || '';
        document.getElementById('lyricsSearchArtist').value = track.artist || '';
    }

    // Clear previous results
    document.getElementById('lyricsSearchResults').style.display = 'none';
    document.getElementById('lyricsSearchError').style.display = 'none';
    document.getElementById('lyricsSearchResultsList').innerHTML = '';

    // Show modal
    document.getElementById('lyricsSearchModal').style.display = 'flex';
}

// Close lyrics search modal
function closeLyricsSearchModal() {
    document.getElementById('lyricsSearchModal').style.display = 'none';
    currentSearchTrackId = null;
}

// Perform lyrics search
async function performLyricsSearch() {
    const query = document.getElementById('lyricsSearchQuery').value.trim();
    const artist = document.getElementById('lyricsSearchArtist').value.trim();
    const provider = document.getElementById('lyricsSearchProvider').value;

    if (!query) {
        alert('Please enter a song title');
        return;
    }

    // Show loading
    document.getElementById('lyricsSearchLoading').style.display = 'block';
    document.getElementById('lyricsSearchError').style.display = 'none';
    document.getElementById('lyricsSearchResults').style.display = 'none';

    try {
        // Build query params
        let url = `${API_BASE}/lyrics/search?q=${encodeURIComponent(query)}&provider=${provider}`;
        if (artist) {
            url += `&artist=${encodeURIComponent(artist)}`;
        }

        const response = await fetch(url);

        if (!response.ok) {
            throw new Error('Search failed');
        }

        const results = await response.json();

        // Hide loading
        document.getElementById('lyricsSearchLoading').style.display = 'none';

        if (results.length === 0) {
            document.getElementById('lyricsSearchError').textContent = 'No results found. Try different search terms.';
            document.getElementById('lyricsSearchError').style.display = 'block';
            return;
        }

        // Display results
        displayLyricsSearchResults(results, provider);

    } catch (error) {
        console.error('Error searching lyrics:', error);
        document.getElementById('lyricsSearchLoading').style.display = 'none';
        document.getElementById('lyricsSearchError').textContent = 'Failed to search lyrics. Please try again.';
        document.getElementById('lyricsSearchError').style.display = 'block';
    }
}

// Display lyrics search results
function displayLyricsSearchResults(results, provider) {
    const resultsList = document.getElementById('lyricsSearchResultsList');
    resultsList.innerHTML = '';

    results.forEach(result => {
        const item = document.createElement('div');
        item.className = 'lyrics-search-result-item';
        item.onclick = () => fetchAndApplyLyrics(result.id, provider);

        // Format duration
        const duration = result.duration ? formatDuration(Math.floor(result.duration.secs)) : 'Unknown';

        item.innerHTML = `
            <div class="lyrics-search-result-title">${escapeHtml(result.title)}</div>
            <div class="lyrics-search-result-artist"><i data-lucide="user"></i> ${escapeHtml(result.artist)}</div>
            <div class="lyrics-search-result-meta">
                ${result.album ? `<div class="lyrics-search-result-album"><i data-lucide="disc"></i> ${escapeHtml(result.album)}</div>` : ''}
                <div class="lyrics-search-result-duration"><i data-lucide="clock"></i> ${duration}</div>
                <span class="lyrics-search-result-provider">${provider}</span>
            </div>
        `;

        resultsList.appendChild(item);
    });

    document.getElementById('lyricsSearchResults').style.display = 'block';
    lucide.createIcons();
}

// Fetch and apply lyrics from selected result
async function fetchAndApplyLyrics(songId, provider) {
    if (!currentSearchTrackId) return;

    // Show loading in search modal
    document.getElementById('lyricsSearchLoading').style.display = 'block';
    document.getElementById('lyricsSearchError').style.display = 'none';

    try {
        const response = await fetch(`${API_BASE}/lyrics/fetch/${provider}/${encodeURIComponent(songId)}`);

        if (!response.ok) {
            throw new Error('Failed to fetch lyrics');
        }

        const lyricsData = await response.json();

        // Apply lyrics to the edit form
        document.getElementById('lyricsContent').value = lyricsData.content;
        document.getElementById('lyricsFormat').value = lyricsData.format || '';
        document.getElementById('lyricsLanguage').value = lyricsData.language || '';
        document.getElementById('lyricsSource').value = lyricsData.source || '';

        // Close search modal
        closeLyricsSearchModal();

        // Show success message
        console.log('Lyrics fetched successfully from', provider);

        // Auto-save the lyrics
        await saveLyrics();

    } catch (error) {
        console.error('Error fetching lyrics:', error);
        document.getElementById('lyricsSearchLoading').style.display = 'none';
        document.getElementById('lyricsSearchError').textContent = 'Failed to fetch lyrics. Please try again.';
        document.getElementById('lyricsSearchError').style.display = 'block';
    }
}

// Format duration helper
function formatDuration(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// ========== THEME MANAGEMENT ==========

// Initialize theme on page load
function initTheme() {
    // Check localStorage for saved theme preference
    const savedTheme = localStorage.getItem('music_station_theme');

    // Default to light mode if no preference saved
    if (savedTheme === 'dark') {
        document.body.classList.add('dark-mode');
        updateThemeIcon(true);
    } else {
        // Explicitly set to light mode (default)
        document.body.classList.remove('dark-mode');
        updateThemeIcon(false);
    }
}

// Toggle between light and dark mode
function toggleTheme() {
    const isDarkMode = document.body.classList.toggle('dark-mode');

    // Save preference to localStorage
    localStorage.setItem('music_station_theme', isDarkMode ? 'dark' : 'light');

    // Update icon
    updateThemeIcon(isDarkMode);
}

// Update the theme toggle icon
function updateThemeIcon(isDarkMode) {
    const themeIcon = document.querySelector('.theme-icon');
    if (themeIcon) {
        themeIcon.innerHTML = isDarkMode ? '<i data-lucide="sun"></i>' : '<i data-lucide="moon"></i>';
        lucide.createIcons();
    }
}

// ========== AUTO-FETCH LYRICS ==========

let autoFetchState = {
    isRunning: false,
    isCancelled: false,
    total: 0,
    processed: 0,
    succeeded: 0,
    failed: 0,
    skipped: 0
};

// Start auto-fetch lyrics process
async function startAutoFetchLyrics() {
    // Get tracks without lyrics
    const tracksWithoutLyrics = fullTracks.filter(track => !track.has_lyrics && track.title);

    if (tracksWithoutLyrics.length === 0) {
        alert('All tracks already have lyrics!');
        return;
    }

    // Confirm with user
    const confirmed = confirm(
        `Found ${tracksWithoutLyrics.length} tracks without lyrics.\n\n` +
        `This will search for lyrics using QQ Music API and automatically apply them.\n\n` +
        `Continue?`
    );

    if (!confirmed) return;

    // Reset state
    autoFetchState = {
        isRunning: true,
        isCancelled: false,
        total: tracksWithoutLyrics.length,
        processed: 0,
        succeeded: 0,
        failed: 0,
        skipped: 0
    };

    // Show modal
    openAutoFetchModal();
    updateAutoFetchUI();

    // Process tracks
    for (let i = 0; i < tracksWithoutLyrics.length; i++) {
        if (autoFetchState.isCancelled) {
            addAutoFetchLog('Process cancelled by user', 'info');
            break;
        }

        const track = tracksWithoutLyrics[i];
        await processTrackForAutoFetch(track);

        // Sleep 1 second between requests (except for last track)
        if (i < tracksWithoutLyrics.length - 1 && !autoFetchState.isCancelled) {
            await sleep(1000);
        }
    }

    // Finalize
    autoFetchState.isRunning = false;
    document.getElementById('autoFetchCancelBtn').style.display = 'none';
    document.getElementById('autoFetchCloseBtn').style.display = 'inline-block';

    if (!autoFetchState.isCancelled) {
        addAutoFetchLog('Auto-fetch completed!', 'success');
    }

    // Refresh tracks to update has_lyrics flags
    await loadTracks(false);
}

// Process single track for auto-fetch
async function processTrackForAutoFetch(track) {
    autoFetchState.processed++;
    updateAutoFetchUI();

    const trackTitle = track.title || 'Unknown';
    const trackArtist = track.artist || '';

    // Update current track display
    document.getElementById('autoFetchCurrentTrack').innerHTML =
        `<strong>Processing:</strong> ${escapeHtml(trackTitle)} ${trackArtist ? `- ${escapeHtml(trackArtist)}` : ''}`;

    try {
        // Search for lyrics
        let url = `${API_BASE}/lyrics/search?q=${encodeURIComponent(trackTitle)}&provider=qqmusic`;
        if (trackArtist) {
            url += `&artist=${encodeURIComponent(trackArtist)}`;
        }

        const searchResponse = await fetch(url);
        if (!searchResponse.ok) {
            throw new Error('Search failed');
        }

        const results = await searchResponse.json();

        if (results.length === 0) {
            autoFetchState.skipped++;
            addAutoFetchLog(`No results: ${trackTitle}`, 'skip');
            return;
        }

        // Find best match based on duration
        let bestMatch = null;
        let minDurationDiff = Infinity;

        for (const result of results) {
            if (!result.duration || !result.duration.secs || !track.duration_secs) {
                continue;
            }

            const durationDiff = Math.abs(result.duration.secs - track.duration_secs);

            // Check if duration difference is within 10 seconds
            if (durationDiff < 10 && durationDiff < minDurationDiff) {
                bestMatch = result;
                minDurationDiff = durationDiff;
            }
        }

        if (!bestMatch) {
            autoFetchState.skipped++;
            addAutoFetchLog(`No match (duration): ${trackTitle}`, 'skip');
            return;
        }

        // Fetch lyrics
        const fetchResponse = await fetch(
            `${API_BASE}/lyrics/fetch/qqmusic/${encodeURIComponent(bestMatch.id)}`
        );

        if (!fetchResponse.ok) {
            throw new Error('Failed to fetch lyrics');
        }

        const lyricsData = await fetchResponse.json();

        // Upload lyrics to track
        const uploadResponse = await fetch(`${API_BASE}/lyrics/${track.id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                content: lyricsData.content,
                format: lyricsData.format,
                language: lyricsData.language,
                source: lyricsData.source
            })
        });

        if (!uploadResponse.ok) {
            throw new Error('Failed to upload lyrics');
        }

        // Success
        autoFetchState.succeeded++;
        track.has_lyrics = true; // Update local state
        addAutoFetchLog(`Success: ${trackTitle}`, 'success');

    } catch (error) {
        autoFetchState.failed++;
        addAutoFetchLog(`Error: ${trackTitle} - ${error.message}`, 'error');
        console.error('Auto-fetch error:', error);
    }
}

// Update auto-fetch UI
function updateAutoFetchUI() {
    const { total, processed, succeeded, failed, skipped } = autoFetchState;

    // Update progress text
    document.getElementById('autoFetchProgress').textContent =
        `${processed} / ${total} tracks processed`;

    // Update progress bar
    const percentage = total > 0 ? (processed / total) * 100 : 0;
    const progressBar = document.getElementById('autoFetchProgressBar');
    progressBar.style.width = `${percentage}%`;
    progressBar.textContent = `${Math.round(percentage)}%`;

    // Update details
    document.getElementById('autoFetchTotal').textContent = total;
    document.getElementById('autoFetchProcessed').textContent = processed;
    document.getElementById('autoFetchSucceeded').textContent = succeeded;
    document.getElementById('autoFetchFailed').textContent = failed;
    document.getElementById('autoFetchSkipped').textContent = skipped;
}

// Add log entry
function addAutoFetchLog(message, type = 'info') {
    const log = document.getElementById('autoFetchLog');
    const entry = document.createElement('div');
    entry.className = `log-entry log-${type}`;

    let icon = 'info';
    if (type === 'success') icon = 'check-circle';
    if (type === 'error') icon = 'alert-circle';
    if (type === 'skip') icon = 'skip-forward';

    entry.innerHTML = `
        <span class="log-time">[${new Date().toLocaleTimeString()}]</span>
        <span class="log-icon"><i data-lucide="${icon}"></i></span>
        <span class="log-message">${message}</span>
    `;
    log.appendChild(entry);

    // Auto-scroll to bottom
    log.scrollTop = log.scrollHeight;
    lucide.createIcons();
}

// Open auto-fetch modal
function openAutoFetchModal() {
    document.getElementById('autoFetchModal').style.display = 'flex';
    document.getElementById('autoFetchCancelBtn').style.display = 'inline-block';
    document.getElementById('autoFetchCloseBtn').style.display = 'none';
    document.getElementById('autoFetchLog').innerHTML = '';
    document.getElementById('autoFetchCurrentTrack').innerHTML = '';
}

// Close auto-fetch modal
function closeAutoFetchModal() {
    if (autoFetchState.isRunning) {
        const confirmed = confirm('Auto-fetch is still running. Are you sure you want to close?');
        if (!confirmed) return;
        autoFetchState.isCancelled = true;
    }
    document.getElementById('autoFetchModal').style.display = 'none';
}

// Cancel auto-fetch
function cancelAutoFetch() {
    if (!confirm('Are you sure you want to cancel auto-fetch?')) return;
    autoFetchState.isCancelled = true;
    addAutoFetchLog('Cancelling...', 'info');
}

// Sleep helper
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

