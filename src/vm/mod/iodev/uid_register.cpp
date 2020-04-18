#include "uid_register.hpp"

namespace kcpu::iodev {

const int uid_register::UID = 0xBEEF;

uid_register::uid_register() : single_port_io_device(PORT_BASE) {
}

halfcycle_count_t uid_register::write(regval_t val) {
    throw vm_error("writing to serial register");
}

std::pair<regval_t, halfcycle_count_t> uid_register::read() {
    return std::pair(UID, 0);
}

}