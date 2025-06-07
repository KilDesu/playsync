use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// Add a new playlist to the configuration
    #[clap(
        short = 'a',
        long,
        alias = "add-playlist",
        value_name = "PLAYLIST_ID",
        default_value = ""
    )]
    pub add: String,

    /// Remove a playlist from the configuration
    #[clap(
        short = 'r',
        long,
        alias = "remove-playlist",
        value_name = "PLAYLIST_ID",
        default_value = ""
    )]
    pub remove: String,

    /// List all playlists in the configuration
    #[clap(short = 'l', long, alias = "list-playlists")]
    pub list: bool,

    /// Reset the configuration to default values
    #[clap(long)]
    pub reset: bool,

    /// Set the API key for the configuration
    #[clap(short = 'k', long, default_value = "PLAYSYNC_YOUTUBE_API_KEY")]
    pub api_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Name of the environment variable holding the YouTube API key
    pub api_key: String,

    /// List of playlists to sync
    pub playlists: Vec<Playlist>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Playlist {
    /// The ID of the playlist
    pub id: String,

    /// The title of the playlist
    pub title: String,

    /// Optionally specify playlists to sync from
    /// The playlists should be specified as a space-separated list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_from: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: "PLAYSYNC_YOUTUBE_API_KEY".to_string(),
            playlists: Vec::new(),
        }
    }
}

impl Config {
    /// Add a playlist to the configuration
    pub fn add_playlist(&mut self, playlist: Playlist) -> &Self {
        self.playlists.push(playlist);

        self
    }

    /// Remove a playlist by its ID
    pub fn remove_playlist(&mut self, id: &str) -> &Self {
        self.playlists.retain(|p| p.id != id);

        self
    }

    /// Set the API key for the configuration
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    /// Read the configuration from the file
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let cfg: Config = confy::load("playsync", None)?;

        Ok(cfg)
    }

    /// Write the configuration to the file
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        confy::store("playsync", None, self)?;

        Ok(())
    }
}

/// Ask the user to select playlists to sync from/to.
///
/// This function will present a list of playlists that are not the current playlist
/// and that do not already have a sync relationship with the current playlist.
/// It will return a vector of playlist IDs that the user has selected.
pub fn ask_for_sync_items(playlist_id: String) -> Vec<String> {
    use cliclack::multiselect;

    let cfg = Config::read().unwrap_or_default();
    let playlists = cfg
        .playlists
        .iter()
        .filter(|p| {
            // Skip the current playlist
            if p.id == playlist_id {
                return false;
            }

            // Skip playlists that are already set to sync from the current playlist
            // This is to prevent circular dependencies
            if let Some(sync_from) = &p.sync_from {
                return !sync_from.contains(&playlist_id);
            }

            true
        })
        .collect::<Vec<&Playlist>>();

    if playlists.is_empty() {
        return Vec::new();
    }

    let items: Vec<(String, String, &str)> = playlists
        .iter()
        .map(|p| (p.id.clone(), p.title.clone(), ""))
        .collect();

    let selected = multiselect("Select playlists to sync from:")
        .items(&items)
        .filter_mode()
        .required(false)
        .interact()
        .unwrap_or_default();

    selected
}
