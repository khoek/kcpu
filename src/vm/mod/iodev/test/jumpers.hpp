#ifndef VM_MOD_IODEV_TEST_JUMPERS_H
#define VM_MOD_IODEV_TEST_JUMPERS_H

#include "../iodev.hpp"

namespace kcpu {

namespace iodev {

class jumpers : public single_port_io_device {
    private:
    static const int PORT_BASE = 0xD0;

    // Connect the TUI line of CTL, ANDed with NOT_CLOCK, to the NMI assert of the PIC
    static const regval_t FLAG_TUI2NMI = 0x0001;

    ctl_out_interface &ctl;
    pic_in_interface &pic;

    regval_t flags;

    public:
    jumpers(ctl_out_interface &ctl, pic_in_interface &pic);
    std::pair<regval_t, halfcycle_count_t> read() override;
    halfcycle_count_t write(regval_t val) override;

    void process_halfcycle(bool offclock) override;
};

}

}

#endif