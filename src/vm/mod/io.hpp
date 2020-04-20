#ifndef VM_MOD_IO_H
#define VM_MOD_IO_H

#include "../common.hpp"
#include "iodev/iodev_manager.hpp"
#include "iodev/probe.hpp"
#include "iodev/pic.hpp"
#include "iodev/uid_register.hpp"
#include "iodev/video.hpp"
#include "iodev/test/slow_registers.hpp"
#include "iodev/test/jumpers.hpp"
#include "iodev/test/slow_ints.hpp"

namespace kcpu {

class mod_io {
    private:
    vm_logger &logger;

    io_device_manager iodev_manager;
    iodev::probe id_probe;
    iodev::pic id_pic;
    iodev::uid_register id_uid_register;
    iodev::video id_video;

    iodev::slow_registers id_slow_registers;
    iodev::jumpers id_jumpers;
    iodev::slow_ints id_slow_ints;

    public:
    mod_io(vm_logger &logger, ctl_out_interface &ctl);
    void dump_registers();
    bool is_io_done();
    pic_interface & get_pic();

    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
    void offclock_pulse();
};

}

#endif