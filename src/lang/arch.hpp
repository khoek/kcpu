#ifndef LANG_LANG_H
#define LANG_LANG_H

#include <vector>
#include <string>
#include <optional>
#include <cassert>
#include <unordered_set>
#include <unordered_map>

#include "../types.hpp"
#include "../except.hpp"
#include "../spec/hw.hpp"
#include "../spec/inst.hpp"

namespace kcpu {

class arch_error : public bt_error {
    public:
    arch_error(const std::string &arg);
};

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

class parameter {
    public:
    enum kind {
        PARAM_WREG,
        PARAM_BLREG,
        PARAM_BHREG,
        PARAM_CONST,
    };

    kind type;
    bool noconst;
    bool byteconst;

    parameter(kind type, bool noconst, bool byteconst);

    bool accepts(kind other);
};

parameter param_wreg();
parameter param_wreg_noconst();
parameter param_breg_lo();
parameter param_breg_lo_noconst();
parameter param_breg_hi();
parameter param_breg_hi_noconst();
parameter param_wconst();
parameter param_bconst();

std::vector<parameter> argtype_to_param_list(argtype args);

class family {
    public:
    class mapping {
        public:
        std::string name;
        std::vector<parameter> params;

        mapping(std::string name, std::vector<parameter> value);
    };

    std::string name;
    std::vector<mapping> mappings;

    family(std::string name, std::vector<mapping> mappings);

    std::optional<std::string> match(std::vector<parameter::kind> params);
};

class arch {
    private:
    uinst_t ucode[UCODE_LEN];
    std::string ucode_name[UCODE_LEN];
    std::optional<instruction> ucode_inst[OPCODE_LEN];

    std::unordered_set<std::string> prefixes;
    std::unordered_map<std::string, alias> aliases;
    std::unordered_map<std::string, family> families;
    std::unordered_map<regval_t, instruction> insts;

    void reg_opcode(regval_t opcode, instruction i);

    public:
    arch();
    void reg_inst(instruction i);
    void reg_alias(alias a);
    void reg_family(family f);

    uinst_t ucode_read(regval_t inst, ucval_t uc);

    bool inst_is_prefix(std::string str);
    std::optional<family> lookup_family(std::string name);
    std::optional<alias> lookup_alias(std::string name);
    std::optional<instruction> lookup_inst(regval_t opcode);

    static arch & self();
};

namespace internal {

void register_insts();
void register_aliases();
void register_families();

}

}

#endif
