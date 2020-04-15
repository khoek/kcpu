#include <exception>

#include "sdl2_renderer.hpp"

#ifdef ENABLE_SDL_GRAPHICS

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
    SDL_CreateWindowAndRenderer(get_fb().get_width(), get_fb().get_height(), SDL_WINDOW_OPENGL, &window, &rend);

    if(!window) {
        std::lock_guard<std::mutex> l(mutex);
        running = false;
        startup_complete = true;
        cv.notify_all();
        return;
    }

    // Note that if SDL_PIXELFORMAT_ARGB8888 does not match the format used on the graphics card,
    // then the texture must be converted, which incurs a great penalty.
    texture = SDL_CreateTexture(rend, SDL_PIXELFORMAT_ARGB8888, SDL_TEXTUREACCESS_STREAMING, get_fb().get_width(), get_fb().get_height());

    int pitch;
    SDL_LockTexture(texture, NULL, (void **) &buffer, &pitch);

    {
        std::lock_guard<std::mutex> l(mutex);
        startup_complete = true;
        cv.notify_all();
    }

    while(running) {
        SDL_Delay(1000 / 50);

        {
            {
                std::lock_guard(get_fb().get_lock());
                memcpy(buffer, get_fb().get_fb_active(), 4 * get_fb().get_width() * get_fb().get_height());
            }

            SDL_UnlockTexture(texture);

            SDL_RenderCopy(rend, texture, NULL, NULL);
            SDL_RenderPresent(rend);

            SDL_LockTexture(texture, NULL, (void **) &buffer, &pitch);
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
        // NOTE: We don't do anything if the user closes the graphics window while the display is running.
        // throw new std::runtime_error("graphics window closed");
    }

    buffer = NULL;

    SDL_UnlockTexture(texture);

    SDL_DestroyTexture(texture);
    SDL_DestroyRenderer(rend);
    SDL_DestroyWindow(window);
}

#endif