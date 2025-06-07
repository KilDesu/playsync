use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

pub struct YouTubeClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistInfo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    pub video_id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
struct PlaylistResponse {
    items: Vec<PlaylistResponseItem>,
}

#[derive(Debug, Deserialize)]
struct PlaylistResponseItem {
    snippet: PlaylistSnippet,
}

#[derive(Debug, Deserialize)]
struct PlaylistSnippet {
    title: String,
}

#[derive(Debug, Deserialize)]
struct PlaylistItemsResponse {
    items: Vec<PlaylistItemResponseItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlaylistItemResponseItem {
    snippet: PlaylistItemSnippet,
}

#[derive(Debug, Deserialize)]
struct PlaylistItemSnippet {
    title: String,
    #[serde(rename = "resourceId")]
    resource_id: ResourceId,
}

#[derive(Debug, Deserialize)]
struct ResourceId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Debug, Serialize)]
struct InsertPlaylistItemRequest {
    snippet: InsertPlaylistItemSnippet,
}

#[derive(Debug, Serialize)]
struct InsertPlaylistItemSnippet {
    #[serde(rename = "playlistId")]
    playlist_id: String,
    #[serde(rename = "resourceId")]
    resource_id: InsertResourceId,
}

#[derive(Debug, Serialize)]
struct InsertResourceId {
    kind: String,
    #[serde(rename = "videoId")]
    video_id: String,
}

impl YouTubeClient {
    pub fn new(api_key_env: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var(api_key_env)
            .map_err(|_| format!("Environment variable {} not found", api_key_env))?;

        Ok(YouTubeClient {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn get_playlist_info(
        &self,
        playlist_id: &str,
    ) -> Result<PlaylistInfo, Box<dyn std::error::Error>> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/playlists?part=snippet,contentDetails&id={}&key={}",
            playlist_id, self.api_key
        );

        let response: PlaylistResponse = self.client.get(&url).send().await?.json().await?;

        if let Some(item) = response.items.first() {
            Ok(PlaylistInfo {
                title: item.snippet.title.clone(),
            })
        } else {
            Err("Playlist not found".into())
        }
    }

    pub async fn get_playlist_items(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<PlaylistItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();
        let mut page_token = None;

        loop {
            let mut url = format!(
                "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=50&key={}",
                playlist_id, self.api_key
            );

            if let Some(token) = &page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response: PlaylistItemsResponse =
                self.client.get(&url).send().await?.json().await?;

            for item in response.items {
                items.push(PlaylistItem {
                    video_id: item.snippet.resource_id.video_id,
                    title: item.snippet.title,
                });
            }

            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }

        Ok(items)
    }

    pub async fn add_video_to_playlist(
        &self,
        playlist_id: &str,
        video_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&key={}",
            self.api_key
        );

        let request = InsertPlaylistItemRequest {
            snippet: InsertPlaylistItemSnippet {
                playlist_id: playlist_id.to_string(),
                resource_id: InsertResourceId {
                    kind: "youtube#video".to_string(),
                    video_id: video_id.to_string(),
                },
            },
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to add video to playlist: {}", response.status()).into());
        }

        Ok(())
    }
}
