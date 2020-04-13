#include "iodev.hpp"
#include "../../common.hpp"

namespace kcpu {

single_port_io_device::single_port_io_device(regval_t port) : port(port) {
}

std::vector<regval_t> single_port_io_device::get_reserved_ports() {
    return { port };
}

std::pair<regval_t, halfcycle_count_t> single_port_io_device::read(regval_t p) {
    vm_assert(p == port);
    return read();
}

halfcycle_count_t single_port_io_device::write(regval_t p, regval_t val) {
    vm_assert(p == port);
    return write(val);
}

}