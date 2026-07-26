# restate-yt-transcript-endpoint

Standalone endpoint hosting the YouTube transcript service for [Restate](https://restate.dev/).

## Run

```bash
restate-yt-transcript --port 9080
restate deployments register http://localhost:9080
```

Or use the container image:

```bash
docker run -p 9080:9080 ghcr.io/sagikazarmark/restate-yt-transcript:latest
```

The standalone endpoint creates `YouTubeTranscriptApi::new(None, None, None)`. Use the library crate when cookies, a proxy, or a custom HTTP client are required.

## Configuration

`--config <FILE>` accepts TOML, JSON, or YAML. `--port <PORT>` defaults to `9080`. The equivalent environment variables are `CONFIG_FILE` and `PORT`; `RUST_LOG` controls logging.

```toml
[restate.service]
inactivity_timeout = "5m"
abort_timeout = "10m"
journal_retention = "24h"
ingress_private = false

retry_policy_initial_interval = "100ms"
retry_policy_exponentiation_factor = 2.0
retry_policy_max_interval = "30s"
retry_policy_max_attempts = 5
retry_policy_on_max_attempts = "pause"

[restate.service.handlers.fetchTranscript]
inactivity_timeout = "3m"
```

Set `ingress_private = true` if signed URLs in transcript-list or streaming-data responses should only be available to other Restate services.

## License

Licensed under either the Apache License, Version 2.0 or the MIT license, at your option.
