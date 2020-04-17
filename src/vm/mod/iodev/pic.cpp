#include <strings.h>

#include "pic.hpp"

namespace kcpu::iodev {

pic::pic(vm_logger &logger) : single_port_io_device(PORT_BASE), logger(logger) {
}

void pic::dump_registers() {
    logger.logf("[IRQs] mask: %04X pend: %04X serv: %04X (ap:%d)\n",
        irq_mask, irq_pend, irq_serv, aint_prev ? 1 : 0);
}

// HARDWARE NOTE: NMI enable jumper?
// HARDWARE NOTE: This function ignores the masked bits!
regval_t pic::get_next_pending_bit(bool expect_nonzero) {
    regval_t irqs_masked = irq_pend & (irq_mask | MASK_NMIS);
    if(!irqs_masked) {
        if(expect_nonzero) {
            throw new vm_error("irq_ACK with no active interrupt");
        }

        return 0;
    }
    return 1 << (ffs(irqs_masked) - 1);
}

// HARDWARE NOTE: This function represents an asynchronous propogation of signals through the PIC LOGIC!
void pic::handle_aint(bool aint) {
    // Primitive rising edge detection
    if(aint_prev == aint) {
        return;
    }
    aint_prev = aint;

    if(!aint) {
        return;
    }

    // FIXME how to implement this in hardware?
    regval_t pending_bit = get_next_pending_bit(true);
    irq_serv = pending_bit;
    // HARDWARE NOTE: It is important that we clear the pending bit, and record it in the in-service register
    // at this point, so that we can recieve further copies of that interrupt while it is being serviced.
    irq_pend &= ~pending_bit;

    // Consequently, since irq_serv is now nonzero, the PINT line will go low.
}

bool pic::is_pint_active() {
    return !irq_serv && !!get_next_pending_bit(false);
}

// HARDWARE NOTE: Let's only set an interrupt pending in the PIC on the rising edge of an interrupt line, so we can implement a hard "reset button" for example.
void pic::assert(regval_t bit) {
    // NOTE in practice this condition could arise, but for testing purposes in the simulator it likely indicates a bug.
    vm_assert(!(irq_pend & (1 << bit)));

    irq_pend |= 1 << bit;
}

halfcycle_count_t pic::write(regval_t val) {
    switch(val & MASK_CMD) {
        case CMD_EOI: {
            if(!irq_serv) {
                throw new vm_error("EOI with no active interrupt");
            }
            irq_serv = 0;
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
        default: throw new vm_error("unknown pic command");
    }

    return 0;
}

std::pair<regval_t, halfcycle_count_t> pic::read() {
    return std::pair(irq_serv, 0);
}

}