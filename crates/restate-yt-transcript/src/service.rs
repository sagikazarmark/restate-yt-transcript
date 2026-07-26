use std::time::Duration;

use restate_sdk::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use yt_transcript_rs::{
    CookieError, CouldNotRetrieveTranscript, FetchedTranscript, MicroformatData, StreamingData,
    TranscriptList, VideoDetails, VideoInfos, YouTubeTranscriptApi,
    errors::CouldNotRetrieveTranscriptReason,
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct VideoRequest {
    /// YouTube video ID, not a full YouTube URL.
    pub video_id: String,
}

impl VideoRequest {
    pub fn new(video_id: impl Into<String>) -> Result<Self, TerminalError> {
        let request = Self {
            video_id: video_id.into(),
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> Result<(), TerminalError> {
        if self.video_id.trim().is_empty() {
            return Err(TerminalError::new("video_id must not be empty").with_code(400));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FetchTranscriptRequest {
    /// YouTube video ID, not a full YouTube URL.
    pub video_id: String,
    /// Language codes in preference order.
    pub languages: Vec<String>,
    /// Preserve supported HTML formatting in transcript snippets.
    #[serde(default)]
    pub preserve_formatting: bool,
}

impl FetchTranscriptRequest {
    pub fn new<I, S>(
        video_id: impl Into<String>,
        languages: I,
        preserve_formatting: bool,
    ) -> Result<Self, TerminalError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let request = Self {
            video_id: video_id.into(),
            languages: languages.into_iter().map(Into::into).collect(),
            preserve_formatting,
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> Result<(), TerminalError> {
        VideoRequest {
            video_id: self.video_id.clone(),
        }
        .validate()?;

        if self.languages.is_empty() {
            return Err(TerminalError::new("languages must not be empty").with_code(400));
        }
        if self
            .languages
            .iter()
            .any(|language| language.trim().is_empty())
        {
            return Err(
                TerminalError::new("languages must not contain empty values").with_code(400),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FetchTranscriptResponse(
    #[schemars(with = "schema::FetchedTranscript")] pub FetchedTranscript,
);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct ListTranscriptsResponse(#[schemars(with = "schema::TranscriptList")] pub TranscriptList);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FetchVideoDetailsResponse(#[schemars(with = "schema::VideoDetails")] pub VideoDetails);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FetchMicroformatResponse(
    #[schemars(with = "schema::MicroformatData")] pub MicroformatData,
);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FetchStreamingDataResponse(
    #[schemars(with = "schema::StreamingData")] pub StreamingData,
);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FetchVideoInfosResponse(#[schemars(with = "schema::VideoInfos")] pub VideoInfos);

pub struct YouTubeTranscript {
    api: YouTubeTranscriptApi,
}

impl YouTubeTranscript {
    pub fn new(api: YouTubeTranscriptApi) -> Self {
        Self { api }
    }

    pub fn try_default() -> Result<Self, CookieError> {
        YouTubeTranscriptApi::new(None, None, None).map(Self::new)
    }
}

#[restate_sdk::service(name = "YouTubeTranscript")]
impl YouTubeTranscript {
    #[handler(name = "fetchTranscript")]
    async fn fetch_transcript(
        &self,
        ctx: Context<'_>,
        request: Json<FetchTranscriptRequest>,
    ) -> HandlerResult<Json<FetchTranscriptResponse>> {
        let request = request.into_inner();
        request.validate()?;
        let api = self.api.clone();

        let transcript = ctx
            .run(move || async move {
                let languages: Vec<_> = request.languages.iter().map(String::as_str).collect();
                api.fetch_transcript(
                    request.video_id.as_str(),
                    languages.as_slice(),
                    request.preserve_formatting,
                )
                .await
                .map(|transcript| Json(FetchTranscriptResponse(transcript)))
                .map_err(map_retrieval_error)
            })
            .name("fetch-youtube-transcript")
            .retry_policy(youtube_retry_policy())
            .await?;

        Ok(transcript)
    }

    #[handler(name = "listTranscripts")]
    async fn list_transcripts(
        &self,
        ctx: Context<'_>,
        request: Json<VideoRequest>,
    ) -> HandlerResult<Json<ListTranscriptsResponse>> {
        let request = request.into_inner();
        request.validate()?;
        let api = self.api.clone();

        let transcripts = ctx
            .run(move || async move {
                api.list_transcripts(request.video_id.as_str())
                    .await
                    .map(|transcripts| Json(ListTranscriptsResponse(transcripts)))
                    .map_err(map_retrieval_error)
            })
            .name("list-youtube-transcripts")
            .retry_policy(youtube_retry_policy())
            .await?;

        Ok(transcripts)
    }

    #[handler(name = "fetchVideoDetails")]
    async fn fetch_video_details(
        &self,
        ctx: Context<'_>,
        request: Json<VideoRequest>,
    ) -> HandlerResult<Json<FetchVideoDetailsResponse>> {
        let request = request.into_inner();
        request.validate()?;
        let api = self.api.clone();

        let details = ctx
            .run(move || async move {
                api.fetch_video_details(request.video_id.as_str())
                    .await
                    .map(|details| Json(FetchVideoDetailsResponse(details)))
                    .map_err(map_retrieval_error)
            })
            .name("fetch-youtube-video-details")
            .retry_policy(youtube_retry_policy())
            .await?;

        Ok(details)
    }

    #[handler(name = "fetchMicroformat")]
    async fn fetch_microformat(
        &self,
        ctx: Context<'_>,
        request: Json<VideoRequest>,
    ) -> HandlerResult<Json<FetchMicroformatResponse>> {
        let request = request.into_inner();
        request.validate()?;
        let api = self.api.clone();

        let microformat = ctx
            .run(move || async move {
                api.fetch_microformat(request.video_id.as_str())
                    .await
                    .map(|microformat| Json(FetchMicroformatResponse(microformat)))
                    .map_err(map_retrieval_error)
            })
            .name("fetch-youtube-microformat")
            .retry_policy(youtube_retry_policy())
            .await?;

        Ok(microformat)
    }

    #[handler(name = "fetchStreamingData")]
    async fn fetch_streaming_data(
        &self,
        ctx: Context<'_>,
        request: Json<VideoRequest>,
    ) -> HandlerResult<Json<FetchStreamingDataResponse>> {
        let request = request.into_inner();
        request.validate()?;
        let api = self.api.clone();

        let streaming_data = ctx
            .run(move || async move {
                api.fetch_streaming_data(request.video_id.as_str())
                    .await
                    .map(|data| Json(FetchStreamingDataResponse(data)))
                    .map_err(map_retrieval_error)
            })
            .name("fetch-youtube-streaming-data")
            .retry_policy(youtube_retry_policy())
            .await?;

        Ok(streaming_data)
    }

    #[handler(name = "fetchVideoInfos")]
    async fn fetch_video_infos(
        &self,
        ctx: Context<'_>,
        request: Json<VideoRequest>,
    ) -> HandlerResult<Json<FetchVideoInfosResponse>> {
        let request = request.into_inner();
        request.validate()?;
        let api = self.api.clone();

        let infos = ctx
            .run(move || async move {
                api.fetch_video_infos(request.video_id.as_str())
                    .await
                    .map(|infos| Json(FetchVideoInfosResponse(infos)))
                    .map_err(map_retrieval_error)
            })
            .name("fetch-youtube-video-infos")
            .retry_policy(youtube_retry_policy())
            .await?;

        Ok(infos)
    }
}

fn youtube_retry_policy() -> RunRetryPolicy {
    RunRetryPolicy::default()
        .initial_delay(Duration::from_secs(1))
        .exponentiation_factor(2.0)
        .max_delay(Duration::from_secs(30))
        .max_attempts(5)
}

pub(crate) fn map_retrieval_error(error: CouldNotRetrieveTranscript) -> HandlerError {
    let Some(code) = error.reason.as_ref().and_then(terminal_error_code) else {
        return error.into();
    };

    TerminalError::new(error.to_string()).with_code(code).into()
}

pub(crate) fn terminal_error_code(reason: &CouldNotRetrieveTranscriptReason) -> Option<u16> {
    match reason {
        CouldNotRetrieveTranscriptReason::InvalidVideoId => Some(400),
        CouldNotRetrieveTranscriptReason::AgeRestricted => Some(403),
        CouldNotRetrieveTranscriptReason::NoTranscriptFound { .. }
        | CouldNotRetrieveTranscriptReason::VideoUnavailable => Some(404),
        CouldNotRetrieveTranscriptReason::TranscriptsDisabled
        | CouldNotRetrieveTranscriptReason::VideoUnplayable { .. }
        | CouldNotRetrieveTranscriptReason::TranslationUnavailable(_)
        | CouldNotRetrieveTranscriptReason::TranslationLanguageUnavailable(_) => Some(422),
        CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie => Some(500),
        CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_) => Some(502),
        CouldNotRetrieveTranscriptReason::IpBlocked(_)
        | CouldNotRetrieveTranscriptReason::RequestBlocked(_)
        | CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(_) => None,
    }
}

#[allow(dead_code)]
mod schema {
    use std::collections::HashMap;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct TranslationLanguage {
        pub language: String,
        pub language_code: String,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct FetchedTranscriptSnippet {
        pub text: String,
        pub start: f64,
        pub duration: f64,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct FetchedTranscript {
        pub snippets: Vec<FetchedTranscriptSnippet>,
        pub video_id: String,
        pub language: String,
        pub language_code: String,
        pub is_generated: bool,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct Transcript {
        pub video_id: String,
        pub url: String,
        pub language: String,
        pub language_code: String,
        pub is_generated: bool,
        pub translation_languages: Vec<TranslationLanguage>,
        pub translation_languages_map: HashMap<String, String>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct TranscriptList {
        pub video_id: String,
        pub manually_created_transcripts: HashMap<String, Transcript>,
        pub generated_transcripts: HashMap<String, Transcript>,
        pub translation_languages: Vec<TranslationLanguage>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct VideoThumbnail {
        pub url: String,
        pub width: u32,
        pub height: u32,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct VideoDetails {
        pub video_id: String,
        pub title: String,
        pub length_seconds: u32,
        pub keywords: Option<Vec<String>>,
        pub channel_id: String,
        pub short_description: String,
        pub view_count: String,
        pub author: String,
        pub thumbnails: Vec<VideoThumbnail>,
        pub is_live_content: bool,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct MicroformatEmbed {
        pub height: Option<i32>,
        pub iframe_url: Option<String>,
        pub width: Option<i32>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct MicroformatThumbnail {
        pub thumbnails: Option<Vec<VideoThumbnail>>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct MicroformatData {
        pub available_countries: Option<Vec<String>>,
        pub category: Option<String>,
        pub description: Option<String>,
        pub embed: Option<MicroformatEmbed>,
        pub external_channel_id: Option<String>,
        pub external_video_id: Option<String>,
        pub has_ypc_metadata: Option<bool>,
        pub is_family_safe: Option<bool>,
        pub is_shorts_eligible: Option<bool>,
        pub is_unlisted: Option<bool>,
        pub length_seconds: Option<String>,
        pub like_count: Option<String>,
        pub owner_channel_name: Option<String>,
        pub owner_profile_url: Option<String>,
        pub publish_date: Option<String>,
        pub thumbnail: Option<MicroformatThumbnail>,
        pub title: Option<String>,
        pub upload_date: Option<String>,
        pub view_count: Option<String>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct Range {
        pub start: String,
        pub end: String,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct ColorInfo {
        pub primaries: Option<String>,
        pub transfer_characteristics: Option<String>,
        pub matrix_coefficients: Option<String>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct StreamingFormat {
        pub itag: u32,
        pub url: Option<String>,
        pub mime_type: String,
        pub bitrate: u64,
        pub width: Option<u32>,
        pub height: Option<u32>,
        pub init_range: Option<Range>,
        pub index_range: Option<Range>,
        pub last_modified: Option<String>,
        pub content_length: Option<String>,
        pub quality: String,
        pub fps: Option<u32>,
        pub quality_label: Option<String>,
        pub projection_type: String,
        pub average_bitrate: Option<u64>,
        pub audio_quality: Option<String>,
        pub approx_duration_ms: String,
        pub audio_sample_rate: Option<String>,
        pub audio_channels: Option<u32>,
        pub quality_ordinal: Option<String>,
        pub high_replication: Option<bool>,
        pub color_info: Option<ColorInfo>,
        pub loudness_db: Option<f64>,
        pub is_drc: Option<bool>,
        pub xtags: Option<String>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct StreamingData {
        pub expires_in_seconds: String,
        pub formats: Vec<StreamingFormat>,
        pub adaptive_formats: Vec<StreamingFormat>,
        pub server_abr_streaming_url: Option<String>,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    pub struct VideoInfos {
        pub video_details: VideoDetails,
        pub microformat: MicroformatData,
        pub streaming_data: StreamingData,
        pub transcript_list: TranscriptList,
    }
}
