/**
 * C-facing shim for CREngine lifecycle, layout, and rendering surfaces.
 *
 * This header intentionally exposes a limited, stable surface for the Rust bindings layer. All
 * types are opaque to keep the ABI steady even while the underlying implementation evolves.
 */
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Result codes returned by shim calls. */
typedef enum CreResult {
    CRE_RESULT_OK = 0,
    CRE_RESULT_UNSUPPORTED = 1,
    CRE_RESULT_INVALID_ARGUMENT = 2,
    CRE_RESULT_INTERNAL_ERROR = 3,
} CreResult;

/** Pixel formats supported by the rendering surface. */
typedef enum CreSurfaceFormat {
    CRE_SURFACE_FORMAT_INVALID = 0,
    CRE_SURFACE_FORMAT_GRAY8 = 1,
    CRE_SURFACE_FORMAT_MONOCHROME = 2,
} CreSurfaceFormat;

/** Opaque document handle for lifecycle and rendering operations. */
typedef struct CreDocument CreDocument;

/** Basic width/height pair used throughout the shim. */
typedef struct CreSize {
    uint32_t width;
    uint32_t height;
} CreSize;

/** Rendering buffer descriptor provided by the caller. */
typedef struct CreRenderSurface {
    uint8_t *data;
    uint32_t stride;
    CreSize size;
    CreSurfaceFormat format;
} CreRenderSurface;

/** Layout preferences that inform pagination. */
typedef struct CreLayoutConfig {
    uint32_t font_size;
    uint32_t line_height_percent;
    uint32_t page_margin_dp;
} CreLayoutConfig;

/**
 * Opens a document from an on-disk EPUB. The returned handle must be released with
 * cre_close_document when no longer needed.
 */
CreDocument *cre_open_document(const char *path, CreResult *out_status);

/** Releases all resources associated with a document handle. */
void cre_close_document(CreDocument *doc);

/** Reports the number of pages produced by the last layout run. */
CreResult cre_page_count(const CreDocument *doc, uint32_t *out_pages);

/** Applies layout to the document using the provided preferences. */
CreResult cre_layout_document(CreDocument *doc, const CreLayoutConfig *config);

/**
 * Renders a page into the caller-supplied surface buffer. The buffer must be large enough to hold
 * size.height rows, each at least stride bytes wide.
 */
CreResult cre_render_page(const CreDocument *doc, uint32_t page_index, CreRenderSurface *surface);

#ifdef __cplusplus
} /* extern "C" */
#endif
