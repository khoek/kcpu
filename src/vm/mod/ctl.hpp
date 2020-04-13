#ifndef VM_MOD_CTL_H
#define VM_MOD_CTL_H

#include "../common.hpp"

namespace kcpu {

#define CBIT_INSTMASK 0
#define CBIT_HALTED   1
#define CBIT_ABORTED  2

// This bit is a bit tricky.
// It is set on CLK rising edge whenever IO_READ or IO_WRITE are asserted.
// It is cleared on CLK falling edge whenever IO_DONE is asserted.

// Moreover, it masks the UC increment and UC unlatching, subject to: if IO_DONE is asserted on a CLK
// falling edge (so that CBIT_IO_WAIT should be cleared) simultaneously the its UC unlatching-inhbit
// function does not occur (that is, IO_DONE hard overrides the UC unlatching-inhibit function of this bit,
// and clears the bit at the same time).
//
// On the other hand, the UC should inc once on a rising edge of CLK at which time IO_WAIT is simultaneously set.
#define CBIT_IO_WAIT  3

#define NUM_CBITS 4

class mod_ctl {
    private:
    vm_logger &logger;

    void set_instmask_enabled(bool state);
    void ft_enter();

    public:
    //FIXME make this private
    regval_t reg[NUM_SREGS];
    bool cbits[NUM_CBITS];

    mod_ctl(vm_logger &logger);
    void dump_registers();
    regval_t get_inst();
    uinst_t get_uinst();
    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
    void offclock_pulse(uinst_t ui, bool io_done);
};

}

#endif