#include "jumpers.hpp"

namespace kcpu::iodev {

jumpers::jumpers(ctl_out_interface &ctl, pic_in_interface &pic) : single_port_io_device(PORT_BASE), ctl(ctl), pic(pic) {
}

std::pair<regval_t, halfcycle_count_t> jumpers::read() {
    return std::pair(flags, 0);
}

halfcycle_count_t jumpers::write(regval_t val) {
    flags = val;
    return 0;
}

void jumpers::process_halfcycle(bool offclock) {
    if(flags & FLAG_TUI2NMI) {
        if(offclock && ctl.is_tui_active()) {
            pic.assert(pic_in_interface::NMI_INT);
        }
    }
}

}