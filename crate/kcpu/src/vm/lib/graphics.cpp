#include "graphics.hpp"

#ifdef ENABLE_SDL_GRAPHICS
#include "graphics/sdl2_renderer.hpp"
#endif

graphics & graphics::get_graphics() {
    static graphics g;
    return g;
}

void graphics::configure(bool headless) {
    this->headless = headless;
}

renderer * graphics::new_renderer(unsigned int width, unsigned int height) {
#ifdef ENABLE_SDL_GRAPHICS
    if(!headless) {
        return new sdl2_renderer(width, height);
    }
#endif

    return new headless_renderer(width, height);
}