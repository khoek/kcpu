#ifndef LIB_RENDERER_RENDERER_H
#define LIB_RENDERER_RENDERER_H

class renderer {
    private:
    unsigned int width;
    unsigned int height;

    int next_buffer;
    char * buffer[2];

    public:
    renderer(unsigned int width, unsigned int height);
    virtual ~renderer();
    unsigned int get_width();
    unsigned int get_height();
    char * get_next_framebuffer();
    void flip();

    virtual void publish_next_framebuffer() = 0;
};

#endif