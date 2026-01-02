# PlaySync

A command-line tool for syncing YouTube playlists. PlaySync allows you to automatically keep your YouTube playlists in sync by adding videos from source playlists to a target playlist, eliminating duplicates and saving you time from manual playlist management.

## Features

- **Playlist Synchronization**: Automatically sync videos from multiple source playlists to a target playlist
- **Duplicate Prevention**: Intelligent detection ensures videos aren't added twice
- **Dry-Run Mode**: Preview changes before applying them
- **Configuration Management**: Store and manage your playlist setup locally
- **OAuth2 Authentication**: Secure authentication with YouTube using OAuth2
- **Multiple Playlist Support**: Configure and manage multiple playlist sync rules

## Why Use PlaySync?

- **Save Time**: No more manually adding videos to playlists
- **Stay Updated**: Keep your playlists current with videos from multiple sources
- **Avoid Duplicates**: Never worry about the same video being added multiple times
- **Flexible Configuration**: Set up different sync rules for different playlists
- **Safe Dry-Runs**: Test your sync configuration before making actual changes

## Installation

### Prerequisites

- Rust 1.70 or later
- YouTube API credentials (OAuth2 JSON file)

### Building from Source

```bash
git clone https://github.com/yourusername/playsync
cd playsync
cargo build --release
```

The compiled binary will be located at `target/release/playsync`.

## Setup

### 1. Get YouTube API Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable the YouTube Data API v3
4. Create OAuth2 credentials (Desktop application)
5. Download the credentials as a JSON file

### 2. Configure PlaySync

Set the path to your OAuth2 JSON file:

```bash
playsync config --oauth2-json /path/to/your/oauth2.json
```

### 3. Add Playlists to Sync

Add a target playlist that will receive videos:

```bash
playsync config --add YOUR_TARGET_PLAYLIST_ID
```

When prompted, specify the source playlist IDs you want to sync from (space-separated).

### 4. View Your Configuration

List all configured playlists:

```bash
playsync config --list
```

## Usage

### Sync All Playlists

Sync all configured playlists:

```bash
playsync sync
```

### Sync a Specific Playlist

Sync only one playlist by its ID:

```bash
playsync sync --id YOUR_PLAYLIST_ID
```

### Dry-Run Mode

Preview what would be synced without making changes:

```bash
playsync sync --dry-run
```

For a specific playlist:

```bash
playsync sync --id YOUR_PLAYLIST_ID --dry-run
```

### Configuration Commands

**Reset Configuration**:

```bash
playsync config --reset
```

**Remove a Playlist**:

```bash
playsync config --remove YOUR_PLAYLIST_ID
```

**View Help**:

```bash
playsync config --help
playsync sync --help
```

## Configuration File

PlaySync stores its configuration in your system's config directory:

- **Linux**: `~/.config/rs.playsync/`
- **macOS**: `~/Library/Application Support/rs.playsync/`
- **Windows**: `%APPDATA%\rs.playsync\`

The configuration file (`playsync.toml`) contains:

- OAuth2 JSON file path
- List of playlists with their sync rules

The token cache (`token_cache.json`) is also stored in the same directory for authentication purposes.

## How It Works

1. **Retrieves Videos**: Gets the list of videos from all source playlists
2. **Checks Target**: Compares against videos already in the target playlist
3. **Identifies New Videos**: Finds videos that aren't already in the target
4. **Adds Videos**: Adds new videos to your target playlist
5. **Reports Results**: Shows you which videos were added and any errors

## Troubleshooting

### "OAuth2 JSON path is not set"

Run: `playsync config --oauth2-json /path/to/your/oauth2.json`

### "Failed to authenticate"

- Verify your OAuth2 JSON file path is correct
- Ensure the file hasn't been deleted or moved
- Try deleting `token_cache.json` from your config directory and authenticating again

### Videos not syncing

- Check your source playlist IDs are correct
- Ensure the videos aren't already in the target playlist
- Verify your YouTube API quota hasn't been exceeded
- Try a dry-run first: `playsync sync --dry-run`

## Scheduling Syncs

To run syncs automatically, use your system's task scheduler:

### Linux/macOS (Cron)

```bash
# Edit crontab
crontab -e

# Example: Sync every day at 9 AM
0 9 * * * /path/to/playsync sync
```

### Windows (Task Scheduler)

1. Open Task Scheduler
2. Create a new task to run: `playsync sync` (write the full path to the executable if it is not in your PATH)
3. Set your desired schedule

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For issues, questions, or suggestions, please open an issue on GitHub.
