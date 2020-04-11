#ifndef VM_MOD_IO_H
#define VM_MOD_IO_H

#include "../common.hpp"

namespace kcpu {

class mod_io {
    private:
    vm_logger &logger;

    public:

    mod_io(vm_logger &logger);
    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
};

}

#endif