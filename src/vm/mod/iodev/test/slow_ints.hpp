#ifndef VM_MOD_IODEV_TEST_SLOW_INTS_H
#define VM_MOD_IODEV_TEST_SLOW_INTS_H

#include "../iodev.hpp"

namespace kcpu {

namespace iodev {

class slow_ints : public single_port_io_device {
    private:
    static const int PORT_BASE = 0xD0;
    static const int INT_NUM = 3;

    regval_t count = 0;

    public:
    slow_ints();
    std::pair<regval_t, halfcycle_count_t> read() override;
    halfcycle_count_t write(regval_t val) override;

    void process_halfcycle(pic_in_interface &pic, bool offclock) override;
};

}

}

#endif