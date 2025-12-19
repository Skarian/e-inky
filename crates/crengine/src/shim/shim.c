#include "shim.h"

#include <stddef.h>
#include <stdlib.h>

struct CreDocument {
    uint32_t pages;
};

static CreResult validate_surface(const CreRenderSurface *surface) {
    if (surface == NULL) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    if (surface->data == NULL || surface->stride == 0) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    if (surface->size.width == 0 || surface->size.height == 0) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    if (surface->format == CRE_SURFACE_FORMAT_INVALID) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    return CRE_RESULT_OK;
}

CreDocument *cre_open_document(const char *path, CreResult *out_status) {
    if (path == NULL) {
        if (out_status != NULL) {
            *out_status = CRE_RESULT_INVALID_ARGUMENT;
        }
        return NULL;
    }

    CreDocument *doc = (CreDocument *)malloc(sizeof(CreDocument));
    if (doc == NULL) {
        if (out_status != NULL) {
            *out_status = CRE_RESULT_INTERNAL_ERROR;
        }
        return NULL;
    }

    doc->pages = 0;
    if (out_status != NULL) {
        *out_status = CRE_RESULT_OK;
    }
    return doc;
}

void cre_close_document(CreDocument *doc) {
    if (doc == NULL) {
        return;
    }
    free(doc);
}

CreResult cre_page_count(const CreDocument *doc, uint32_t *out_pages) {
    if (doc == NULL || out_pages == NULL) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    *out_pages = doc->pages;
    return CRE_RESULT_OK;
}

CreResult cre_layout_document(CreDocument *doc, const CreLayoutConfig *config) {
    if (doc == NULL || config == NULL) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    // Stub layout: pretend a single page exists until real pagination is wired.
    doc->pages = 1;
    (void)config;
    return CRE_RESULT_OK;
}

CreResult cre_render_page(const CreDocument *doc, uint32_t page_index, CreRenderSurface *surface) {
    if (doc == NULL) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }
    if (page_index >= doc->pages) {
        return CRE_RESULT_INVALID_ARGUMENT;
    }

    CreResult surface_status = validate_surface(surface);
    if (surface_status != CRE_RESULT_OK) {
        return surface_status;
    }

    // Stub renderer: fills the buffer with a simple pattern to prove wiring works.
    uint32_t rows = surface->size.height;
    uint32_t stride = surface->stride;
    for (uint32_t y = 0; y < rows; y++) {
        uint8_t value = (uint8_t)((page_index + y) & 0xFF);
        for (uint32_t x = 0; x < stride; x++) {
            surface->data[y * stride + x] = value;
        }
    }

    return CRE_RESULT_OK;
}
