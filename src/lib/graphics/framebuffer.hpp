#ifndef LIB_GRAPHICS_FRAMEBUFFER_H
#define LIB_GRAPHICS_FRAMEBUFFER_H

#include <mutex>

class framebuffer {
    public:
    static const int BUFFER_COUNT = 2;

    private:
    unsigned int width;
    unsigned int height;

    int active_buffer;
    char * buffer[BUFFER_COUNT];

    std::mutex lock;

    public:
    framebuffer(unsigned int width, unsigned int height);
    virtual ~framebuffer();
    unsigned int get_width();
    unsigned int get_height();
    char * get_fb_active();
    char * get_fb_next();
    std::mutex & get_lock();
    void advance();
};

#endif