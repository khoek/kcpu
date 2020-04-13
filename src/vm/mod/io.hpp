#ifndef VM_MOD_IO_H
#define VM_MOD_IO_H

#include "../common.hpp"
#include "iodev/iodev_manager.hpp"
#include "iodev/probe.hpp"
#include "iodev/uid_register.hpp"
#include "iodev/slow_registers.hpp"

namespace kcpu {

class mod_io {
    private:
    vm_logger &logger;

    io_device_manager iodev_manager;
    iodev::probe id_probe;
    iodev::uid_register id_uid_register;
    iodev::slow_registers id_slow_registers;

    public:
    mod_io(vm_logger &logger);
    void dump_registers();
    bool is_io_done();

    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
    void offclock_pulse();
};

}

#endif