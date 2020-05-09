#include "framebuffer.hpp"

framebuffer::framebuffer(unsigned int width, unsigned int height) : width(width), height(height), active_buffer(0) {
    buffer[0] = new char[width * height * 4];
    buffer[1] = new char[width * height * 4];
}

framebuffer::~framebuffer() {
    delete buffer[0];
    delete buffer[1];
}

unsigned int framebuffer::get_width() {
    return width;
}

unsigned int framebuffer::get_height() {
    return height;
}

static int advance_buffer_index(int active) {
    return (active + 1) % framebuffer::BUFFER_COUNT;
}

char * framebuffer::get_fb_active() {
    return buffer[active_buffer];
}

char * framebuffer::get_fb_next() {
    return buffer[advance_buffer_index(active_buffer)];
}

void framebuffer::advance() {
    std::lock_guard l(lock);
    active_buffer = advance_buffer_index(active_buffer);
}

std::mutex & framebuffer::get_lock() {
    return lock;
}