use crate::config::Playlist;
use crate::youtube::YouTubeClient;
use cliclack::{log, spinner};
use std::collections::HashSet;

pub async fn sync_playlist(
    youtube_client: &YouTubeClient,
    target_playlist: &Playlist,
    source_playlist_ids: &[String],
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let sp = spinner();
    sp.start(&format!("Syncing playlist: {}", target_playlist.title));

    // Get existing videos in target playlist
    let target_videos = youtube_client
        .get_playlist_items(&target_playlist.id)
        .await?;
    let target_video_ids: HashSet<String> = target_videos
        .iter()
        .map(|item| item.video_id.clone())
        .collect();

    let mut videos_to_add = Vec::new();

    // Collect videos from all source playlists
    for source_id in source_playlist_ids {
        let source_videos = youtube_client.get_playlist_items(source_id).await?;

        for video in source_videos {
            if !target_video_ids.contains(&video.video_id) {
                videos_to_add.push(video);
            }
        }
    }

    sp.stop(&format!(
        "Found {} videos to sync to '{}'",
        videos_to_add.len(),
        target_playlist.title
    ));

    if videos_to_add.is_empty() {
        return Ok(());
    }

    if dry_run {
        log::info(&format!("Would add {} videos:", videos_to_add.len()))?;
        for video in &videos_to_add {
            log::info(&format!("  - {}", video.title))?;
        }
        return Ok(());
    }

    // Add videos to target playlist
    let sp = spinner();
    sp.start(&format!(
        "Adding {} videos to playlist",
        videos_to_add.len()
    ));

    let mut added_count = 0;
    for video in videos_to_add {
        match youtube_client
            .add_video_to_playlist(&target_playlist.id, &video.video_id)
            .await
        {
            Ok(_) => {
                added_count += 1;
                log::info(&format!("Added: {}", video.title))?;
            }
            Err(e) => {
                log::warning(&format!("Failed to add '{}': {}", video.title, e))?;
            }
        }
    }

    sp.stop(&format!("Successfully added {} videos", added_count));
    Ok(())
}
