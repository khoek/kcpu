#ifndef VM_MOD_IODEV_PIC_H
#define VM_MOD_IODEV_PIC_H

#include "iodev.hpp"

namespace kcpu {

namespace iodev {

class pic : public io_device {
    private:
    static const unsigned int PORT_BASE = 0x10;

    static const unsigned int REG_IMR = 0;
    static const unsigned int REG_ISR = 1;
    static const unsigned int REGISTER_COUNT = 2;

    regval_t imr; /* Interrupt Mask    Register */
    regval_t isr; /* Interrupt Service Register */
    regval_t ipr; /* Interrupt Pending Register */

    public:
    std::vector<regval_t> get_reserved_ports();
    std::pair<regval_t, halfcycle_count_t> read(regval_t port);
    halfcycle_count_t write(regval_t port, regval_t val);
};

}

}

#endif