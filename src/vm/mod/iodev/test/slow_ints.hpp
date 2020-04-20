#ifndef VM_MOD_IODEV_TEST_SLOW_INTS_H
#define VM_MOD_IODEV_TEST_SLOW_INTS_H

#include "../iodev.hpp"

namespace kcpu {

namespace iodev {

class slow_ints : public single_port_io_device {
    private:
    static const int PORT_BASE = 0xD1;
    static const regval_t MASK_NMI_FLAG = 0x8000;

    static const int NMI_NUM = 0;
    static const int INT_NUM = 3;

    pic_in_interface &pic;

    regval_t count[2] = {0, 0};

    void single_count_process_halfcycle(int count_num, int int_num);

    public:
    slow_ints(pic_in_interface &pic);
    std::pair<regval_t, halfcycle_count_t> read() override;
    halfcycle_count_t write(regval_t val) override;

    void process_halfcycle(bool offclock) override;
};

}

}

#endif