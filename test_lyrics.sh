#!/bin/bash
# Example script to test lyrics functionality
# Usage: ./test_lyrics.sh <track_id>

TRACK_ID=${1:-"your-track-id"}
SERVER="http://localhost:3000"

echo "========================================"
echo "Music Station - Lyrics Feature Test"
echo "========================================"
echo ""

# Test 1: Upload plain text lyrics
echo "1. Uploading plain text lyrics..."
curl -X PUT "${SERVER}/lyrics/${TRACK_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "First line of the song\nSecond line of the song\nThird line of the song",
    "language": "en",
    "source": "Manual upload"
  }' | jq '.'
echo ""

sleep 1

# Test 2: Retrieve lyrics
echo "2. Retrieving lyrics..."
curl -s "${SERVER}/lyrics/${TRACK_ID}" | jq '.'
echo ""

sleep 1

# Test 3: Update with LRC format
echo "3. Updating with LRC format lyrics..."
curl -X PUT "${SERVER}/lyrics/${TRACK_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "[00:12.00]First line of the song\n[00:16.50]Second line of the song\n[00:21.00]Third line of the song",
    "format": "lrc",
    "language": "en",
    "source": "LRC file"
  }' | jq '.'
echo ""

sleep 1

# Test 4: Retrieve updated lyrics
echo "4. Retrieving updated lyrics..."
curl -s "${SERVER}/lyrics/${TRACK_ID}" | jq '.'
echo ""

sleep 1

# Test 5: Check track info (should show has_lyrics: true)
echo "5. Checking track info (has_lyrics should be true)..."
curl -s "${SERVER}/tracks/${TRACK_ID}" | jq '{id, title, artist, has_lyrics}'
echo ""

sleep 1

# Test 6: Delete lyrics
echo "6. Deleting lyrics..."
curl -X DELETE "${SERVER}/lyrics/${TRACK_ID}" -w "\nHTTP Status: %{http_code}\n"
echo ""

sleep 1

# Test 7: Verify deletion
echo "7. Verifying deletion (should return 404)..."
curl -s "${SERVER}/lyrics/${TRACK_ID}" -w "\nHTTP Status: %{http_code}\n" | head -1
echo ""

# Test 8: Check track info again (should show has_lyrics: false)
echo "8. Checking track info again (has_lyrics should be false)..."
curl -s "${SERVER}/tracks/${TRACK_ID}" | jq '{id, title, artist, has_lyrics}'
echo ""

echo "========================================"
echo "Tests completed!"
echo "========================================"
