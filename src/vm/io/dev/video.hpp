#ifndef VM_MOD_IODEV_VIDEO_H
#define VM_MOD_IODEV_VIDEO_H

#include "iodev.hpp"
#include "../../../lib/graphics.hpp"

namespace kcpu {

namespace iodev {

class video : public io_device {
    private:
    static const unsigned int PIXEL_WIDTH = 8;
    static const unsigned int WIDTH = 160;
    static const unsigned int HEIGHT = 120;

    static const unsigned int PORT_BASE = 0xC0;

    static const unsigned int REG_CMD    = 0;
    static const unsigned int REG_STREAM = 1;
    static const unsigned int REG_HIADDR = 2;
    static const unsigned int REG_LOADDR = 3;
    static const unsigned int REG_DATA   = 4;
    static const unsigned int REGISTER_COUNT = 5;

    static const unsigned int CMD_FLIP = 0x01;
    static const unsigned int CMD_STREAMRST = 0x02;

    struct vram_addr {
        unsigned int r;
        unsigned int c;
        unsigned int comp;
    };

    renderer *rend;

    unsigned int stream_ptr;
    RegVal hiaddr = 0;
    RegVal loaddr = 0;

    constexpr unsigned int get_framebuffer_size();
    unsigned int get_addr();
    video::vram_addr decode_addr(unsigned int addr);
    RegVal vram_read(unsigned int addr);
    void vram_write(unsigned int addr, RegVal val);
    void handle_command(RegVal cmd);

    public:
    video();
    ~video();
    std::vector<RegVal> get_reserved_ports() override;
    std::pair<RegVal, halfcycle_count_t> read(RegVal port) override;
    halfcycle_count_t write(RegVal port, RegVal val) override;
};

}

}

#endif