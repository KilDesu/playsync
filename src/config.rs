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

    /// Path to the OAuth2 JSON file for YouTube API authentication
    #[clap(
        short = 'o',
        long,
        alias = "oauth2-json",
        value_name = "OAUTH2_JSON_PATH"
    )]
    pub oauth2_json: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// OAuth2 JSON file path for YouTube API authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth2_json: Option<String>,

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
            playlists: Vec::new(),
            oauth2_json: None,
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

    /// Set the OAuth2 JSON file path for the configuration
    pub fn set_oauth_path(&mut self, oauth2_json: Option<String>) {
        self.oauth2_json = oauth2_json;
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
