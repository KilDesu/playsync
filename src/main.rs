use clap::{Parser, Subcommand};
use cliclack::{confirm, intro, outro};

mod config;
mod sync;
mod youtube;

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
    let cli = Cli::parse();

    match cli.command {
        Commands::Config(args) => handle_config(args).await?,
        Commands::Sync {
            playlist_id,
            dry_run,
        } => handle_sync(playlist_id, dry_run).await?,
    }

    Ok(())
}

async fn handle_config(args: config::ConfigArgs) -> Result<(), Box<dyn std::error::Error>> {
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

    if !args.api_key.is_empty() && args.api_key != "PLAYSYNC_YOUTUBE_API_KEY" {
        cfg.set_api_key(args.api_key.clone());
        cfg.write()?;
        outro("✅ API key updated successfully")?;
        return Ok(());
    }

    if !args.add.is_empty() {
        let youtube_client = youtube::YouTubeClient::new(&cfg.api_key)?;

        match youtube_client.get_playlist_info(&args.add).await {
            Ok(playlist_info) => {
                let sync_from = if cfg.playlists.len() > 0 {
                    config::ask_for_sync_items(args.add.clone())
                } else {
                    Vec::new()
                };

                let playlist = config::Playlist {
                    id: args.add.clone(),
                    title: playlist_info.title,
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
        outro("📜 Listing all playlists:")?;
        for playlist in &cfg.playlists {
            outro(&format!(" - {} (ID: {})", playlist.title, playlist.id))?;

            if playlist.sync_from.is_some() {
                outro("   Syncs from:")?;
                for sync_id in playlist.sync_from.as_ref().unwrap() {
                    if let Some(sync_playlist) = &cfg.playlists.iter().find(|p| p.id == *sync_id) {
                        outro(&format!(
                            "    - {} (ID: {})",
                            sync_playlist.title, sync_playlist.id
                        ))?;
                    } else {
                        outro(&format!("    - Unknown Playlist ID: {}", sync_id))?;
                    }
                }
            } else {
                outro("   No sync sources")?;
            }
        }
    }

    Ok(())
}

async fn handle_sync(
    playlist_id: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    intro(if dry_run {
        "🔍 Playlist Sync (Dry Run)"
    } else {
        "🔄 Playlist Sync"
    })?;

    let cfg = config::Config::read()?;
    let youtube_client = youtube::YouTubeClient::new(&cfg.api_key)?;

    let playlists_to_sync = if let Some(id) = playlist_id {
        cfg.playlists.into_iter().filter(|p| p.id == id).collect()
    } else {
        cfg.playlists
    };

    if playlists_to_sync.is_empty() {
        outro("❌ No playlists found to sync")?;
        return Ok(());
    }

    for playlist in playlists_to_sync {
        if let Some(sync_from) = &playlist.sync_from {
            sync::sync_playlist(&youtube_client, &playlist, sync_from, dry_run).await?;
        }
    }

    outro(if dry_run {
        "✅ Dry run completed"
    } else {
        "✅ Sync completed"
    })?;
    Ok(())
}
