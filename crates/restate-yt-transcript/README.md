# restate-yt-transcript

Embeddable YouTube transcript and video metadata service for [Restate](https://restate.dev/).

## Install

```bash
cargo add restate-yt-transcript
```

## Bind The Service

Use the default unauthenticated YouTube client:

```rust
use restate_sdk::{endpoint::Endpoint, service::IntoServiceDefinition};
use restate_yt_transcript::YouTubeTranscript;

let service = YouTubeTranscript::try_default()?.into_service_definition();
let endpoint = Endpoint::builder().bind(service).build();
```

Or inject a client configured with cookies, proxies, or a custom `reqwest::Client`:

```rust
use restate_yt_transcript::YouTubeTranscript;
use yt_transcript_rs::YouTubeTranscriptApi;

let api = YouTubeTranscriptApi::new(cookie_path, proxy_config, http_client)?;
let service = YouTubeTranscript::new(api);
```

## API

The stateless `YouTubeTranscript` service exposes:

| Handler | Input | Output |
| --- | --- | --- |
| `fetchTranscript` | `FetchTranscriptRequest` | fetched transcript and snippets |
| `listTranscripts` | `VideoRequest` | available manual/generated transcripts |
| `fetchVideoDetails` | `VideoRequest` | basic video metadata |
| `fetchMicroformat` | `VideoRequest` | microformat metadata |
| `fetchStreamingData` | `VideoRequest` | available streaming formats and URLs |
| `fetchVideoInfos` | `VideoRequest` | aggregate details, microformat, streaming data, and transcripts |

`VideoRequest`:

```json
{"video_id":"dQw4w9WgXcQ"}
```

`FetchTranscriptRequest`:

```json
{
  "video_id": "dQw4w9WgXcQ",
  "languages": ["en", "de"],
  "preserve_formatting": false
}
```

`preserve_formatting` defaults to `false`. Empty video IDs, language lists, and language entries return terminal `400` errors. Output field names and values preserve the upstream crate's snake_case Serde representation.

YouTube calls execute in named durable runs with five attempts, exponential backoff from one second, and a 30-second maximum delay. Semantic failures are terminal; blocked and generic request failures are retried within that bound.

## License

Licensed under either the Apache License, Version 2.0 or the MIT license, at your option.
