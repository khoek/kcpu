#include <cstdlib>
#include "dummy_renderer.hpp"

dummy_renderer::dummy_renderer(unsigned int width, unsigned int height) : renderer(width, height) {
    next_buffer = 0;
    buffer[0] = (char *) malloc(width * height * 4);
    buffer[1] = (char *) malloc(width * height * 4);
}

dummy_renderer::~dummy_renderer() {
    free(buffer[0]);
    free(buffer[1]);
}

char * dummy_renderer::get_next_framebuffer() {
    return buffer[next_buffer];
}

void dummy_renderer::flip() {
    next_buffer = next_buffer ? 0 : 1;
}