use restate_sdk::{discovery::ServiceType, service::Discoverable};
use yt_transcript_rs::{
    CouldNotRetrieveTranscript, FetchedTranscript, TranscriptList,
    errors::CouldNotRetrieveTranscriptReason, models::FetchedTranscriptSnippet,
};

use crate::{
    FetchMicroformatResponse, FetchStreamingDataResponse, FetchTranscriptRequest,
    FetchTranscriptResponse, FetchVideoDetailsResponse, FetchVideoInfosResponse,
    ListTranscriptsResponse, VideoRequest, YouTubeTranscript,
    service::{map_retrieval_error, terminal_error_code},
};

#[test]
fn fetch_request_defaults_to_plain_text() {
    let request: FetchTranscriptRequest = serde_json::from_value(serde_json::json!({
        "video_id": "dQw4w9WgXcQ",
        "languages": ["en", "de"]
    }))
    .unwrap();

    assert!(!request.preserve_formatting);
}

#[test]
fn requests_reject_empty_values() {
    assert!(VideoRequest::new("").is_err());
    assert!(FetchTranscriptRequest::new("video", Vec::<String>::new(), false).is_err());
    assert!(FetchTranscriptRequest::new("video", ["en", ""], false).is_err());
}

#[test]
fn discovers_youtube_transcript_api() {
    let service = <YouTubeTranscript as Discoverable>::discover();

    assert_eq!(service.name.as_str(), "YouTubeTranscript");
    assert_eq!(service.ty, ServiceType::Service);

    let mut handlers: Vec<_> = service
        .handlers
        .iter()
        .map(|handler| handler.name.as_str())
        .collect();
    handlers.sort_unstable();
    assert_eq!(
        handlers,
        [
            "fetchMicroformat",
            "fetchStreamingData",
            "fetchTranscript",
            "fetchVideoDetails",
            "fetchVideoInfos",
            "listTranscripts",
        ]
    );
}

#[test]
fn transcript_response_preserves_upstream_json_and_describes_it() {
    let response = FetchTranscriptResponse(FetchedTranscript {
        snippets: vec![FetchedTranscriptSnippet {
            text: "Hello".to_owned(),
            start: 1.5,
            duration: 2.0,
        }],
        video_id: "video".to_owned(),
        language: "English".to_owned(),
        language_code: "en".to_owned(),
        is_generated: false,
    });

    assert_eq!(
        serde_json::to_value(response).unwrap(),
        serde_json::json!({
            "snippets": [{"text": "Hello", "start": 1.5, "duration": 2.0}],
            "video_id": "video",
            "language": "English",
            "language_code": "en",
            "is_generated": false
        })
    );

    let schema = serde_json::to_value(schemars::schema_for!(FetchTranscriptResponse)).unwrap();
    let object_schema = schema
        .get("properties")
        .map(|_| &schema)
        .or_else(|| {
            schema["$defs"]
                .as_object()?
                .values()
                .find(|definition| definition["properties"].get("video_id").is_some())
        })
        .expect("response schema must describe transcript fields");
    let required = object_schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("snippets")));
    assert!(required.contains(&serde_json::json!("video_id")));
    assert_eq!(object_schema["properties"]["snippets"]["type"], "array");
}

#[test]
fn response_schemas_match_upstream_0_1_8_shapes() {
    assert_schema_object::<FetchTranscriptResponse>(&[
        "snippets",
        "video_id",
        "language",
        "language_code",
        "is_generated",
    ]);
    assert_schema_object::<FetchTranscriptResponse>(&["text", "start", "duration"]);

    assert_schema_object::<ListTranscriptsResponse>(&[
        "video_id",
        "manually_created_transcripts",
        "generated_transcripts",
        "translation_languages",
    ]);
    assert_schema_object::<ListTranscriptsResponse>(&[
        "video_id",
        "url",
        "language",
        "language_code",
        "is_generated",
        "translation_languages",
        "translation_languages_map",
    ]);
    assert_schema_object::<ListTranscriptsResponse>(&["language", "language_code"]);

    assert_schema_object::<FetchVideoDetailsResponse>(&[
        "video_id",
        "title",
        "length_seconds",
        "keywords",
        "channel_id",
        "short_description",
        "view_count",
        "author",
        "thumbnails",
        "is_live_content",
    ]);
    assert_schema_object::<FetchVideoDetailsResponse>(&["url", "width", "height"]);

    assert_schema_object::<FetchMicroformatResponse>(&[
        "available_countries",
        "category",
        "description",
        "embed",
        "external_channel_id",
        "external_video_id",
        "has_ypc_metadata",
        "is_family_safe",
        "is_shorts_eligible",
        "is_unlisted",
        "length_seconds",
        "like_count",
        "owner_channel_name",
        "owner_profile_url",
        "publish_date",
        "thumbnail",
        "title",
        "upload_date",
        "view_count",
    ]);
    assert_schema_object::<FetchMicroformatResponse>(&["height", "iframe_url", "width"]);
    assert_schema_object::<FetchMicroformatResponse>(&["thumbnails"]);

    assert_schema_object::<FetchStreamingDataResponse>(&[
        "expires_in_seconds",
        "formats",
        "adaptive_formats",
        "server_abr_streaming_url",
    ]);
    assert_schema_object::<FetchStreamingDataResponse>(&[
        "itag",
        "url",
        "mime_type",
        "bitrate",
        "width",
        "height",
        "init_range",
        "index_range",
        "last_modified",
        "content_length",
        "quality",
        "fps",
        "quality_label",
        "projection_type",
        "average_bitrate",
        "audio_quality",
        "approx_duration_ms",
        "audio_sample_rate",
        "audio_channels",
        "quality_ordinal",
        "high_replication",
        "color_info",
        "loudness_db",
        "is_drc",
        "xtags",
    ]);
    assert_schema_object::<FetchStreamingDataResponse>(&["start", "end"]);
    assert_schema_object::<FetchStreamingDataResponse>(&[
        "primaries",
        "transfer_characteristics",
        "matrix_coefficients",
    ]);

    assert_schema_object::<FetchVideoInfosResponse>(&[
        "video_details",
        "microformat",
        "streaming_data",
        "transcript_list",
    ]);
}

fn assert_schema_object<T: schemars::JsonSchema>(expected_fields: &[&str]) {
    let schema = serde_json::to_value(schemars::schema_for!(T)).unwrap();
    let matches = |candidate: &serde_json::Value| {
        let Some(properties) = candidate["properties"].as_object() else {
            return false;
        };
        properties.len() == expected_fields.len()
            && expected_fields
                .iter()
                .all(|field| properties.contains_key(*field))
    };

    assert!(
        matches(&schema)
            || schema["$defs"]
                .as_object()
                .is_some_and(|definitions| definitions.values().any(matches)),
        "schema does not contain object with fields {expected_fields:?}"
    );
}

#[test]
fn classifies_semantic_and_operational_errors() {
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::InvalidVideoId),
        Some(400)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::AgeRestricted),
        Some(403)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::VideoUnavailable),
        Some(404)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::NoTranscriptFound {
            requested_language_codes: vec!["en".to_owned()],
            transcript_data: TranscriptList::new(
                "video".to_owned(),
                Default::default(),
                Default::default(),
                Vec::new(),
            ),
        }),
        Some(404)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::TranscriptsDisabled),
        Some(422)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::VideoUnplayable {
            reason: None,
            sub_reasons: Vec::new(),
        }),
        Some(422)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::TranslationUnavailable(
            "disabled".to_owned()
        )),
        Some(422)
    );
    assert_eq!(
        terminal_error_code(
            &CouldNotRetrieveTranscriptReason::TranslationLanguageUnavailable("unknown".to_owned())
        ),
        Some(422)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie),
        Some(500)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
            "changed".to_owned()
        )),
        Some(502)
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::IpBlocked(None)),
        None
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::RequestBlocked(None)),
        None
    );
    assert_eq!(
        terminal_error_code(&CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
            "timeout".to_owned()
        )),
        None
    );
    let unknown = map_retrieval_error(CouldNotRetrieveTranscript {
        video_id: "video".to_owned(),
        reason: None,
    });
    assert!(format!("{unknown:?}").contains("Retryable"));
}
