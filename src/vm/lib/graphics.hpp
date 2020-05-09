#ifndef LIB_GRAPHICS_H
#define LIB_GRAPHICS_H

#include "graphics/renderer.hpp"

class graphics {
    private:
    bool headless = false;

    public:
    static graphics & get_graphics();

    void configure(bool headless);
    renderer * new_renderer(unsigned int width, unsigned int height);
};

#endif