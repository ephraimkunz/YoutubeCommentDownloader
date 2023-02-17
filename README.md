# YoutubeCommentDownloader
Download comments for all the videos in a given YouTube channel.

## Setup
1. Login to [Google Console](console.cloud.google.com). 
2. Enable `YouTube Data API v3`.
3. Create an Oauth Client ID with Application type of `Desktop App`.
4. Download the JSON for that client ID and pass the path to it to the command line interface.

* This grants you 10,000 API requests per day.

## Usage
```
Download all comments on all videos uploaded to a certain Youtube channel and store the output in a JSON file

Usage: youtube-comments [OPTIONS] <CHANNEL_HANDLE>

Arguments:
  <CHANNEL_HANDLE>  Handle of the channel for whose videos comments will be fetched. Ex: @smartereveryday

Options:
  -t, --token-cache-name <TOKEN_CACHE_NAME>
          Name of the file that will be used to cache the oauth token [default: tokencache.json]
  -c, --client-secret-name <CLIENT_SECRET_NAME>
          Name of the file where client secret can be read from. This file should contain the JSON downloaded from the Credentials section of the Google Cloud console [default: client_secret.json]
  -o, --output-name <OUTPUT_NAME>
          Name of the file where comment JSON will be dumped [default: comments.json]
  -h, --help
          Print help
  -V, --version
          Print version
```
