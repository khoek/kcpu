#ifndef LIB_GRAPHICS_RENDERER_H
#define LIB_GRAPHICS_RENDERER_H

class renderer {
    private:
    unsigned int width;
    unsigned int height;

    public:
    renderer(unsigned int width, unsigned int height);
    virtual ~renderer();
    unsigned int get_width();
    unsigned int get_height();

    virtual char * get_next_framebuffer() = 0;
    virtual void flip() = 0;
};

#endif