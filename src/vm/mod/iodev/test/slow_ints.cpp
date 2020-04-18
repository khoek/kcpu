#include "slow_ints.hpp"

namespace kcpu::iodev {

slow_ints::slow_ints() : single_port_io_device(PORT_BASE) {
}

std::pair<regval_t, halfcycle_count_t> slow_ints::read() {
    throw new vm_error("unimplemented");
}

halfcycle_count_t slow_ints::write(regval_t val) {
    if(!(val & MASK_NMI_FLAGS)) {
        count[0] = (val & ~MASK_NMI_FLAGS) + 1;
    }

    if(val & BIT_NMI1_FLAG) {
        count[1] = (val & ~MASK_NMI_FLAGS) + 1;
    }

    if(val & BIT_NMI2_FLAG) {
        count[2] = (val & ~MASK_NMI_FLAGS) + 1;
    }

    return 0;
}

void slow_ints::single_count_process_halfcycle(pic_in_interface &pic, int count_num, int int_num) {
    if(!count[count_num]) {
        return;
    }

    count[count_num]--;

    if(!count[count_num]) {
        pic.assert(int_num);
    }
}

void slow_ints::process_halfcycle(pic_in_interface &pic, bool offclock) {
    single_count_process_halfcycle(pic, 0, INT_NUM);
    single_count_process_halfcycle(pic, 1, NMI1_NUM);
    single_count_process_halfcycle(pic, 2, NMI2_NUM);
}

}