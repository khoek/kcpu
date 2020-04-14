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
            rend->flip();
            break;
        }
        default: throw new vm_error("unknown video command");
    }
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
            // FIXME implement
            throw new vm_error("unimplemented");
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
            unsigned int addr = get_addr();
            unsigned int addr_r = addr / (WIDTH * 4);
            unsigned int addr_c = (addr % (WIDTH * 4)) / 4;
            unsigned int addr_comp = addr % 4;

            for(int r = 0; r < PIXEL_WIDTH; r++) {
                for(int c = 0; c < PIXEL_WIDTH; c++) {
                    int new_r = r + (PIXEL_WIDTH * addr_r);
                    int new_c = c + (PIXEL_WIDTH * addr_c);
                    rend->get_next_framebuffer()[(new_r * 4 * WIDTH * PIXEL_WIDTH) + (new_c * 4) + addr_comp] = val;
                }
            }
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
            return std::pair(rend->get_next_framebuffer()[get_addr()], 0);
        }
        case REG_CMD:
        case REG_STREAM:
        case REG_HIADDR:
        case REG_LOADDR: throw new vm_error("cannot read from that graphics register");
        default: throw new vm_error("unknown graphics register");
    }
}

}