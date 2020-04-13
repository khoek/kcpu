#ifndef VM_MOD_IODEV_SLOW_REGISTERS_H
#define VM_MOD_IODEV_SLOW_REGISTERS_H

#include "iodev.hpp"

namespace kcpu {

namespace iodev {

class slow_registers : public io_device {
    private:
    static const int PORT_BASE = 0xF0;
    static const int REGISTER_COUNT = 5;

    mutable regval_t regs[REGISTER_COUNT];

    public:
    slow_registers();
    std::vector<regval_t> get_reserved_ports();
    std::pair<regval_t, halfcycle_count_t> read(regval_t port);
    halfcycle_count_t write(regval_t port, regval_t val);
};

}

}

#endif