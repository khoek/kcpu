#include "slow_ints.hpp"

namespace kcpu::iodev {

slow_ints::slow_ints(pic_in_interface &pic) : single_port_io_device(PORT_BASE), pic(pic) {
}

std::pair<regval_t, halfcycle_count_t> slow_ints::read() {
    throw vm_error("unimplemented");
}

halfcycle_count_t slow_ints::write(regval_t val) {
    if(!(val & MASK_NMI_FLAG)) {
        count[0] = (val & ~MASK_NMI_FLAG) + 1;
    }

    if(val & MASK_NMI_FLAG) {
        count[1] = (val & ~MASK_NMI_FLAG) + 1;
    }

    return 0;
}

void slow_ints::single_count_process_halfcycle(int count_num, int int_num) {
    if(!count[count_num]) {
        return;
    }

    count[count_num]--;

    if(!count[count_num]) {
        pic.assert(int_num);
    }
}

void slow_ints::process_halfcycle(bool offclock) {
    single_count_process_halfcycle(0, INT_NUM);
    single_count_process_halfcycle(1, NMI_NUM);
}

}