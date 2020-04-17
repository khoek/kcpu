#include <iostream>
#include "slow_ints.hpp"

namespace kcpu::iodev {

slow_ints::slow_ints() : single_port_io_device(PORT_BASE) {
}

std::pair<regval_t, halfcycle_count_t> slow_ints::read() {
    throw new vm_error("unimplemented");
}

halfcycle_count_t slow_ints::write(regval_t val) {
    count = val + 1;
    return 0;
}

void slow_ints::process_halfcycle(pic_in_interface &pic, bool offclock) {
    if(!count) {
        return;
    }

    count--;

    if(!count) {
        pic.assert(INT_NUM);
    }
}

}