#ifndef VM_MOD_IODEV_H
#define VM_MOD_IODEV_H

#include <vector>
#include "../../common.hpp"
#include "../interface/pic.hpp"

// This class is not a module. It just simulates external IO device hardware
// attached over the IO bus.

namespace kcpu {

typedef unsigned int halfcycle_count_t;

class io_device {
    public:
    virtual std::vector<regval_t> get_reserved_ports() = 0;
    virtual std::pair<regval_t, halfcycle_count_t> read(regval_t port) = 0;
    virtual halfcycle_count_t write(regval_t port, regval_t val) = 0;

    virtual void process_halfcycle(pic_in_interface &pic, bool offclock);
};

class single_port_io_device : public io_device {
    private:
    regval_t port;

    public:
    single_port_io_device(regval_t port);

    std::vector<regval_t> get_reserved_ports() final override;
    std::pair<regval_t, halfcycle_count_t> read(regval_t port) final override;
    halfcycle_count_t write(regval_t port, regval_t val) final override;

    virtual std::pair<regval_t, halfcycle_count_t> read() = 0;
    virtual halfcycle_count_t write(regval_t val) = 0;
};

}

#endif