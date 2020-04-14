#include "renderer.hpp"

renderer::renderer(unsigned int width, unsigned int height) : width(width), height(height) {
    next_buffer = 0;
    buffer[0] = new char[width * height * 4];
    buffer[1] = new char[width * height * 4];
}

renderer::~renderer() {
    delete buffer[0];
    delete buffer[1];
}

unsigned int renderer::get_width() {
    return width;
}

unsigned int renderer::get_height() {
    return height;
}

char * renderer::get_next_framebuffer() {
    return buffer[next_buffer];
}

void renderer::flip() {
    publish_next_framebuffer();
    next_buffer = next_buffer ? 0 : 1;
}