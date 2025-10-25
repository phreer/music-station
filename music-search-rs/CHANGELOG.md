# Changelog

## [Unreleased] - 2025-10-25

### Added

#### Command-Line Arguments Support
- Added `clap` dependency for robust CLI argument parsing
- New `--api` / `-a` option to specify music service (netease/qq) directly
- New `--cookie` / `-c` option for authentication cookie
- New `--query` / `-q` option to provide search query via command line
- Support for multiple API name aliases:
  - NetEase: `netease`, `ne`, `163`, `1`
  - QQ Music: `qq`, `qqmusic`, `tencent`, `2`

#### Environment Variable Support
- `MUSIC_COOKIE` environment variable for cookie authentication
- Can be used as alternative to `--cookie` command-line argument
- Automatically detected by clap with `#[arg(env = "MUSIC_COOKIE")]`

#### Non-Interactive Mode
- Binary can now run without user interaction when all arguments provided
- Useful for scripting and automation
- Still falls back to interactive prompts for missing information

### Changed
- Binary now supports both interactive and non-interactive modes
- Updated help message with detailed option descriptions
- Improved user experience with clearer service name display

### Fixed
- Fixed UTF-8 string slicing panic in binary (same issue as examples)
- Changed from byte-based slicing to character-based truncation
- Prevents crashes when displaying Chinese characters in song/artist/album names

### Bug Fixes Applied (from previous fixes)
- QQ Music song ID type mismatch (integer vs string)
- QQ Music playlist field name inconsistency (`song_count` vs `song_Count`)
- UTF-8 string boundary issues in examples

## Usage Examples

### Interactive Mode (Original Behavior)
```bash
./target/release/music_search
# User will be prompted for service selection and search query
```

### Command-Line Mode (New)
```bash
# Specify everything via arguments
./target/release/music_search --api qq --query "告白气球"

# With authentication
./target/release/music_search --api netease --cookie "MUSIC_U=..." --query "周杰伦"

# Using environment variable
export MUSIC_COOKIE="MUSIC_U=..."
./target/release/music_search --api netease --query "稻香"
```

### Help and Version
```bash
# Display help
./target/release/music_search --help

# Display version
./target/release/music_search --version
```

## Technical Details

### Dependencies Added
- `clap = { version = "4.0", features = ["derive", "env"] }`

### Code Changes
- Modified `src/bin/music_search.rs`:
  - Added `Args` struct with `#[derive(Parser)]`
  - Implemented conditional logic for interactive vs CLI mode
  - Fixed UTF-8 string truncation using `.chars().take(n).collect()`
  
### Backward Compatibility
- ✅ Fully backward compatible
- If no arguments provided, behaves exactly as before (interactive mode)
- No breaking changes to existing functionality

### Benefits
1. **Automation**: Can be scripted or used in CI/CD pipelines
2. **Security**: Cookie can be passed via environment variable (safer than command line)
3. **Convenience**: Skip interactive prompts for known queries
4. **Flexibility**: Mix interactive and non-interactive modes as needed

## Migration Guide

### For Users

**Old way (still works):**
```bash
./target/release/music_search
# Then type: 2
# Then type: 告白气球
```

**New way (faster):**
```bash
./target/release/music_search --api qq --query "告白气球"
```

### For Scripts

**Before:**
```bash
echo -e "2\n告白气球\n1\n1\n" | ./target/release/music_search
```

**After:**
```bash
export MUSIC_COOKIE="your_cookie"
./target/release/music_search --api qq --query "告白气球"
# Still needs interactive input for song selection and lyrics type
```

## Known Limitations

- Song selection and lyrics type selection still require interactive input
- Full non-interactive mode (download without prompts) not yet implemented
- Could be enhanced in future with additional flags like `--song-number` and `--lyrics-type`

## Future Enhancements

Potential features for future versions:
- `--song-index` to auto-select song from results
- `--lyrics-type` to specify which lyrics to download (original/translation/all)
- `--output-dir` to specify custom output directory
- `--format` to choose output format (lrc/txt/json)
- `--batch` mode to process multiple queries from file
- `--no-interactive` flag to exit on errors instead of prompting

## Documentation Updates

- Updated `BINARY_USAGE.md` with command-line options section
- Updated `README.md` with new quick start examples
- Created this `CHANGELOG.md` to track changes
- Added usage examples for all new features
