#include "slow_registers.hpp"

namespace kcpu::iodev {

slow_registers::slow_registers() {
    for(uint i = 0; i < REGISTER_COUNT; i++) {
        regs[i] = 0x7A;
    }
}

static halfcycle_count_t reg_to_halfcycles(int reg) {
    return reg;
}

std::vector<regval_t> slow_registers::get_reserved_ports() {
    std::vector<regval_t> ports;
    for(uint i = 0; i < REGISTER_COUNT; i++) {
        ports.push_back(PORT_BASE + i);
    }
    return ports;
}

halfcycle_count_t slow_registers::write(regval_t port, regval_t val) {
    uint reg = port - PORT_BASE;
    vm_assert(port >= PORT_BASE && reg <= REGISTER_COUNT);

    regs[reg] = val;
    return reg_to_halfcycles(reg);
}

std::pair<regval_t, halfcycle_count_t> slow_registers::read(regval_t port) {
    uint reg = port - PORT_BASE;
    vm_assert(port >= PORT_BASE && reg <= REGISTER_COUNT);

    return std::pair(regs[reg], reg_to_halfcycles(reg));
}

}