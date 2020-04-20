#ifndef VM_MOD_IODEV_PIC_H
#define VM_MOD_IODEV_PIC_H

#include "iodev.hpp"
#include "../interface/pic.hpp"

namespace kcpu {

namespace iodev {

// HARDWARE NOTE: Since the int_mask bits have to be high to enable an interrupt,
// the computer can start safely with interrupts disabled so long as this register
// is reset upon boot.

class pic : public pic_interface, public single_port_io_device {
    private:
    static const unsigned int PORT_BASE = 0x01;

    static const regval_t MASK_CMD  = 0xC000;
    static const regval_t MASK_VAL  = 0x3FFF;
    static const regval_t SHIFT_CMD = 14;

    static const regval_t CMD_EOI      = 0b01 << SHIFT_CMD;
    static const regval_t CMD_SET_MASK = 0b10 << SHIFT_CMD;
// HARDWARE NOTE: In hardware we don't have to implement this one; and a CMD_CLEAR_PEND is probably sufficient.
// But it does allow us to raise interrupts from software, which is great for testing. (So maybe do it?)
    static const regval_t CMD_SET_PEND = 0b11 << SHIFT_CMD;

// HARDWARE NOTE: ASK_NMIS assert the additional PNMI line when pending BUT DO NOT ignore the irq_mask field,
// else we could have NMIs being recursively handled.
    static const regval_t MASK_NMIS = 0x0001;

    bool aint_prev = false;

    regval_t irq_mask = 0;
    regval_t irq_serv = 0;
    regval_t irq_pend = 0;

    regval_t get_next_pending_bit(bool expect_nonzero, bool nmi_only);

    vm_logger &logger;
    ctl_out_interface &ctl;

    public:
    pic(vm_logger &logger, ctl_out_interface &ctl);
    void dump_registers();

    bool is_pint_active() override;
    bool is_pnmi_active() override;
    void assert(regval_t bit) override;

    std::pair<regval_t, halfcycle_count_t> read() override;
    halfcycle_count_t write(regval_t val) override;

    void process_halfcycle(bool offclock) override;
};

}

}

#endif