#ifndef VM_COMMON_H
#define VM_COMMON_H

#include <sstream>
#include "../types.hpp"
#include "../except.hpp"
#include "../spec/hw.hpp"

namespace kcpu {

class vm_error : public bt_error {
    public:
    vm_error(const std::string &arg);
};

static inline void vm_assert_raw(bool cond, const char *file, int line) {
    if(!cond) {
        std::stringstream ss;
        ss << "assertion failed! " << file << ":" << line;
        throw vm_error(ss.str());
    }
}

#define vm_assert(cond) vm_assert_raw((cond), __FILE__, __LINE__)

class vm_logger {
    public:
    bool disassemble;
    bool dump_registers;
    bool dump_bus;

    vm_logger(bool disassemble = false, bool dump_registers = false, bool dump_bus = false);

    void logf(const char *fmt, ...) __attribute__((format (printf, 2, 3)));
    void logf(const std::string &str);
};

extern const char * BUS_NAMES[];

#define BUS_DEFAULT_VAL 0x0

class bus_state {
    private:
    vm_logger &logger;

    bool frozen;
    bool set[NUM_BUSES];
    regval_t bus[NUM_BUSES];

    regval_t get_unset_value(bus_t b);

    public:
    bus_state(vm_logger &logger);

    void freeze();
    void assign(bus_t b, regval_t val);
    bool is_assigned(bus_t b);
    void connect(bus_t b1, bus_t b2);
    regval_t early_read(bus_t b);
    regval_t read(bus_t b);
};

}

#endif
