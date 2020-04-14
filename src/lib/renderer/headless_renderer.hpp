#ifndef LIB_RENDERER_HEADLESS_RENDERER_H
#define LIB_RENDERER_HEADLESS_RENDERER_H

#include "renderer.hpp"

class headless_renderer : public renderer {
    public:
    headless_renderer(unsigned int width, unsigned int height);
    void publish_next_framebuffer();
};

#endif