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
    static const unsigned int REG_ADDR   = 2;
    static const unsigned int REG_DATA   = 3;
    static const unsigned int REGISTER_COUNT = 4;

    static const unsigned int CMD_FLIP = 0x01;

    char *buffer[2];

    renderer *rend;

    regval_t addr;

    void handle_command(regval_t cmd);

    public:
    video();
    ~video();
    std::vector<regval_t> get_reserved_ports();
    std::pair<regval_t, halfcycle_count_t> read(regval_t port);
    halfcycle_count_t write(regval_t port, regval_t val);
};

}

}

#endif