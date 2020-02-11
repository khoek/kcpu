#ifndef LANG_LANG_H
#define LANG_LANG_H

#include <vector>
#include <string>
#include <optional>
#include <cassert>
#include <unordered_set>
#include <unordered_map>

#include "../types.h"
#include "../except.h"
#include "../spec/hw.h"
#include "../spec/inst.h"

namespace kcpu {

class lang_error : public bt_error {
    public:
    lang_error(const std::string &arg);
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

class arch {
    private:
    uinst_t ucode[UCODE_LEN];
    std::string ucode_name[UCODE_LEN];
    std::optional<instruction> ucode_inst[OPCODE_LEN];

    std::unordered_set<std::string> prefixes;
    std::unordered_map<std::string, alias> aliases;
    std::unordered_map<regval_t, instruction> insts;

    void reg_opcode(regval_t opcode, instruction i);

    public:
    arch();
    void reg_inst(instruction i);
    void reg_alias(alias a);

    uinst_t ucode_lookup(regval_t inst, ucval_t uc);

    bool inst_is_prefix(std::string str);
    std::optional<alias> alias_lookup(std::string name);
    std::optional<instruction> inst_lookup(regval_t opcode);

    static arch & self();
};

}

#endif
