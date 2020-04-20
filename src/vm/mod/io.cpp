#include "../../spec/ucode.hpp"
#include "../../spec/inst.hpp"
#include "io.hpp"

namespace kcpu {

mod_io::mod_io(vm_logger &logger, ctl_out_interface &ctl) : logger(logger), iodev_manager(logger),
        id_probe(iodev_manager.get_ports()), id_pic(logger, ctl),
        id_jumpers(ctl, id_pic), id_slow_ints(id_pic) {
    iodev_manager.register_iodev(id_probe);
    iodev_manager.register_iodev(id_pic);

    iodev_manager.register_iodev(id_uid_register);

    // FIXME implement external memory?
    // iodev_manager.register_iodev(id_external_memory);

    iodev_manager.register_iodev(id_video);
    // devices.push_back(<a serial thing? :D>); (this one would be disabled by default.)

    iodev_manager.register_iodev(id_slow_registers);
    iodev_manager.register_iodev(id_jumpers);
    iodev_manager.register_iodev(id_slow_ints);
}

void mod_io::dump_registers() {
    iodev_manager.dump_registers();
    id_pic.dump_registers();
}

bool mod_io::is_io_done() {
    return iodev_manager.is_io_done();
}

pic_interface & mod_io::get_pic() {
    return id_pic;
}

void mod_io::clock_outputs(uinst_t ui, bus_state &s) {
    switch(ui & MASK_CTRL_ACTION) {
        case ACTION_CTRL_NONE:
        case ACTION_GCTRL_RIP_BUSA_O:
        case ACTION_MCTRL_BUSMODE_X: {
            break;
        }
        case ACTION_GCTRL_USE_ALT: {

            break;
        }
        default: throw vm_error("unknown CTRL_COMMAND");
    }

    if(is_gctrl_nrm_io_readwrite(ui)) {
        if((ui & MASK_GCTRL_DIR) == GCTRL_CREG_I) {
            iodev_manager.before_clock_outputs_read(s.early_read(BUS_A));
        } else {
            iodev_manager.before_clock_outputs_write(s.early_read(BUS_A), s.early_read(BUS_B));
        }
    }

    iodev_manager.process_halfcycle(false);

    if(is_gctrl_nrm_io_readwrite(ui)) {
        if((ui & MASK_GCTRL_DIR) == GCTRL_CREG_I) {
            if(iodev_manager.is_io_done()) {
                s.assign(BUS_B, iodev_manager.get_read_result());
            }
            iodev_manager.after_clock_outputs_read();
        } else {
            iodev_manager.after_clock_outputs_write();
        }
    } else {
        iodev_manager.after_clock_outputs_none();
    }
}

void mod_io::clock_inputs(uinst_t ui, bus_state &s) {
}

void mod_io::offclock_pulse() {
    iodev_manager.process_halfcycle(true);
}

}