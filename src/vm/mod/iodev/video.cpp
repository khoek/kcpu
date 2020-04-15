#include "video.hpp"

namespace kcpu::iodev {

video::video() : rend(graphics::get_graphics().new_renderer(WIDTH * PIXEL_WIDTH, HEIGHT * PIXEL_WIDTH)) {
}

video::~video() {
    delete rend;
}

constexpr unsigned int video::get_framebuffer_size() {
    return WIDTH * PIXEL_WIDTH * HEIGHT * PIXEL_WIDTH * 4;
}

unsigned int video::get_addr() {
    unsigned int addr = (((unsigned int) hiaddr) << 16) | ((unsigned int) loaddr);
    vm_assert(addr < get_framebuffer_size());
    return addr;
}

video::vram_addr video::decode_addr(unsigned int addr) {
    return { .r = addr / (WIDTH * 4), .c = (addr % (WIDTH * 4)) / 4, .comp = addr % 4};
}

std::vector<regval_t> video::get_reserved_ports() {
    std::vector<regval_t> ports;
    for(int i = 0; i < REGISTER_COUNT; i++) {
        ports.push_back(PORT_BASE + i);
    }
    return ports;
}

void video::handle_command(regval_t cmd) {
    switch(cmd) {
        case CMD_FLIP: {
            rend->get_fb().advance();
            break;
        }
        case CMD_STREAMRST: {
            stream_ptr = 0;
            break;
        }
        default: throw new vm_error("unknown video command");
    }
}

void video::vram_write(unsigned int a, regval_t val) {
    vram_addr addr = decode_addr(a);

    for(int r = 0; r < PIXEL_WIDTH; r++) {
        for(int c = 0; c < PIXEL_WIDTH; c++) {
            int new_r = r + (PIXEL_WIDTH * addr.r);
            int new_c = c + (PIXEL_WIDTH * addr.c);
            rend->get_fb().get_fb_next()[(new_r * 4 * WIDTH * PIXEL_WIDTH) + (new_c * 4) + addr.comp] = val;
        }
    }
}

regval_t video::vram_read(unsigned int a) {
    vram_addr addr = decode_addr(a);

    int new_r = (PIXEL_WIDTH * addr.r);
    int new_c = (PIXEL_WIDTH * addr.c);
    return rend->get_fb().get_fb_next()[(new_r * 4 * WIDTH * PIXEL_WIDTH) + (new_c * 4) + addr.comp];
}

halfcycle_count_t video::write(regval_t port, regval_t val) {
    int reg = port - PORT_BASE;
    vm_assert(reg >= 0 && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_CMD: {
            handle_command(val);
            break;
        }
        case REG_STREAM: {
            vram_write(stream_ptr++, val);
            break;
        }
        case REG_HIADDR: {
            hiaddr = val;
            break;
        }
        case REG_LOADDR: {
            loaddr = val;
            break;
        }
        case REG_DATA: {
            vram_write(get_addr(), val);
            break;
        }
        default: throw new vm_error("unknown video register");
    }

    return 0;
}

std::pair<regval_t, halfcycle_count_t> video::read(regval_t port) {
    int reg = port - PORT_BASE;
    vm_assert(reg >= 0 && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_DATA: {
            return std::pair(vram_read(get_addr()), 0);
        }
        case REG_CMD:
        case REG_STREAM:
        case REG_HIADDR:
        case REG_LOADDR: throw new vm_error("cannot read from that graphics register");
        default: throw new vm_error("unknown graphics register");
    }
}

}