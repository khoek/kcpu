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

/* An `opclass` represents one of:
    1. (NO_IU3) a single opcode which ignores IU3.
    2. (IU3_SINGLE) a single opcode which cares about IU3.
    3. (IU3_ALL) a range of opcodes for which IU3 represents a third argument.

   In principle more complicated patterns can be supported.
*/
class opclass {
    public:
    enum kind {
        NO_IU3,
        IU3_SINGLE,
        IU3_ALL,
    };

    regval_t raw;
    kind cls;
    preg_t iu3;

    opclass(regval_t raw);
    opclass(regval_t raw, kind k, preg_t iu3);

    regval_t resolve();
    regval_t resolve(preg_t r);
    regval_t resolve_dummy();
};

opclass opclass_iu3_single(regval_t raw, preg_t iu3);
opclass opclass_iu3_all(regval_t raw);

class instruction {
    private:
    void check_valid();

    public:
    std::string name;
    opclass op;
    argtype args;
    std::vector<uinst_t> uis;

    instruction(std::string name, opclass op, argtype args, std::vector<uinst_t> uis);
    instruction(std::string name, opclass op, argtype args, uinst_t ui);
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

class virtual_instruction {
    public:
    opclass op;
    std::vector<slot> bi;

    virtual_instruction(opclass op, std::vector<slot> bi);
    virtual_instruction(opclass op, argtype args);

    regval_t build_inst(bool loaddata, std::vector<preg_t> ius);
};

class alias {
    public:
    std::string name;
    argtype args;
    std::vector<virtual_instruction> insts;

    alias(std::string name, argtype args, std::vector<virtual_instruction> insts);
    alias(std::string name, argtype args, virtual_instruction inst);
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
