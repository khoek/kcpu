#include "graphics.hpp"

namespace kcpu::iodev {

// graphics::graphics() : renderer(WIDTH * PIXEL_WIDTH, HEIGHT * PIXEL_WIDTH) {
// }
graphics::graphics() : renderer(WIDTH * PIXEL_WIDTH, HEIGHT * PIXEL_WIDTH) {
}

std::vector<regval_t> graphics::get_reserved_ports() {
    std::vector<regval_t> ports;
    for(int i = 0; i < REGISTER_COUNT; i++) {
        ports.push_back(PORT_BASE + i);
    }
    return ports;
}

void graphics::handle_command(regval_t cmd) {
    switch(cmd) {
        case CMD_FLIP: {
            renderer.flip();
            break;
        }
        default: throw new vm_error("unknown graphics command");
    }
}

halfcycle_count_t graphics::write(regval_t port, regval_t val) {
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
        case REG_ADDR: {
            addr = val;
            break;
        }
        case REG_DATA: {
            int addr_r = addr / (WIDTH * 4);
            int addr_c = (addr % (WIDTH * 4)) / 4;
            int addr_comp = addr % 4;

            // FIXME bounds check
            for(int r = 0; r < PIXEL_WIDTH; r++) {
                for(int c = 0; c < PIXEL_WIDTH; c++) {
                    int new_r = r + (PIXEL_WIDTH * addr_r);
                    int new_c = c + (PIXEL_WIDTH * addr_c);
                    renderer.get_next_framebuffer()[(new_r * 4 * WIDTH * PIXEL_WIDTH) + (new_c * 4) + addr_comp] = val;
                }
            }
            break;
        }
        default: throw new vm_error("unknown graphics register");
    }

    return 0;
}

std::pair<regval_t, halfcycle_count_t> graphics::read(regval_t port) {
    int reg = port - PORT_BASE;
    vm_assert(reg >= 0 && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_DATA: {
            // FIXME bounds check
            return std::pair(renderer.get_next_framebuffer()[addr], 0);
        }
        case REG_CMD: throw new vm_error("cannot read from graphics command register");
        case REG_STREAM: throw new vm_error("cannot read from graphics stream register");
        case REG_ADDR: throw new vm_error("cannot read from graphics address register");
        default: throw new vm_error("unknown graphics register");
    }
}

}