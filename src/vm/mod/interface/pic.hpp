#ifndef VM_INTERFACE_PIC_H
#define VM_INTERFACE_PIC_H

#include "../../common.hpp"
#include "ctl.hpp"

namespace kcpu {

class pic_out_interface {
    public:
    virtual bool is_pint_active() = 0;
    virtual bool is_pnmi_active() = 0;
};

class pic_in_interface {
    public:
    // Convenience constant for asserting the NMI.
    static const regval_t NMI_INT = 0;

    virtual void assert(regval_t bit) = 0;
};

class pic_interface : public pic_in_interface, public pic_out_interface {
};

}

#endif
