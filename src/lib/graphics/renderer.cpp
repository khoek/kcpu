#include "renderer.hpp"

renderer::renderer(unsigned int width, unsigned int height) : width(width), height(height) {
}

renderer::~renderer() {
}

unsigned int renderer::get_width() {
    return width;
}

unsigned int renderer::get_height() {
    return height;
}