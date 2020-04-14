#ifdef ENABLE_SDL_GRAPHICS

#include <exception>

#include "sdl2_renderer.hpp"

sdl2_runtime & sdl2_runtime::get_runtime() {
    static sdl2_runtime r;
    return r;
}

sdl2_runtime::sdl2_runtime() {
    SDL_Init(SDL_INIT_VIDEO);
}

sdl2_runtime::~sdl2_runtime() {
    SDL_Quit();
}

sdl2_renderer::sdl2_renderer(unsigned int width, unsigned int height) : renderer(width, height) {
    sdl2_runtime::get_runtime();

    thread = std::thread(&sdl2_renderer::render_loop, this);

    std::unique_lock<std::mutex> l(mutex);
    cv.wait(l, [this]{ return startup_complete; });
}

sdl2_renderer::~sdl2_renderer() {
    destroyed = true;
    running = false;
    thread.join();
}

void sdl2_renderer::render_loop() {
    SDL_CreateWindowAndRenderer(get_width(), get_height(), SDL_WINDOW_OPENGL, &window, &rend);

    if(!window) {
        std::lock_guard<std::mutex> l(mutex);
        running = false;
        startup_complete = true;
        cv.notify_all();
        return;
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

    if(!destroyed) {
        // TODO just gracefully provide a dummy framebuffer instead?
        throw new std::runtime_error("graphics window closed");
    }

    buffer = NULL;
    cv.notify_all();

    SDL_UnlockTexture(texture);

    SDL_DestroyTexture(texture);
    SDL_DestroyRenderer(rend);
    SDL_DestroyWindow(window);
}

void sdl2_renderer::publish_next_framebuffer() {
    if(!running) {
        return;
    }

    memcpy(buffer, get_next_framebuffer(), 4 * get_width() * get_height());

    {
        std::lock_guard<std::mutex> l(mutex);
        do_flip = true;
    }

    std::unique_lock<std::mutex> l(mutex);
    cv.wait(l, [this]{ return !running || !do_flip; });
}

#endif