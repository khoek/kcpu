#include "pic.hpp"

namespace kcpu::iodev {

// FIXME let the PIC assert the INT1 and INT2 lines.

std::vector<regval_t> pic::get_reserved_ports() {
    std::vector<regval_t> ports;
    for(int i = 0; i < REGISTER_COUNT; i++) {
        ports.push_back(PORT_BASE + i);
    }
    return ports;
}

halfcycle_count_t pic::write(regval_t port, regval_t val) {
    int reg = port - PORT_BASE;
    vm_assert(reg >= 0 && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_IMR: {
            imr = val;
            break;
        }
        case REG_ISR: {
            isr &= ~val;
            break;
        }
        default: throw new vm_error("unknown pic register");
    }

    return 0;
}

std::pair<regval_t, halfcycle_count_t> pic::read(regval_t port) {
    int reg = port - PORT_BASE;
    vm_assert(reg >= 0 && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_IMR: {
            return std::pair(imr, 0);
        }
        case REG_ISR: {
            return std::pair(isr, 0);
        }
        default: throw new vm_error("unknown pic register");
    }
}

}