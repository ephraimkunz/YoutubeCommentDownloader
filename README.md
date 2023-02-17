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

## Output Format
```json
[
  {
    "title": "Knowledge From Facts OR Experience? (Saber & Conocer)",
    "id": "C6D_tFJeLWk",
    "comments": [
      {
        "text": "Reminds me of Elder Bednar’s talk where he discusses Testimony va conversion.",
        "author_name": "Zion Mama",
        "children": []
      },
      {
        "text": "so it's theroretically and experimental",
        "author_name": "space fan",
        "children": [
          {
            "text": "That is the process, not the end. Knowledge is gained through experience and experiment.",
            "author_name": "Cwic Media"
          }
        ]
      }
    ]
  },
  {
    "title": "The Most Precious Things In Life",
    "id": "0cTXYmmazQ8",
    "comments": [
      {
        "text": "\"When facts come secondary to emotion, truth dies. A society that doesn't value truth cannot survive.\" - Ben Shapiro",
        "author_name": "CoffeeDrinkingIsNotASin",
        "children": [
          {
            "text": "@Chischili Snez Objective Truth is what exists and can be proved in this physicality. ...\n\nNormative Truth is what we, as a group, agree is true. ...\n\nSubjective Truth is how the individual sees or experiences the world.",
            "author_name": "CoffeeDrinkingIsNotASin"
          },
          {
            "text": "As Pilate said to Jesus, \"What is truth?\"",
            "author_name": "Chischili Snez"
          }
        ]
      },
      {
        "text": "Exactly, lots of factors play into how we know something.",
        "author_name": "Joscelyn Pease",
        "children": []
      }
    ]
  }
]
```
