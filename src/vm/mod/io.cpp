#include "../../spec/ucode.hpp"
#include "../../spec/inst.hpp"
#include "io.hpp"

namespace kcpu {

mod_io::mod_io(vm_logger &logger) : logger(logger), iodev_manager(logger), id_probe(iodev_manager.get_ports()) {
    iodev_manager.register_iodev(id_probe);
    iodev_manager.register_iodev(id_uid_register);
    iodev_manager.register_iodev(id_slow_registers);

    // FIXME implement external memory
    // iodev_manager.register_iodev(id_external_memory);

    // FIXME add build flags for both of these, and make the respective tests
    // still pass by consulting the probe API for info about their presence.
    // ALTERNATIVELY, we can just provide a relevant "dummy" entity when they
    // are absent (e.g. just emulate a framebuffer, and don't draw, easy when
    // the canvas is abstracted away).
    iodev_manager.register_iodev(id_graphics);
    // devices.push_back(<a serial thing? :D>); (this one would be disabled by default.)
}

void mod_io::dump_registers() {
    iodev_manager.dump_registers();
}

bool mod_io::is_io_done() {
    return iodev_manager.is_io_done();
}

void mod_io::clock_outputs(uinst_t ui, bus_state &s) {
    switch(ui & MASK_CTRL_COMMAND) {
        case COMMAND_NONE:
        case COMMAND_RCTRL_RSP_INC: {
            break;
        }
        case COMMAND_IO_READ: {
            iodev_manager.before_clock_outputs_read(s.early_read(BUS_A));
            break;
        }
        case COMMAND_IO_WRITE: {
            iodev_manager.before_clock_outputs_write(s.early_read(BUS_A), s.early_read(BUS_B));
            break;
        }
        default: throw vm_error("unknown CTRL_COMMAND");
    }

    iodev_manager.process_halfcycle(false);

    switch(ui & MASK_CTRL_COMMAND) {
        case COMMAND_NONE:
        case COMMAND_RCTRL_RSP_INC: {
            iodev_manager.after_clock_outputs_none();
            break;
        }
        case COMMAND_IO_READ: {
            if(iodev_manager.is_io_done()) {
                s.assign(BUS_B, iodev_manager.get_read_result());
            }
            iodev_manager.after_clock_outputs_read();
            break;
        }
        case COMMAND_IO_WRITE: {
            iodev_manager.after_clock_outputs_write();
            break;
        }
        default: throw vm_error("unknown CTRL_COMMAND");
    }
}

void mod_io::clock_inputs(uinst_t ui, bus_state &s) {
}

void mod_io::offclock_pulse() {
    iodev_manager.process_halfcycle(true);
}

}