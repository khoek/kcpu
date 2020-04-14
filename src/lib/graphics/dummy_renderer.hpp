#ifndef LIB_GRAPHICS_DUMMY_RENDERER_H
#define LIB_GRAPHICS_DUMMY_RENDERER_H

#include "renderer.hpp"

class dummy_renderer : public renderer {
    private:
    int next_buffer;
    char * buffer[2];

    public:
    dummy_renderer(unsigned int width, unsigned int height);
    virtual ~dummy_renderer();

    char * get_next_framebuffer();
    void flip();
};

#endif