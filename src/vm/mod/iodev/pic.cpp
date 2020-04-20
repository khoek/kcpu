#include <strings.h>

#include "pic.hpp"

namespace kcpu::iodev {

pic::pic(vm_logger &logger, ctl_out_interface &ctl) : single_port_io_device(PORT_BASE), logger(logger), ctl(ctl) {
}

void pic::dump_registers() {
    logger.logf("[IRQs] mask: %04X pend: %04X serv: %04X (ap:%d)\n",
        irq_mask, irq_pend, irq_serv, aint_prev ? 1 : 0);
}

static regval_t get_lowest_bit(regval_t bitmask) {
    return bitmask ? 1 << (ffs(bitmask) - 1) : 0;
}

// HARDWARE NOTE: NMI enable jumper?
// HARDWARE NOTE: This function ignores the masked bits!
regval_t pic::get_next_pending_bit(bool expect_nonzero, bool nmi_only) {
    regval_t irqs_masked = irq_pend & ((irq_mask | MASK_NMIS) & (nmi_only ? MASK_NMIS : 0xFFFF));
    if(!irqs_masked && expect_nonzero) {
        throw vm_error("irq_ACK with no active interrupt");
    }
    return get_lowest_bit(irqs_masked);
}

bool pic::is_pnmi_active() {
    return !(irq_serv & MASK_NMIS) && !!get_next_pending_bit(false, true);
}

/*
    PINT is higher if: 1) there is any interrupt pending and no interrupt
    is being serviced, or 2) the NMI is pending and is not currently being serviced.
*/
bool pic::is_pint_active() {
    return (!irq_serv && !!get_next_pending_bit(false, false)) || is_pnmi_active();
}

// HARDWARE NOTE: Let's only set an interrupt pending in the PIC on the rising edge of an interrupt line, so we can implement a hard "reset button" for example.
void pic::assert(regval_t bit) {
    // NOTE in practice this condition can arise, but for testing purposes in the simulator it likely indicates a bug.
    vm_assert(!(irq_serv & (1 << bit)));

    irq_pend |= 1 << bit;
}

halfcycle_count_t pic::write(regval_t val) {
    switch(val & MASK_CMD) {
        case CMD_EOI: {
            if(!irq_serv) {
                throw vm_error("EOI with no active interrupt");
            }
            irq_serv &= ~get_lowest_bit(irq_serv);
            break;
        }
        case CMD_SET_MASK: {
            irq_mask = val & MASK_VAL;
            break;
        }
        case CMD_SET_PEND: {
            irq_pend = val & MASK_VAL;
            break;
        }
        default: throw vm_error("unknown pic command");
    }

    return 0;
}

std::pair<regval_t, halfcycle_count_t> pic::read() {
    return std::pair(irq_serv, 0);
}

// HARDWARE NOTE: This implementation is a bit of a hack since the PIC
// handles aint asynchronously (at least I think that is how it will be implemented).
void pic::process_halfcycle(bool offclock) {
    // Primitive rising edge detection
    if(aint_prev == ctl.is_aint_active()) {
        return;
    }
    aint_prev = ctl.is_aint_active();

    if(!aint_prev) {
        return;
    }

    // HARDWARE NOTE: Implement this in hardware using daisy-chaining.
    regval_t pending_bit = get_next_pending_bit(true, false);
    irq_serv |= pending_bit;
    // HARDWARE NOTE: It is important that we clear the pending bit, and record it in the in-service register
    // at this point, so that we can recieve further copies of that interrupt while it is being serviced.
    irq_pend &= ~pending_bit;

    // Consequently, since irq_serv is now nonzero, the PINT line will go low.
}

}