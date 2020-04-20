#include "iodev_manager.hpp"
#include "../../common.hpp"

namespace kcpu {

io_device_manager::io_device_manager(vm_logger &logger) : logger(logger), state(state::IDLE) {
}

void io_device_manager::register_iodev(io_device &d) {
    std::vector<regval_t> l = d.get_reserved_ports();
    for(regval_t port : l) {
        vm_assert(ports.find(port) == ports.end());
        ports.emplace(port, d);
    }

    devices.push_back(d);
}

const std::unordered_map<regval_t, std::reference_wrapper<io_device>> & io_device_manager::get_ports() {
    return ports;
}

void io_device_manager::dump_registers() {
    logger.logf("IO: ");
    switch(state) {
        case state::IDLE: {
            logger.logf("IDLE");
            break;
        }
        case state::ONGOING_READ: {
            logger.logf("ONGOING_READ (%d, 0x%X)\n", halfcycles_remaining, read_result);
            break;
        }
        case state::ONGOING_WRITE: {
            logger.logf("ONGOING_WRITE (%d, 0x%X)\n", halfcycles_remaining, read_result);
            break;
        }
        case state::PRESENTING_READ: {
            logger.logf("PRESENTING_READ (0x%X)\n", read_result);
            break;
        }
        case state::PRESENTING_WRITE: {
            logger.logf("PRESENTING_WRITE\n");
            break;
        }
        default: throw vm_error("unknown io state");
    }
    logger.logf("\n");
}

bool io_device_manager::is_io_done() {
    return state == state::RETURNING || state == state::PRESENTING_READ || state == state::PRESENTING_WRITE;
}

regval_t io_device_manager::get_read_result() {
    vm_assert(state == state::PRESENTING_READ);
    return read_result;
}

io_device & io_device_manager::get_device(regval_t port) {
    auto val = ports.find(port);
    if(val == ports.end()) {
        std::stringstream ss;
        ss << "command to floating port: 0x" << std::hex << std::uppercase << port;
        throw vm_error(ss.str());
    }
    return val->second;
}

void io_device_manager::before_clock_outputs_read(regval_t port) {
    switch(state) {
        case state::IDLE: {
            auto r = get_device(port).read(port);
            read_result = r.first;
            halfcycles_remaining = r.second;

            state = state::ONGOING_READ;
            ongoing_port = port;

            if(logger.dump_bus) logger.logf("io read(0x%X) starting, %d hcycles remaining\n", port, halfcycles_remaining);

            break;
        }
        case state::ONGOING_READ:
        case state::PRESENTING_READ: {
            vm_assert(port == ongoing_port);
            break;
        }
        default: throw vm_error("unacceptable before_clock_outputs_read state");
    }
}

void io_device_manager::before_clock_outputs_write(regval_t port, regval_t val) {
    switch(state) {
        case state::IDLE: {
            halfcycles_remaining = get_device(port).write(port, val);

            state = state::ONGOING_WRITE;
            ongoing_port = port;

            if(logger.dump_bus) logger.logf("io write(0x%X) starting, %d hcycles remaining\n", port, halfcycles_remaining);

            break;
        }
        case state::ONGOING_WRITE:
        case state::PRESENTING_WRITE: {
            vm_assert(port == ongoing_port);
            break;
        }
        default: throw vm_error("unacceptable before_clock_outputs_read state");
    }
}

void io_device_manager::after_clock_outputs_none() {
    switch(state) {
        case state::IDLE: {
            break;
        }
        default: throw vm_error("unacceptable after_clock_outputs_none state");
    }
}

void io_device_manager::after_clock_outputs_read() {
    switch(state) {
        case state::IDLE:
        case state::ONGOING_READ: {
            break;
        }
        case state::PRESENTING_READ: {
            if(logger.dump_bus) logger.logf("io read ending presentation\n");
            state = state::RETURNING;
            break;
        }
        default: throw vm_error("unacceptable after_clock_outputs_read state");
    }
}

void io_device_manager::after_clock_outputs_write() {
    switch(state) {
        case state::IDLE:
        case state::ONGOING_WRITE: {
            break;
        }
        case state::PRESENTING_WRITE: {
            if(logger.dump_bus) logger.logf("io write ending presentation\n");
            state = state::RETURNING;
            break;
        }
        default: throw vm_error("unacceptable after_clock_outputs_write state");
    }
}

void io_device_manager::process_halfcycle(bool offclock) {
    for(auto dev : devices) {
        dev.get().process_halfcycle(offclock);
    }

    switch(state) {
        case state::IDLE:
        case state::PRESENTING_READ:
        case state::PRESENTING_WRITE: {
            break;
        }
        case state::RETURNING: {
            if(offclock) {
                if(logger.dump_bus) logger.logf("io resetting io_done\n");
                state = state::IDLE;
            }
            break;
        }
        case state::ONGOING_READ:
        case state::ONGOING_WRITE: {
            vm_assert(halfcycles_remaining >= 0);

            if(halfcycles_remaining > 0) {
                halfcycles_remaining--;
                if(logger.dump_bus) logger.logf("io op ongoing, %d hcycles remaining\n", halfcycles_remaining);
                break;
            }

            if(halfcycles_remaining == 0) {
                if(state == state::ONGOING_READ) {
                    if(logger.dump_bus) logger.logf("io read now presenting, value 0x%X\n", read_result);
                    state = state::PRESENTING_READ;
                }

                if(state == state::ONGOING_WRITE) {
                    if(logger.dump_bus) logger.logf("io write now presenting\n");
                    state = state::PRESENTING_WRITE;
                }
            }

            break;
        }
        default: throw vm_error("unknown io state");
    }
}

}