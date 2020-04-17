#ifndef VM_INTERFACE_PIC_H
#define VM_INTERFACE_PIC_H

#include "../../common.hpp"

namespace kcpu {

class pic_interface {
    public:
    virtual void assert(regval_t bit) = 0;
    virtual bool is_pint_active() = 0;
    virtual void handle_aint(bool aint) = 0;
};

}

#endif
