#include "probe.hpp"

namespace kcpu::iodev {

probe::probe(const std::unordered_map<regval_t, std::reference_wrapper<io_device>> &ports) : single_port_io_device(PORT_BASE), target_port(0), ports(ports) {
}

halfcycle_count_t probe::write(regval_t val) {
    target_port = val;
    return 0;
}

std::pair<regval_t, halfcycle_count_t> probe::read() {
    // This assertion is not formally neccesary, but is useful to detect bad port IO.
    // i.e. when we read from port 0 by accident, when we aren't using the probe function.
    vm_assert(target_port != 0);
    return std::pair(ports.find(target_port) == ports.end() ? 0 : 1, 0);
}

}