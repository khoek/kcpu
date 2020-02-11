#include <cstdlib>
#include <cstdio>
#include <cstring>
#include <cassert>
#include <unordered_set>
#include <unordered_map>
#include <sstream>
#include <iterator>

#include "../spec/inst.h"
#include "../spec/ucode.h"
#include "arch.h"
#include "insts.h"
#include "alias.h"

static std::string ucode_name[UCODE_LEN];
static std::optional<instruction> ucode_inst[OPCODE_LEN];
static uinst_t ucode[UCODE_LEN];

static std::unordered_set<std::string> prefixes;
static std::unordered_map<std::string, alias> aliases;
static std::unordered_map<regval_t, instruction> insts;

static uint16_t uaddr(regval_t inst, ucval_t uc) {
    if(uc > UCVAL_MAX) {
        throw "uc too great";
    }

    if(inst > INST_MAX) {
        throw "inst too great";
    }
    
    return (inst << UCVAL_WIDTH) | uc;
}

uinst_t ucode_lookup(regval_t inst, ucval_t uc) {
    return ucode[uaddr((inst & ~P_I_LOADDATA) >> INST_SHIFT, uc)];
}

bool inst_is_prefix(std::string str) {
    return prefixes.find(str) != prefixes.end();
}

std::optional<alias> alias_lookup(std::string name) {
    auto r = aliases.find(name);
    return r == aliases.end() ? std::nullopt : std::optional(r->second);
}

std::optional<instruction> inst_lookup(regval_t opcode) {
    auto r = insts.find(opcode);
    return r == insts.end() ? std::nullopt : std::optional(r->second);
}

slot slot_reg(preg_t reg) {
    return {.kind = slot::SLOT_REG, .val = { .reg = reg } };
}

slot slot_arg(uint8_t argidx) {
    return {.kind = slot::SLOT_ARG, .val = { .argidx = argidx } };
}

slot slot_constval(regval_t constval) {
    return {.kind = slot::SLOT_CONSTVAL, .val = { .constval = constval } };
}

unbound_opcode::unbound_opcode(regval_t raw, std::vector<slot> bi) : raw(raw), bi(bi) {}

unbound_opcode::unbound_opcode(regval_t raw, argtype args) : raw(raw), bi() {
    for(int i = 0; i < args.count; i++) {
        bi.push_back(slot_arg(i));
    }
}

alias::alias(std::string name, argtype args, std::vector<unbound_opcode> insts)
    : name(name), args(args), insts(insts) { }

alias::alias(std::string name, argtype args, unbound_opcode inst)
    : name(name), args(args), insts({inst}) { }

static void check_ucode(std::string name, std::vector<uinst_t> uis) {
    if(uis.size() > UCODE_LEN) {
        std::stringstream ss;
        ss << "ucode for instruction " << name << " too long (" << uis.size() << " > " << UCODE_LEN << ")";
        throw ss.str();
    }
    
    for(int i = 0; i < uis.size(); i++) {
        if((uis[i] & MASK_GCTRL_FTJM) == GCTRL_FT_ENTER) {
            if(name != "NOP" && i + 1 != uis.size()) {
                std::stringstream ss;
                ss << "ucode for instruction " << name << " has dangling GCTRL_FT_ENTER at position " << i << "/" << uis.size();
                throw ss.str();
            }
        }
    }
}

instruction::instruction(std::string name, regval_t opcode, argtype args, std::vector<uinst_t> uis)
    : name(name), opcode(opcode), args(args), uis(uis) {
    check_ucode(name, uis);
}

instruction::instruction(std::string name, regval_t opcode, argtype args, uinst_t ui)
    : instruction(name, opcode, args, std::vector<uinst_t>{ui}) {}

template <typename Out>
void split(const std::string &s, char delim, Out result) {
    std::istringstream iss(s);
    std::string item;
    while (std::getline(iss, item, delim)) {
        *result++ = item;
    }
}

std::vector<std::string> split(const std::string &s, char delim) {
    std::vector<std::string> elems;
    split(s, delim, std::back_inserter(elems));
    return elems;
}

void arch::reg_inst(instruction i) {
    if(i.opcode & P_I_LOADDATA) {
        throw "instruction " + i.name + " has LOADDATA bit set!";
    }

    if(ucode_inst[i.opcode]) {
        throw "opcode collision: " + ucode_inst[i.opcode]->name + ", " + i.name;
    }

    ucode_inst[i.opcode] = i;
    insts.emplace(i.opcode, i);

    for(std::size_t uc = 0; uc < i.uis.size(); uc++) {
        uint16_t ua = uaddr(i.opcode, uc);
        if(ucode[ua]) {
            throw "ucode collision: " + ucode_name[ua] + ", " + i.name;
        }
        
        ucode[ua] = i.uis[uc];
        ucode_name[ua] = i.name;
    }

    // This must happen last in order not to trip up the sanity checker
    // in reg_alias.
    reg_alias(alias(i.name, i.args, { unbound_opcode(i.opcode, i.args) }));
}

void arch::reg_alias(alias a) {
    // In this loop we just do some consistency checks.
    for(auto j = a.insts.begin(); j < a.insts.end(); j++) {
        std::optional<instruction> i = ucode_inst[j->raw];
        if(!i) {
            throw "alias " + a.name + " registers an unknown opcode";
        }

        if(i->args.count != j->bi.size()) {
            throw "alias " + a.name + " has the wrong number of arguments for instruction " + i->name;
        }

        int const_count = 0;
        for(int k = 0; k < i->args.count; k++) {
            if(j->bi[k].kind == slot::SLOT_CONSTVAL) {
                const_count++;
            }

            if(j->bi[k].kind == slot::SLOT_ARG && j->bi[k].val.argidx >= a.args.count) {
                throw "alias " + a.name + " uses a non-existent (too large) arg index for isntruction " + i->name;
            }
        }

        if(const_count > 1) {
            throw "alias " + a.name + " uses multiple const args in the same expression for instruction " + i->name;
        }

        for(int k = 0; k < i->args.count; k++) {
            if (i->args.maybeconst != k) {
                    if(a.args.maybeconst >= 0 && j->bi[k].kind == slot::SLOT_ARG && j->bi[k].val.argidx == a.args.maybeconst) {
                        throw "alias " + a.name + " might (depending on argument) bind a constant to a nonstandard argument for instruction " + i->name
                            + ", and this is probably an accident.\n(Maybe we will want this in the future?)";
                    }
                    
                    if(j->bi[k].kind == slot::SLOT_CONSTVAL) {
                        throw "alias " + a.name + " binds a constant to a nonstandard argument for instruction " + i->name
                            + ", and this is probably an accident.\n(Maybe we will want this in the future?)";
                    }
            }
        }
    }

    if(aliases.find(a.name) != aliases.end()) {
        throw "alias " + a.name + " name collision";
    }

    // Actually try to register the alias.
    std::vector<std::string> tks = split(a.name, ' ');
    if(tks.size() > 2) {
        throw "too many spaces in name!";
    }

    if(tks.size() == 2) {
        prefixes.emplace(tks[0]);
    }

    aliases.emplace(a.name, a);
}

void init_arch() {
    register_insts();
    register_aliases();
}

static void write_ucode() {
    // TODO this.

    throw "do this";
}
