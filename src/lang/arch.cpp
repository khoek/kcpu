
#include <sstream>
#include <iterator>

#include "../spec/inst.hpp"
#include "../spec/ucode.hpp"
#include "arch.hpp"

namespace kcpu {

arch_error::arch_error(const std::string &msg) : bt_error(msg) { }

static uint16_t uaddr(regval_t inst, ucval_t uc) {
    if(uc > UCVAL_MAX) {
        throw arch_error("uc too great");
    }

    if(inst > INST_MAX) {
        throw arch_error("inst too great");
    }

    return (inst << UCVAL_WIDTH) | uc;
}

slot slot_reg(preg_t reg) {
    return (slot) {.kind = slot::SLOT_REG, .val = { .reg = reg } };
}

slot slot_arg(uint8_t argidx) {
    return (slot) {.kind = slot::SLOT_ARG, .val = { .argidx = argidx } };
}

slot slot_constval(regval_t constval) {
    return (slot) {.kind = slot::SLOT_CONSTVAL, .val = { .constval = constval } };
}

static std::vector<slot> get_slots(argtype args) {
    std::vector<slot> bi;
    for(int i = 0; i < args.count; i++) {
        bi.push_back(slot_arg(i));
    }
    return bi;
}

static void check_opcode_supports_argcount(opclass op, uint8_t argcount) {
    switch(op.cls) {
        case opclass::NO_IU3: {
            assert(argcount < 3);
            break;
        }
        case opclass::IU3_SINGLE: {
            assert(argcount == 2);
            break;
        }
        case opclass::IU3_ALL: {
            assert(argcount == 3);
            break;
        }
        default: throw arch_error("unknown opclass");
    }
}

virtual_instruction::virtual_instruction(opclass op, std::vector<slot> bi) : op(op), bi(bi) {
    check_opcode_supports_argcount(op, bi.size());
}

virtual_instruction::virtual_instruction(opclass op, argtype args) : virtual_instruction(op, get_slots(args)) { }

regval_t virtual_instruction::build_inst(bool loaddata, std::vector<preg_t> ius) {
    assert(bi.size() == ius.size());

    regval_t inst = ((loaddata) ? P_I_LOADDATA : 0) | (op.raw << INST_SHIFT);
    switch(bi.size()) {
        case 3: inst |= INST_MK_IU3(ius[2]);
        case 2: inst |= INST_MK_IU2(ius[1]);
        case 1: inst |= INST_MK_IU1(ius[0]);
        case 0: break;
        default: throw arch_error("too many args!");
    }

    if(op.cls == opclass::IU3_SINGLE) {
        inst |= INST_MK_IU3(op.iu3);
    }

    return inst;
}

alias::alias(std::string name, argtype args, std::vector<virtual_instruction> insts)
    : name(name), args(args), insts(insts) { }

alias::alias(std::string name, argtype args, virtual_instruction inst)
    : name(name), args(args), insts({inst}) { }

parameter::parameter(kind type, bool noconst, bool byteconst)
    : type(type), noconst(noconst), byteconst(byteconst) { }

bool parameter::accepts(parameter::kind other) {
    switch(type) {
        case parameter::PARAM_WREG:  return (other == parameter::PARAM_WREG ) || (!noconst && other == parameter::PARAM_CONST);
        case parameter::PARAM_BLREG: return (other == parameter::PARAM_BLREG) || (!noconst && other == parameter::PARAM_CONST);
        case parameter::PARAM_BHREG: return (other == parameter::PARAM_BHREG) || (!noconst && other == parameter::PARAM_CONST);
        case parameter::PARAM_CONST: return  other == parameter::PARAM_CONST;
        default: throw arch_error("unknown parameter type");
    }
}

parameter param_wreg() {
    return parameter(parameter::PARAM_WREG, false, false);
}

parameter param_wreg_noconst() {
    return parameter(parameter::PARAM_WREG, true, false);
}

parameter param_breg_lo() {
    return parameter(parameter::PARAM_BLREG, false, true);
}

parameter param_breg_lo_noconst() {
    return parameter(parameter::PARAM_BLREG, true, true);
}

parameter param_breg_hi() {
    return parameter(parameter::PARAM_BHREG, false, true);
}

parameter param_breg_hi_noconst() {
    return parameter(parameter::PARAM_BHREG, true, true);
}

parameter param_wconst(bool noconst) {
    return parameter(parameter::PARAM_CONST, false, false);
}

parameter param_bconst(bool noconst) {
    return parameter(parameter::PARAM_CONST, false, true );
}

std::vector<parameter> argtype_to_param_list(argtype args) {
    std::vector<parameter> params;
    for(int j = 0; j < args.count; j++) {
        // FIXME change what argtypes are, so we get nice protections on the standard instructions as well.
        params.push_back(parameter(parameter::PARAM_WREG, j != args.maybeconst, false));
    }
    return params;
}

family::mapping::mapping(std::string name, std::vector<parameter> value)
    : name(name), params(value) { }

family::family(std::string name, std::vector<family::mapping> mappings)
    : name(name), mappings(mappings) { }

std::optional<std::string> family::match(std::vector<parameter::kind> params) {
    for(auto ps = mappings.begin(); ps < mappings.end(); ps++) {
        if(ps->params.size() != params.size()) {
            continue;
        }

        bool fail = false;
        for(int j = 0; j < ps->params.size(); j++) {
            if(!ps->params[j].accepts(params[j])) {
                fail = true;
                break;
            }
        }

        if(!fail) {
            return ps->name;
        }
    }
    return std::nullopt;
}

void instruction::check_valid() {
    check_opcode_supports_argcount(op, args.count);

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

        if(uis[i] & MASK_RCTRL_IU3) {
            if(op.cls == opclass::NO_IU3) {
                std::stringstream ss;
                ss << "ucode for instruction " << name << " refers to IU3 but does not declare this in the opcode, at position " << i << "/" << uis.size();
                throw ss.str();
            }
        }
    }
}

static std::vector<uinst_t> invert_active_low_bits(std::vector<uinst_t> uis) {
    std::vector<uinst_t> fixed_uis;
    for(auto ui = uis.begin(); ui < uis.end(); ui++) {
        fixed_uis.push_back(*ui ^ MASK_I_INVERT);
    }
    return fixed_uis;
}

instruction::instruction(std::string name, opclass op, argtype args, std::vector<uinst_t> uis)
    : name(name), op(op), args(args), uis(invert_active_low_bits(uis)) {
    check_valid();
}

instruction::instruction(std::string name, opclass op, argtype args, uinst_t ui)
    : instruction(name, op, args, std::vector<uinst_t>{ui}) { }

template <typename Out>
static void split(const std::string &s, char delim, Out result) {
    std::istringstream iss(s);
    std::string item;
    while (std::getline(iss, item, delim)) {
        *result++ = item;
    }
}

static std::vector<std::string> split(const std::string &s, char delim) {
    std::vector<std::string> elems;
    split(s, delim, std::back_inserter(elems));
    return elems;
}

arch::arch() {
}

uinst_t arch::ucode_read(regval_t inst, ucval_t uc) {
    return ucode[uaddr((inst & ~P_I_LOADDATA) >> INST_SHIFT, uc)];
}

bool arch::inst_is_prefix(std::string str) {
    return prefixes.find(str) != prefixes.end();
}

std::optional<family> arch::lookup_family(std::string name) {
    auto r = families.find(name);
    return r == families.end() ? std::nullopt : std::optional(r->second);
}

std::optional<alias> arch::lookup_alias(std::string name) {
    auto r = aliases.find(name);
    return r == aliases.end() ? std::nullopt : std::optional(r->second);
}

std::optional<instruction> arch::lookup_inst(regval_t opcode) {
    auto r = insts.find(opcode);
    return r == insts.end() ? std::nullopt : std::optional(r->second);
}

void arch::reg_opcode(regval_t opcode, instruction i) {
    if(ucode_inst[opcode]) {
        throw arch_error("opcode collision: " + ucode_inst[opcode]->name + ", " + i.name);
    }

    ucode_inst[opcode] = i;
    insts.emplace(opcode, i);

    for(std::size_t uc = 0; uc < i.uis.size(); uc++) {
        uint16_t ua = uaddr(opcode, uc);
        if(ucode[ua]) {
            throw arch_error("ucode collision: " + ucode_name[ua] + ", " + i.name);
        }

        ucode[ua] = i.uis[uc];
        ucode_name[ua] = i.name;
    }
}

void arch::reg_inst(instruction i) {
    switch(i.op.cls) {
        case opclass::NO_IU3:
        case opclass::IU3_SINGLE: {
            reg_opcode(i.op.resolve(), i);
            break;
        }
        case opclass::IU3_ALL: {
            for(int j = 0; j < NUM_PREGS; j++) {
                reg_opcode(i.op.resolve((preg_t) j), i);
            }
            break;
        }
        default: throw arch_error("unknown opclass!");
    }

    // This must happen last in order not to trip up the sanity checker
    // in reg_alias().
    reg_alias(alias(i.name, i.args, { virtual_instruction(i.op, i.args) }));
}

void arch::reg_alias(alias a) {
    // In this loop we just do some consistency checks between the registered opcodes/argurments and their true arguments.
    for(auto j = a.insts.begin(); j < a.insts.end(); j++) {
        std::optional<instruction> i = ucode_inst[j->op.resolve_dummy()];
        if(!i) {
            throw arch_error("alias " + a.name + " registers an unknown opcode");
        }

        if(i->args.count != j->bi.size()) {
            throw arch_error("alias " + a.name + " has the wrong number of arguments for instruction " + i->name);
        }

        int const_count = 0;
        for(int k = 0; k < i->args.count; k++) {
            if(j->bi[k].kind == slot::SLOT_CONSTVAL) {
                const_count++;
            }

            if(j->bi[k].kind == slot::SLOT_ARG && j->bi[k].val.argidx >= a.args.count) {
                throw arch_error("alias " + a.name + " uses a non-existent (too large) arg index for isntruction " + i->name);
            }
        }

        if(const_count > 1) {
            throw arch_error("alias " + a.name + " uses multiple const args in the same expression for instruction " + i->name);
        }

        for(int k = 0; k < i->args.count; k++) {
            if (i->args.maybeconst != k) {
                    if(a.args.maybeconst >= 0 && j->bi[k].kind == slot::SLOT_ARG && j->bi[k].val.argidx == a.args.maybeconst) {
                        throw arch_error("alias " + a.name + " might (depending on argument) bind a constant to a nonstandard argument for instruction " + i->name
                            + ", and this is probably an accident.\n(Maybe we will want this in the future?)");
                    }

                    if(j->bi[k].kind == slot::SLOT_CONSTVAL) {
                        throw arch_error("alias " + a.name + " binds a constant to a nonstandard argument for instruction " + i->name
                            + ", and this is probably an accident.\n(Maybe we will want this in the future?)");
                    }
            }
        }
    }

    if(aliases.find(a.name) != aliases.end()) {
        throw arch_error("alias " + a.name + " name collision");
    }

    // Actually register the alias.
    aliases.emplace(a.name, a);

    // This must happen last in order not to trip up the sanity checker
    // in reg_family().
    reg_family(family(a.name, { family::mapping(a.name, argtype_to_param_list(a.args)) }));
}

void arch::reg_family(family f) {
    // In this loop we just do some consistency checks between the registered aliases/argurments and their true arguments.
    for(auto m = f.mappings.begin(); m < f.mappings.end(); m++) {
        std::optional<alias> a = lookup_alias(m->name);
        if(!a) {
            throw arch_error("family " + f.name + " registers unknown alias " + m->name);
        }

        if(a->args.count != m->params.size()) {
            std::stringstream ss;
            ss << "family " << f.name << " registers wrong number of arguments (" << m->params.size() << ")"
                 << " for alias " << m->name << " (" << ((uint32_t) a->args.count) << ")";
            throw arch_error(ss.str());
        }
    }

    if(families.find(f.name) != families.end()) {
        throw arch_error("family " + f.name + " name collision");
    }

    // Actually try to register the family.
    std::vector<std::string> tks = split(f.name, ' ');
    if(tks.size() > 2) {
        throw arch_error("too many spaces in name!");
    }

    if(tks.size() == 2) {
        prefixes.emplace(tks[0]);
    }

    families.emplace(f.name, f);
}

template<typename the_map>
static std::vector<typename the_map::mapped_type> get_map_values(const the_map &m) {
    std::vector<typename the_map::mapped_type> r;
    r.reserve(m.size());
    for (const auto &kvp : m) {
        r.push_back(kvp.second);
    }
    return r;
}

std::vector<family> arch::list_families() {
    return get_map_values(families);
}

std::vector<alias> arch::list_aliases(){
    return get_map_values(aliases);
}

std::vector<instruction> arch::list_insts() {
    return get_map_values(insts);
}

arch & arch::self() {
    static arch instance;
    static bool init = false;
    if(!init) {
        init = true;
        internal::register_insts();
        internal::register_aliases();
        internal::register_families();
    }
    return instance;
}

}
