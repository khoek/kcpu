#ifndef VM_MOD_IO_H
#define VM_MOD_IO_H

#include "../common.hpp"

namespace kcpu {

class mod_io {
    private:
    vm_logger &logger;
    bool io_done;

    public:

    mod_io(vm_logger &logger);
    bool get_io_done();

    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
};

}

#endif