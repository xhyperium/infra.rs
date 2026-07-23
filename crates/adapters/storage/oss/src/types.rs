use bytes::Bytes;
use kernel::error::XResult;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey(String);
impl ObjectKey {
    pub fn new(k: impl Into<String>) -> XResult<Self> {
        let k = k.into();
        let t = k.trim().trim_start_matches('/');
        if t.is_empty() || t.contains("..") || t.len() > 1023 || t.chars().any(|c| c.is_control()) {
            return Err(kernel::error::XError::invalid("invalid key"));
        }
        Ok(Self(t.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectMeta {
    pub size: u64,
    pub etag: Option<String>,
    pub version_id: Option<String>,
    pub checksum: Option<String>,
    pub content_type: Option<String>,
}
impl ObjectMeta {
    pub fn with_size(s: u64) -> Self {
        Self { size: s, etag: None, version_id: None, checksum: None, content_type: None }
    }
}

pub type ByteStream = Pin<Box<dyn ByteStreamTrait + Send>>;
pub trait ByteStreamTrait {
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<XResult<Bytes>>>;
}
impl<S: futures_core::Stream<Item = XResult<Bytes>> + Send + 'static> ByteStreamTrait for S {
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<XResult<Bytes>>> {
        futures_core::Stream::poll_next(self, cx)
    }
}
impl futures_core::Stream for dyn ByteStreamTrait + Send + '_ {
    type Item = XResult<Bytes>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        ByteStreamTrait::poll_next(self, cx)
    }
}

#[derive(Clone, Debug, Default)]
pub struct UploadOptions {
    pub content_type: Option<String>,
    pub metadata: Option<Vec<(String, String)>>,
    pub part_size: usize,
    pub sse_enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadOptions {
    pub range: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
    pub version_id: Option<String>,
}
impl DownloadOptions {
    pub fn with_range(r: impl Into<String>) -> Self {
        Self { range: Some(r.into()), ..Default::default() }
    }
}

pub fn byte_stream_from_bytes(data: Bytes) -> ByteStream {
    Box::pin(futures_util::stream::once(async move { Ok(data) }))
}
