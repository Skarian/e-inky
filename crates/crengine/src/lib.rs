#![deny(missing_docs)]

//! Safe wrappers around the CREngine-NG C shim.
//!
//! The upstream engine is not thread-safe; all handles in this module are pinned to the thread
//! where they were created. Each handle uses a phantom `Rc` to opt out of `Send`/`Sync` and
//! records the originating `ThreadId` to enforce same-thread use at runtime.

use std::ffi::CString;
use std::io::Write;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use std::thread::{self, ThreadId};

use thiserror::Error;

pub mod raw;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!("The CREngine shim is only built on desktop targets (linux, macOS, windows).");

/// CREngine target canvas for the X4 device.
pub const TARGET_SIZE: Size = Size {
    width: 480,
    height: 800,
};

/// Errors surfaced by the safe wrapper layer.
#[derive(Debug, Error)]
pub enum Error {
    /// The platform is unsupported (bindings not generated).
    #[error("CREngine shim is unavailable on this target")]
    UnsupportedTarget,
    /// A null pointer was returned from the underlying shim.
    #[error("CREngine returned a null handle")]
    NullHandle,
    /// A handle was used from the wrong thread.
    #[error("CREngine handles must be used on the thread where they were created")]
    WrongThread,
    /// The shim reported an invalid argument.
    #[error("CREngine rejected the call due to an invalid argument")]
    InvalidArgument,
    /// The shim reported an unsupported operation.
    #[error("CREngine reported that the operation is unsupported")]
    Unsupported,
    /// The shim encountered an internal error.
    #[error("CREngine reported an internal error")]
    InternalError,
    /// A page index was outside the available range.
    #[error("Page {index} is out of bounds for a document with {total} pages")]
    PageOutOfBounds { index: u32, total: u32 },
    /// The caller supplied a surface buffer that was too small.
    #[error("Surface buffer too small: expected at least {expected} bytes, got {actual}")]
    SurfaceTooSmall { expected: usize, actual: usize },
    /// A text encoding or FFI conversion failed.
    #[error("FFI string conversion failed: {0}")]
    Ffi(String),
    /// I/O failure while preparing input for the engine.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Result alias for wrapper operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Logical size helper used for layout and surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Supported pixel formats for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormat {
    /// 8-bit linear grayscale.
    Gray8,
    /// 1-bit monochrome.
    Monochrome,
}

impl SurfaceFormat {
    fn as_raw(self) -> raw::CreSurfaceFormat {
        match self {
            SurfaceFormat::Gray8 => raw::CRE_SURFACE_FORMAT_GRAY8,
            SurfaceFormat::Monochrome => raw::CRE_SURFACE_FORMAT_MONOCHROME,
        }
    }
}

/// Layout preferences passed to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutConfig {
    /// Font size in device-independent pixels.
    pub font_size: u32,
    /// Line height multiplier as a percentage (100 = normal).
    pub line_height_percent: u32,
    /// Margin applied around the page in device-independent pixels.
    pub page_margin_dp: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            font_size: 18,
            line_height_percent: 120,
            page_margin_dp: 12,
        }
    }
}

impl From<LayoutConfig> for raw::CreLayoutConfig {
    fn from(value: LayoutConfig) -> Self {
        Self {
            font_size: value.font_size,
            line_height_percent: value.line_height_percent,
            page_margin_dp: value.page_margin_dp,
        }
    }
}

/// A rendering surface backed by an owned buffer.
#[derive(Debug)]
pub struct Canvas {
    buffer: Vec<u8>,
    size: Size,
    stride: usize,
    format: SurfaceFormat,
}

impl Canvas {
    /// Creates a grayscale canvas sized to the X4's display resolution.
    pub fn gray8_target() -> Self {
        Self::new_gray8(TARGET_SIZE)
    }

    /// Creates a grayscale canvas with the provided dimensions.
    pub fn new_gray8(size: Size) -> Self {
        let stride = size.width as usize;
        let len = stride * size.height as usize;
        Self {
            buffer: vec![0; len],
            size,
            stride,
            format: SurfaceFormat::Gray8,
        }
    }

    /// Raw byte view of the canvas.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Mutable raw byte view of the canvas.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn to_surface(&mut self) -> raw::CreRenderSurface {
        raw::CreRenderSurface {
            data: self.buffer.as_mut_ptr(),
            stride: self.stride as u32,
            size: raw::CreSize {
                width: self.size.width,
                height: self.size.height,
            },
            format: self.format.as_raw(),
        }
    }
}

/// Basic document metadata tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    /// Node title.
    pub title: String,
    /// Optional page number for the entry.
    pub page: Option<u32>,
    /// Child entries.
    pub children: Vec<TocEntry>,
}

/// Global engine lifetime token.
#[derive(Debug)]
pub struct Engine {
    thread: ThreadId,
    _no_send_sync: PhantomData<Rc<()>>,
}

impl Engine {
    /// Initializes the engine on the current thread.
    pub fn initialize() -> Result<Self> {
        Ok(Self {
            thread: thread::current().id(),
            _no_send_sync: PhantomData,
        })
    }

    /// Shuts down the engine explicitly.
    pub fn shutdown(self) {
        drop(self);
    }

    /// Loads an EPUB from in-memory bytes.
    pub fn load_epub_from_bytes(&self, bytes: impl AsRef<[u8]>) -> Result<Document> {
        self.ensure_thread()?;
        Document::open_from_bytes(self.handle(), bytes.as_ref(), "epub")
    }

    /// Loads an HTML document from in-memory bytes.
    pub fn load_html_from_bytes(&self, bytes: impl AsRef<[u8]>) -> Result<Document> {
        self.ensure_thread()?;
        Document::open_from_bytes(self.handle(), bytes.as_ref(), "html")
    }

    fn ensure_thread(&self) -> Result<()> {
        if self.thread == thread::current().id() {
            Ok(())
        } else {
            Err(Error::WrongThread)
        }
    }

    fn handle(&self) -> EngineHandle {
        EngineHandle {
            thread: self.thread,
            _no_send_sync: PhantomData,
        }
    }
}

/// Borrowed engine handle used to enforce thread affinity.
#[derive(Debug, Clone)]
struct EngineHandle {
    thread: ThreadId,
    _no_send_sync: PhantomData<Rc<()>>,
}

impl EngineHandle {
    fn ensure_thread(&self) -> Result<()> {
        if self.thread == thread::current().id() {
            Ok(())
        } else {
            Err(Error::WrongThread)
        }
    }
}

/// Managed document handle.
#[derive(Debug)]
pub struct Document {
    raw: NonNull<raw::CreDocument>,
    engine: EngineHandle,
    storage: DocumentStorage,
    pages: u32,
}

impl Document {
    fn open_from_bytes(engine: EngineHandle, bytes: &[u8], suffix: &str) -> Result<Self> {
        engine.ensure_thread()?;

        let mut temp = tempfile::Builder::new()
            .prefix("cre-document-")
            .suffix(&format!(".{suffix}"))
            .tempfile()?;
        Write::write_all(&mut temp, bytes)?;

        let c_path = CString::new(
            temp.path()
                .to_str()
                .ok_or_else(|| Error::Ffi("temp file path contained invalid UTF-8".into()))?,
        )
        .map_err(|e| Error::Ffi(e.to_string()))?;

        let mut status = raw::CRE_RESULT_OK;
        let raw = unsafe { raw::cre_open_document(c_path.as_ptr(), &mut status) };
        map_status(status)?;
        let raw = NonNull::new(raw).ok_or(Error::NullHandle)?;

        Ok(Self {
            raw,
            engine,
            storage: DocumentStorage::Temp(temp),
            pages: 0,
        })
    }

    /// Applies pagination with the provided configuration.
    pub fn layout(&mut self, config: LayoutConfig) -> Result<u32> {
        self.engine.ensure_thread()?;
        let raw_config: raw::CreLayoutConfig = config.into();
        let status = unsafe { raw::cre_layout_document(self.raw.as_ptr(), &raw_config) };
        map_status(status)?;
        self.pages = self.page_count()?;
        Ok(self.pages)
    }

    /// Returns the number of pages produced by the most recent layout.
    pub fn page_count(&self) -> Result<u32> {
        self.engine.ensure_thread()?;
        let mut pages = 0;
        let status = unsafe { raw::cre_page_count(self.raw.as_ptr(), &mut pages) };
        map_status(status)?;
        Ok(pages)
    }

    /// Renders a page into the provided canvas.
    pub fn render_page(&self, page_index: u32, canvas: &mut Canvas) -> Result<()> {
        self.engine.ensure_thread()?;
        let total = self.pages;
        if page_index >= total {
            return Err(Error::PageOutOfBounds {
                index: page_index,
                total,
            });
        }

        let expected = canvas.stride * canvas.size.height as usize;
        let actual = canvas.buffer.len();
        if actual < expected {
            return Err(Error::SurfaceTooSmall { expected, actual });
        }

        let mut surface = canvas.to_surface();
        let status = unsafe { raw::cre_render_page(self.raw.as_ptr(), page_index, &mut surface) };
        map_status(status)
    }

    /// Returns a lightweight page handle for convenience APIs.
    pub fn page<'a>(&'a self, index: u32) -> Result<Page<'a>> {
        self.engine.ensure_thread()?;
        let total = self.pages;
        if index >= total {
            return Err(Error::PageOutOfBounds { index, total });
        }
        Ok(Page {
            document: self,
            index,
        })
    }

    /// Placeholder for Table of Contents extraction.
    pub fn toc(&self) -> Result<Vec<TocEntry>> {
        self.engine.ensure_thread()?;
        Err(Error::Unsupported)
    }

    /// Placeholder for document text extraction.
    pub fn extract_text(&self) -> Result<String> {
        self.engine.ensure_thread()?;
        Err(Error::Unsupported)
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        unsafe {
            raw::cre_close_document(self.raw.as_ptr());
        }
    }
}

#[derive(Debug)]
enum DocumentStorage {
    Temp(tempfile::NamedTempFile),
}

/// View of a single page tied to the parent document's lifetime.
#[derive(Debug)]
pub struct Page<'a> {
    document: &'a Document,
    index: u32,
}

impl<'a> Page<'a> {
    /// Index of the page within the document.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Renders the page into the provided canvas.
    pub fn render(&self, canvas: &mut Canvas) -> Result<()> {
        self.document.render_page(self.index, canvas)
    }
}

fn map_status(status: raw::CreResult) -> Result<()> {
    match status {
        raw::CRE_RESULT_OK => Ok(()),
        raw::CRE_RESULT_UNSUPPORTED => Err(Error::Unsupported),
        raw::CRE_RESULT_INVALID_ARGUMENT => Err(Error::InvalidArgument),
        raw::CRE_RESULT_INTERNAL_ERROR => Err(Error::InternalError),
        _ => Err(Error::InternalError),
    }
}

/// Anchor symbol exported by the Rust side of the crate to ensure the cdylib/staticlib has at least
/// one well-known symbol.
#[no_mangle]
pub extern "C" fn crengine_link_anchor() {}
