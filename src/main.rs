use clap::{Parser, Subcommand};
use cliclack::{confirm, intro, note, outro};

mod config;
mod sync;
mod youtube;

use youtube::YouTubeClient;

#[derive(Parser, Debug)]
struct Cli {
    /// The command to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage playlist configuration
    Config(config::ConfigArgs),
    /// Sync playlists based on configuration
    Sync {
        /// Playlist ID to sync (optional, syncs all if not specified)
        #[clap(short = 'i', long = "id", value_name = "PLAYLIST_ID")]
        playlist_id: Option<String>,
        /// Perform a dry run without making changes
        #[clap(short = 'd', long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    let mut youtube_client = None;

    if matches!(cli.command, Commands::Sync { .. })
        || matches!(
            cli.command,
            Commands::Config(config::ConfigArgs { add: _, .. })
        )
    {
        // Ensure the OAuth2 JSON path is set before proceeding with sync or config reset
        let cfg = config::Config::read().unwrap_or_default();
        if cfg.oauth2_json.is_none() {
            outro("❌ The path to the OAuth2 JSON file is not set. Please set it before syncing.")?;
            return Err("OAuth2 JSON path is not set".into());
        }

        let oauth2_json = cfg
            .oauth2_json
            .as_ref()
            .ok_or("OAuth2 JSON path is not set")?;

        youtube_client = Some(YouTubeClient::new(oauth2_json).await?);
    }

    match cli.command {
        Commands::Config(args) => handle_config(args, youtube_client).await?,
        Commands::Sync {
            playlist_id,
            dry_run,
        } => handle_sync(playlist_id, dry_run, youtube_client).await?,
    }

    Ok(())
}

async fn handle_config(
    args: config::ConfigArgs,
    youtube_client: Option<YouTubeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    intro("📝 Playlist Configuration")?;

    let mut cfg = config::Config::read().unwrap_or_default();

    if args.reset {
        let confirmed = confirm("Are you sure you want to reset the configuration?").interact()?;

        if confirmed {
            cfg = config::Config::default();
            cfg.write()?;
            outro("✅ Configuration reset successfully")?;
        }
        return Ok(());
    }

    if !args.oauth2_json.is_none() {
        cfg.set_oauth_path(args.oauth2_json.clone());
        cfg.write()?;
        outro("✅ OAuth2 JSON path set successfully")?;
    }

    if !args.add.is_empty() {
        let client = youtube_client.ok_or_else(|| {
            let _ = outro("❌ YouTube client is not initialized.");
            "YouTube client is not initialized"
        })?;

        match client.get_playlist_title(&args.add).await {
            Ok(playlist_title) => {
                let sync_from = if cfg.playlists.len() > 0 {
                    config::ask_for_sync_items(args.add.clone())
                } else {
                    Vec::new()
                };

                let playlist = config::Playlist {
                    id: args.add.clone(),
                    title: playlist_title,
                    sync_from: if sync_from.is_empty() {
                        None
                    } else {
                        Some(sync_from)
                    },
                };

                cfg.add_playlist(playlist);
                cfg.write()?;
                outro("✅ Playlist added successfully")?;
            }
            Err(e) => {
                outro(&format!("❌ Failed to fetch playlist info: {}", e))?;
                return Err(e);
            }
        }
    }

    if !args.remove.is_empty() {
        cfg.remove_playlist(&args.remove);
        cfg.write()?;
        outro("✅ Playlist removed successfully")?;
    }

    if args.list {
        if let Some(oauth2_json) = &cfg.oauth2_json {
            note("OAuth2 JSON path", oauth2_json)?;
        } else {
            note("OAuth2 JSON path", "<not set>")?;
        }

        intro("📜 Listing all playlists:")?;

        for playlist in &cfg.playlists {
            let playlist_msg = format!("{} (ID: {})", playlist.title, playlist.id);

            if playlist.sync_from.is_some() {
                let mut sync_sources_msg = String::new();

                for sync_id in playlist.sync_from.as_ref().unwrap() {
                    if let Some(sync_playlist) = &cfg.playlists.iter().find(|p| p.id == *sync_id) {
                        sync_sources_msg.push_str(&format!(
                            "{} (ID: {})\n",
                            sync_playlist.title, sync_playlist.id
                        ));
                    } else {
                        sync_sources_msg.push_str(&format!("Unknown Playlist ID: {}\n", sync_id));
                    }
                }

                note(playlist_msg, &sync_sources_msg)?;
            } else {
                note(playlist_msg, "No sync sources")?;
            }
        }

        outro("✅ Configuration listing completed")?;
    }

    Ok(())
}

async fn handle_sync(
    playlist_id: Option<String>,
    dry_run: bool,
    youtube_client: Option<YouTubeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    intro(if dry_run {
        "🔍 Playlist Sync (Dry Run)"
    } else {
        "🔄 Playlist Sync"
    })?;

    let cfg = config::Config::read()?;

    let playlists_to_sync = if let Some(id) = playlist_id {
        cfg.playlists.into_iter().filter(|p| p.id == id).collect()
    } else {
        cfg.playlists
    };

    if playlists_to_sync.is_empty() {
        outro("❌ No playlists found to sync")?;
        return Ok(());
    }

    let client = youtube_client.ok_or_else(|| {
        let _ = outro("❌ YouTube client is not initialized.");
        "YouTube client is not initialized"
    })?;

    for playlist in playlists_to_sync {
        if let Some(sync_from) = &playlist.sync_from {
            sync::sync_playlist(&client, &playlist, sync_from, dry_run).await?;
        }
    }

    outro(if dry_run {
        "✅ Dry run completed"
    } else {
        "✅ Sync completed"
    })?;
    Ok(())
}
