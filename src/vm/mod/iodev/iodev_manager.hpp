#ifndef VM_MOD_IODEV_MANAGER_H
#define VM_MOD_IODEV_MANAGER_H

#include <vector>
#include <unordered_map>
#include <functional>

#include "../../common.hpp"
#include "iodev.hpp"

// This class is not a module. It just simulates external IO device hardware
// attached over the IO bus.

namespace kcpu {

class io_device_manager {
    private:
    vm_logger &logger;

    enum state {
        IDLE,
// State which means that we have presented during a clock rising
// edge, but that IO_DONE should not go low until a clock falling edge.
        RETURNING,
        ONGOING_READ,
        ONGOING_WRITE,
        PRESENTING_READ,
        PRESENTING_WRITE,
    };

    std::vector<std::reference_wrapper<io_device>> devices;
    std::unordered_map<regval_t, std::reference_wrapper<io_device>> ports;

    regval_t ongoing_port;
    regval_t read_result;
    halfcycle_count_t halfcycles_remaining;

    state state;

    io_device & get_device(regval_t port);

    public:
    io_device_manager(vm_logger &logger);
    void register_iodev(io_device &d);
    const std::unordered_map<regval_t, std::reference_wrapper<io_device>> & get_ports();
    void dump_registers();

    bool is_io_done();
    regval_t get_read_result();

    void before_clock_outputs_read(regval_t port);
    void before_clock_outputs_write(regval_t port, regval_t val);
    void after_clock_outputs_none();
    void after_clock_outputs_read();
    void after_clock_outputs_write();
    void process_halfcycle(bool offclock);
};

}

#endif