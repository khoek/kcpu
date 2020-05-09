#ifndef LIB_GRAPHICS_RENDERER_H
#define LIB_GRAPHICS_RENDERER_H

#include "framebuffer.hpp"

class renderer {
    private:
    framebuffer fb;

    public:
    renderer(unsigned int width, unsigned int height);
    virtual ~renderer() = 0;
    framebuffer & get_fb();
};

class headless_renderer : public renderer {
    public:
    headless_renderer(unsigned int width, unsigned int height);
    ~headless_renderer();
};

#endif