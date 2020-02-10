#ifndef GEN_ARCH_H
#define GEN_ARCH_H

#include <vector>
#include <string>
#include <optional>
#include "../types.h"
#include "../spec/hw.h"

struct argtype {
    uint8_t count;
    int8_t maybeconst;
};

#define ARGS_0         ((argtype) {.count = 0, .maybeconst =  0})
#define ARGS_1_NOCONST ((argtype) {.count = 1, .maybeconst = -1})
#define ARGS_1         ((argtype) {.count = 1, .maybeconst =  0})
#define ARGS_2_NOCONST ((argtype) {.count = 2, .maybeconst = -1})
#define ARGS_2_1CONST  ((argtype) {.count = 2, .maybeconst =  0})
#define ARGS_2_2CONST  ((argtype) {.count = 2, .maybeconst =  1})
#define ARGS_3_NOCONST ((argtype) {.count = 3, .maybeconst = -1})
#define ARGS_3_1CONST  ((argtype) {.count = 3, .maybeconst =  0})
#define ARGS_3_2CONST  ((argtype) {.count = 3, .maybeconst =  1})
#define ARGS_3_3CONST  ((argtype) {.count = 3, .maybeconst =  2})

class instruction {
    public:
    std::string name;
    regval_t opcode;
    argtype args;
    std::vector<uinst_t> uis;

    instruction(std::string name, regval_t opcode, argtype args, std::vector<uinst_t> uis);
    instruction(std::string name, regval_t opcode, argtype args, uinst_t ui);
};

struct slot {
    enum {
        SLOT_REG,
        SLOT_ARG,
        SLOT_CONSTVAL
    } kind;
    union {
        preg_t reg;
        uint8_t argidx;
        regval_t constval;
    } val;
};

slot slot_reg(preg_t reg);
slot slot_arg(uint8_t argidx);
slot slot_constval(regval_t constval);

class unbound_opcode {
    public:
    regval_t raw;
    std::vector<slot> bi;

    unbound_opcode(regval_t raw, std::vector<slot> bi);
    unbound_opcode(regval_t raw, argtype args);
};

class alias {
    public:
    std::string name;
    argtype args;
    std::vector<unbound_opcode> insts;

    alias(std::string name, argtype args, std::vector<unbound_opcode> insts);
    alias(std::string name, argtype args, unbound_opcode inst);
};

namespace arch {
    void reg_inst(instruction i);
    void reg_alias(alias a);
};

void init_arch();

uinst_t ucode_lookup(regval_t inst, ucval_t uc);

bool inst_is_prefix(std::string str);
std::optional<alias> alias_lookup(std::string name);
std::optional<instruction> inst_lookup(regval_t opcode);

#endif
