#ifndef VM_MOD_IODEV_PROBE_H
#define VM_MOD_IODEV_PROBE_H

#include <unordered_map>
#include <functional>

#include "iodev.hpp"

namespace kcpu {

namespace iodev {

class probe : public single_port_io_device {
    private:
    static const int PORT_BASE = 0x00;

    regval_t target_port;
    const std::unordered_map<regval_t, std::reference_wrapper<io_device>> & ports;

    public:
    probe(const std::unordered_map<regval_t, std::reference_wrapper<io_device>> & ports);
    std::pair<regval_t, halfcycle_count_t> read();
    halfcycle_count_t write(regval_t val);
};

}

}

#endif