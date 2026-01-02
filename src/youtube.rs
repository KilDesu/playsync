use google_youtube3::{
    YouTube,
    api::{PlaylistItem, PlaylistItemSnippet, ResourceId},
    hyper_rustls, hyper_util, yup_oauth2,
};

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub video_id: String,
    pub title: String,
}

pub struct YouTubeClient {
    hub: YouTube<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

impl YouTubeClient {
    pub async fn new(oauth_json_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Read OAuth2 credentials from the provided JSON file
        let secret = yup_oauth2::read_application_secret(oauth_json_path).await?;

        // Get the app data directory for token cache
        let cache_dir = confy::get_configuration_file_path("playsync", None)?
            .parent()
            .ok_or("Failed to get config directory")?
            .to_path_buf();

        std::fs::create_dir_all(&cache_dir)?;
        let token_cache_path = cache_dir.join("token_cache.json");

        // Create an authenticator with token persistence and required scopes
        let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk(token_cache_path)
        .build()
        .await?;

        // Force authentication with all required scopes upfront
        let scopes = &[
            "https://www.googleapis.com/auth/youtube.readonly",
            "https://www.googleapis.com/auth/youtube",
        ];
        let _ = auth.token(scopes).await?;

        // Create HTTPS connector
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_or_http()
            .enable_http1()
            .build();

        // Create the YouTube API hub
        let hub = YouTube::new(
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(connector),
            auth,
        );

        Ok(Self { hub })
    }

    pub async fn get_playlist_title(
        &self,
        playlist_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let result = self
            .hub
            .playlists()
            .list(&vec!["snippet".to_string()])
            .add_id(playlist_id)
            .doit()
            .await?;

        if let Some(items) = result.1.items {
            if let Some(playlist) = items.first() {
                if let Some(snippet) = &playlist.snippet {
                    return Ok(snippet.title.clone().unwrap_or_default());
                }
            }
        }

        Err("Playlist not found".into())
    }

    pub async fn get_playlist_items(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<VideoInfo>, Box<dyn std::error::Error>> {
        let mut videos = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut request = self
                .hub
                .playlist_items()
                .list(&vec!["snippet".to_string(), "contentDetails".to_string()])
                .playlist_id(playlist_id)
                .max_results(50);

            if let Some(token) = &page_token {
                request = request.page_token(token);
            }

            let result = request.doit().await?;

            if let Some(items) = result.1.items {
                for item in items {
                    if let (Some(snippet), Some(content_details)) =
                        (&item.snippet, &item.content_details)
                    {
                        if let Some(video_id) = &content_details.video_id {
                            videos.push(VideoInfo {
                                video_id: video_id.clone(),
                                title: snippet.title.clone().unwrap_or_default(),
                            });
                        }
                    }
                }
            }

            page_token = result.1.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        Ok(videos)
    }

    pub async fn add_video_to_playlist(
        &self,
        playlist_id: &str,
        video_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let playlist_item = PlaylistItem {
            snippet: Some(PlaylistItemSnippet {
                playlist_id: Some(playlist_id.to_string()),
                resource_id: Some(ResourceId {
                    kind: Some("youtube#video".to_string()),
                    video_id: Some(video_id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        self.hub
            .playlist_items()
            .insert(playlist_item)
            .add_part("snippet")
            .doit()
            .await?;

        Ok(())
    }
}
