#ifndef VM_MOD_IODEV_uid_register_H
#define VM_MOD_IODEV_uid_register_H

#include "iodev.hpp"

namespace kcpu {

namespace iodev {

class uid_register : public single_port_io_device {
    private:
    static const int PORT_BASE = 0xA0;
    static const int UID;

    public:
    uid_register();

    std::pair<regval_t, halfcycle_count_t> read() override;
    halfcycle_count_t write(regval_t val) override;
};

}

}

#endif