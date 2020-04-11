#include "../../spec/ucode.hpp"
#include "../../spec/inst.hpp"
#include "io.hpp"

namespace kcpu {

mod_io::mod_io(vm_logger &logger) : logger(logger) {
}

void mod_io::clock_outputs(uinst_t ui, bus_state &s) {
    switch(ui & MASK_CTRL_COMMAND) {
        case COMMAND_NONE:
        case COMMAND_RCTRL_RSP_INC: {
            break;
        }
        case COMMAND_IO_READ: {
            break;
        }
        case COMMAND_IO_WRITE:{
            break;
        }
        default: throw vm_error("unknown GCTRL_COMMAND");
    }
}

void mod_io::clock_inputs(uinst_t ui, bus_state &s) {
    switch(ui & MASK_CTRL_COMMAND) {
        case COMMAND_NONE:
        case COMMAND_RCTRL_RSP_INC: {
            break;
        }
        case COMMAND_IO_READ: {
            break;
        }
        case COMMAND_IO_WRITE:{
            break;
        }
        default: throw vm_error("unknown GCTRL_COMMAND");
    }
}

}