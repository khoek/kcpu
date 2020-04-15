#ifndef LIB_GRAPHICS_SDL2_RENDERER_H
#define LIB_GRAPHICS_SDL2_RENDERER_H

#ifdef ENABLE_SDL_GRAPHICS

#include <thread>
#include <mutex>
#include <condition_variable>
#include "SDL.h"

#include "renderer.hpp"
#include "framebuffer.hpp"

class sdl2_runtime {
    public:
    static sdl2_runtime & get_runtime();

    sdl2_runtime();
    ~sdl2_runtime();
};

class sdl2_renderer : public renderer {
    private:
    SDL_Window *window;
    SDL_Renderer *rend;
    SDL_Texture *texture;

    std::mutex mutex;
    std::condition_variable cv;
    std::thread thread;

    volatile bool running = true;
    bool startup_complete = false;
    bool destroyed = false;
    char *buffer;

    void render_loop();

    public:
    sdl2_renderer(unsigned int width, unsigned int height);
    ~sdl2_renderer();
};

#endif

#endif