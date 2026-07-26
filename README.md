# restate-yt-transcript

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/sagikazarmark/restate-yt-transcript/dagger.yaml?style=flat-square)](https://github.com/sagikazarmark/restate-yt-transcript/actions/workflows/dagger.yaml)
[![crates.io](https://img.shields.io/crates/v/restate-yt-transcript?style=flat-square)](https://crates.io/crates/restate-yt-transcript)
[![docs.rs](https://img.shields.io/docsrs/restate-yt-transcript?style=flat-square)](https://docs.rs/restate-yt-transcript)

**A durable YouTube transcript and video metadata service for [Restate](https://restate.dev/).**

The service wraps [`yt-transcript-rs`](https://github.com/akinsella/yt-transcript-rs) and exposes all six operations from `YouTubeTranscriptApi`. YouTube requests run as bounded Restate durable steps, so completed responses are journaled and transient failures are retried.

## Packages

| Package | Description |
| --- | --- |
| [`restate-yt-transcript`](crates/restate-yt-transcript/) | Embeddable Restate service |
| [`restate-yt-transcript-endpoint`](crates/restate-yt-transcript-endpoint/) | Standalone HTTP endpoint |

## Quick Start

Run the server image and register it with Restate:

```bash
docker run -p 9080:9080 ghcr.io/sagikazarmark/restate-yt-transcript:latest
restate deployments register http://localhost:9080
```

Fetch a transcript through Restate ingress:

```bash
curl localhost:8080/YouTubeTranscript/fetchTranscript \
  -H 'content-type: application/json' \
  -d '{"video_id":"dQw4w9WgXcQ","languages":["en"]}'
```

See the [library README](crates/restate-yt-transcript/README.md) for the complete API and the [endpoint README](crates/restate-yt-transcript-endpoint/README.md) for configuration.

## Operational Notes

- The upstream crate uses unofficial, undocumented YouTube interfaces. YouTube changes can break retrieval, and version 0.1.8 contains internal `unwrap` calls that may panic on exceptional client or cookie data.
- The default endpoint does not use cookies or proxies. Embed the library service and inject a configured `YouTubeTranscriptApi` when those are required.
- Transcript-list and streaming-data responses may contain signed YouTube URLs. Set `restate.service.ingress_private = true` when clients must not invoke the service directly.
- Each YouTube operation attempts at most five times with exponential backoff. Known semantic failures are terminal; blocked and generic request failures are retried within that bound.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## License

Licensed under either the Apache License, Version 2.0 or the MIT license, at your option.
