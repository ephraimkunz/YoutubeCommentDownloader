use indicatif::{ProgressIterator, ProgressStyle};
use std::fs::File;

use anyhow::{bail, Context, Result};
use google_youtube3::{
    hyper,
    hyper_rustls::{self, HttpsConnector},
    oauth2, YouTube,
};
use hyper::client::HttpConnector;
use serde::{Deserialize, Serialize};

use clap::Parser;

#[derive(Parser)]
#[command(author, version)]
/// Download all comments on all videos uploaded to a certain Youtube channel and store the output in a JSON file.
struct Cli {
    /// Handle of the channel for whose videos comments will be fetched. Ex: @smartereveryday
    channel_handle: String,

    /// Name of the file that will be used to cache the oauth token.
    #[arg(default_value = "tokencache.json")]
    token_cache_name: String,

    /// Name of the file where client secret can be read from. This file should contain the JSON downloaded from the Credentials section of the Google Cloud console.
    #[arg(default_value = "client_secret.json")]
    client_secret_name: String,

    /// Name of the file where comment JSON will be dumped.
    #[arg(default_value = "comments.json")]
    output_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct ParentComment {
    text: String,
    author_name: String,
    children: Vec<ChildComment>,
}

#[derive(Debug, Clone, Serialize)]
struct ChildComment {
    text: String,
    author_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct Video {
    title: String,
    id: String,
    comments: Vec<ParentComment>,
}

#[derive(Debug, Clone)]
struct PlaylistItem {
    title: String,
    video_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BadRequest {
    error: ErrorResponse,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorResponse {
    code: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct HandleLookup {
    items: Vec<HandleLookupItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct HandleLookupItem {
    id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let youtube = create_youtube_client(&cli.client_secret_name, &cli.token_cache_name).await?;
    let channel_id = get_channel_id(&cli.channel_handle).await?;
    let upload_playlist_id = get_upload_playlist_id(&channel_id, &youtube).await?;
    let playlist_items = get_playlist_items(&upload_playlist_id, &youtube).await?;

    let mut videos: Vec<Video> = Vec::with_capacity(playlist_items.len());
    let progress_style = ProgressStyle::with_template(
        "[elapsed:{elapsed}] [remaining:{eta}] {bar:50} {pos}/{len}",
    )?;
    for playlist_item in playlist_items.iter().progress_with_style(progress_style) {
        let video = Video {
            title: playlist_item.title.clone(),
            id: playlist_item.video_id.clone(),
            comments: get_comments(&playlist_item.video_id, &youtube).await?,
        };
        videos.push(video);
    }

    let output_file = File::create(cli.output_name)?;
    serde_json::to_writer_pretty(output_file, &videos)?;

    Ok(())
}

async fn create_youtube_client(
    client_secret_name: &str,
    token_cache_name: &str,
) -> Result<YouTube<HttpsConnector<HttpConnector>>> {
    let json = std::fs::read_to_string(client_secret_name)?;
    let secret: oauth2::ConsoleApplicationSecret = serde_json::from_str(&json)?;
    let application_secret = secret.installed.context("Unable to read client secret")?;

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        application_secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk(token_cache_name)
    .build()
    .await
    .expect("Unable to build authenticator");

    let scopes = &[
        "https://www.googleapis.com/auth/youtube.force-ssl",
        "https://www.googleapis.com/auth/youtube.readonly",
    ];

    // Prompt for all scopes here so we don't get multiple prompts as we call apis that use different scopes.
    auth.token(scopes).await?;

    Ok(YouTube::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    ))
}

async fn get_channel_id(handle: &str) -> Result<String> {
    // See https://stackoverflow.com/questions/74323173/how-to-map-youtube-handles-to-channel-ids

    let handle = handle.strip_prefix('@').unwrap_or(handle);
    let response: HandleLookup = reqwest::get(format!(
        "https://yt.lemnoslife.com/channels?handle=@{}",
        handle
    ))
    .await?
    .json()
    .await
    .context("Unable to find channel id given handle")?;

    Ok(response
        .items
        .first()
        .context("Unable to find channel id given handle")?
        .id
        .to_string())
}

async fn get_upload_playlist_id(
    channel_id: &str,
    youtube: &YouTube<HttpsConnector<HttpConnector>>,
) -> Result<String> {
    let (_, channel) = youtube
        .channels()
        .list(&vec!["contentDetails".to_string()])
        .add_id(channel_id)
        .doit()
        .await?;

    channel
        .items
        .as_ref()
        .and_then(|i| i.first())
        .and_then(|i| i.content_details.as_ref())
        .and_then(|c| c.related_playlists.as_ref())
        .and_then(|p| p.uploads.as_ref())
        .context("Unable to get upload playlist id")
        .cloned()
}

async fn get_playlist_items(
    playlist_id: &str,
    youtube: &YouTube<HttpsConnector<HttpConnector>>,
) -> Result<Vec<PlaylistItem>> {
    let mut items = vec![];
    let mut playlist_page_token = String::new();

    loop {
        let (_, playlist_items) = youtube
            .playlist_items()
            .list(&vec!["snippet".to_string(), "contentDetails".to_string()])
            .max_results(50)
            .playlist_id(playlist_id)
            .page_token(&playlist_page_token)
            .doit()
            .await?;

        for item in playlist_items.items.unwrap() {
            let video_id = item
                .content_details
                .as_ref()
                .unwrap()
                .video_id
                .as_ref()
                .unwrap()
                .clone();
            let title = item
                .snippet
                .as_ref()
                .unwrap()
                .title
                .as_ref()
                .unwrap()
                .clone();
            items.push(PlaylistItem { title, video_id })
        }

        match playlist_items.next_page_token {
            Some(t) => playlist_page_token = t,
            None => break,
        };
    }

    Ok(items)
}

async fn get_comments(
    video_id: &str,
    youtube: &YouTube<HttpsConnector<HttpConnector>>,
) -> Result<Vec<ParentComment>> {
    let mut thread_page_token = String::new();
    let mut comments: Vec<ParentComment> = vec![];

    loop {
        let result = youtube
            .comment_threads()
            .list(&vec!["snippet".to_string(), "replies".to_string()])
            .text_format("plainText")
            .video_id(video_id)
            .max_results(100)
            .page_token(&thread_page_token)
            .doit()
            .await;

        let threads_response = match result {
            Ok((_, response)) => response,
            Err(google_youtube3::Error::BadRequest(v)) => {
                let error: BadRequest = serde_json::from_value(v)?;
                if error.error.code == 403 {
                    // When a video has disabled comments, Youtube returns a 403. In that case, just return an empty vec of comments instead of failing.
                    return Ok(comments);
                } else {
                    bail!("Unable to parse error response from comment_threads request");
                }
            }
            Err(e) => return Err(e.into()),
        };

        if let Some(items) = threads_response.items {
            for item in &items {
                let Some(parent_comment) = item.snippet.as_ref().and_then(|s| s.top_level_comment.clone()).and_then(|c| c.snippet) else {
                    continue;
                };

                let mut comment = match (
                    parent_comment.text_original,
                    parent_comment.author_display_name,
                ) {
                    (Some(text), Some(author_name)) => ParentComment {
                        text,
                        author_name,
                        children: vec![],
                    },
                    _ => continue,
                };

                let contained_reply_count = item
                    .replies
                    .as_ref()
                    .and_then(|r| r.comments.as_ref())
                    .map_or(0, |c| c.len());
                let total_reply_count = item
                    .snippet
                    .as_ref()
                    .and_then(|s| s.total_reply_count)
                    .unwrap_or(0) as usize;
                if contained_reply_count == total_reply_count {
                    if let Some(child_comment) =
                        item.replies.as_ref().and_then(|r| r.comments.as_ref())
                    {
                        let children = child_comment.iter().filter_map(|cc| {
                            cc.snippet.as_ref().and_then(|s| {
                                match (&s.author_display_name, &s.text_original) {
                                    (Some(author_name), Some(text)) => Some(ChildComment {
                                        text: text.to_string(),
                                        author_name: author_name.to_string(),
                                    }),
                                    _ => None,
                                }
                            })
                        });

                        comment.children.extend(children);
                    }
                } else if let Some(parent_id) = &item.id {
                    let mut comment_page_token = String::new();
                    loop {
                        let (_, comments_response) = youtube
                            .comments()
                            .list(&vec!["snippet".to_string()])
                            .text_format("plainText")
                            .parent_id(parent_id)
                            .max_results(100)
                            .page_token(&comment_page_token)
                            .doit()
                            .await?;

                        if let Some(items) = comments_response.items {
                            let children = items.iter().filter_map(|cc| {
                                cc.snippet.as_ref().and_then(|s| {
                                    match (&s.author_display_name, &s.text_original) {
                                        (Some(author_name), Some(text)) => Some(ChildComment {
                                            text: text.to_string(),
                                            author_name: author_name.to_string(),
                                        }),
                                        _ => None,
                                    }
                                })
                            });

                            comment.children.extend(children);
                        }
                        match comments_response.next_page_token {
                            Some(t) => comment_page_token = t,
                            None => break,
                        };
                    }
                }

                comments.push(comment);
            }
        }

        match threads_response.next_page_token {
            Some(t) => thread_page_token = t,
            None => break,
        };
    }

    Ok(comments)
}
