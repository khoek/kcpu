#include <exception>
#include "SDL.h"

#include "sdl2_renderer.hpp"

sdl2_runtime::sdl2_runtime() {
    SDL_Init(SDL_INIT_VIDEO);
}

sdl2_runtime::~sdl2_runtime() {
    SDL_Quit();
}

sdl2_runtime sdl2_renderer::get_runtime() {
    return sdl2_runtime();
}

sdl2_renderer::sdl2_renderer(unsigned int width, unsigned int height) : renderer(width, height) {
    get_runtime();

    thread = std::thread(&sdl2_renderer::render_loop, this);

    std::unique_lock<std::mutex> l(mutex);
    cv.wait(l, [this]{ return startup_complete; });
}

sdl2_renderer::~sdl2_renderer() {
    running = false;
    thread.join();
}

void sdl2_renderer::render_loop() {
    SDL_CreateWindowAndRenderer(get_width(), get_height(), SDL_WINDOW_OPENGL, &window, &rend);

    if(!window) {
        fprintf(stderr, "Could not create window: %s\n", SDL_GetError());
        exit(1);
    }

    // Note that if SDL_PIXELFORMAT_ARGB8888 does not match the format used on the graphics card,
    // then the texture must be converted, which incurs a great penalty.
    texture = SDL_CreateTexture(rend, SDL_PIXELFORMAT_ARGB8888, SDL_TEXTUREACCESS_STREAMING, get_width(), get_height());

    int pitch;
    SDL_LockTexture(texture, NULL, (void **) &buffer, &pitch);

    {
        std::lock_guard<std::mutex> l(mutex);
        startup_complete = true;
    }
    cv.notify_all();

    while(running) {
        SDL_Delay(1000 / 50);

        {
            std::lock_guard<std::mutex> l(mutex);
            if(do_flip) {
                SDL_UnlockTexture(texture);

                SDL_RenderCopy(rend, texture, NULL, NULL);
                SDL_RenderPresent(rend);

                SDL_LockTexture(texture, NULL, (void **) &buffer, &pitch);

                do_flip = false;
                cv.notify_all();
            }
        }

        SDL_Event e;
        while(SDL_PollEvent(&e)){
            if (e.type == SDL_QUIT) {
                running = false;
                break;
            }
        }
    }

    // TODO just gracefully provide a dummy framebuffer instead?
    throw new std::runtime_error("graphics window closed");

    buffer = NULL;
    cv.notify_all();

    SDL_UnlockTexture(texture);

    SDL_DestroyTexture(texture);
    SDL_DestroyRenderer(rend);
    SDL_DestroyWindow(window);
}

char * sdl2_renderer::get_next_framebuffer() {
    return buffer;
}

void sdl2_renderer::flip() {
    if(!running) {
        return;
    }

    {
        std::lock_guard<std::mutex> l(mutex);
        do_flip = true;
    }

    std::unique_lock<std::mutex> l(mutex);
    cv.wait(l, [this]{ return !running || !do_flip; });
}