#ifndef VM_INTERFACE_CTL_H
#define VM_INTERFACE_CTL_H

#include "../../common.hpp"

namespace kcpu {

class ctl_out_interface {
    public:
    virtual bool is_aint_active() = 0;
    /*
        "True uinstruction". High on the falling edge before
        a clock where a uinst which is part of a "true instruction",
        i.e. not an instruction fetch or interrupt handling.

        HARDWARE NOTE: This signal should only be inspected when
        the clock is going LOW.
    */
    virtual bool is_tui_active() = 0;
};

}

#endif
