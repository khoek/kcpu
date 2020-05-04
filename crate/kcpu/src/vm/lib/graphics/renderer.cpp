#include "renderer.hpp"

renderer::renderer(unsigned int width, unsigned int height) : fb(framebuffer(width, height)) {
}

renderer::~renderer() {
}

framebuffer & renderer::get_fb() {
    return fb;
}

headless_renderer::headless_renderer(unsigned int width, unsigned int height) : renderer(width, height) {
}

headless_renderer::~headless_renderer() {
}