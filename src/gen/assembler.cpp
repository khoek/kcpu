#include <optional>
#include <vector>
#include <sstream>
#include <iostream>
#include <unordered_map>

#include "../spec/inst.h"
#include "../lang/arch.h"
#include "assembler.h"

namespace kcpu {

static std::string format_line_msg(uint32_t line, const std::string &msg) {
    std::stringstream ss;
    ss << "Line " << line << ": " << msg;
    return ss.str();
}

assembler::parse_error::parse_error(uint32_t line, const std::string &msg) : bt_error(format_line_msg(line, msg)) { }

assembler::internal_error::internal_error(uint32_t line, const std::string &msg) : bt_error(format_line_msg(line, msg)) { }

// FIXME make sure labels don't collide with register names

class chunk {
    public:
    bool concrete;
    bool label_def;
    regval_t val;
    std::string label;

    chunk(regval_t val);
    chunk(std::string label, bool label_def);
};

chunk::chunk(regval_t raw) : concrete(true), val(raw) { }

chunk::chunk(std::string label, bool label_def = false) : concrete(false), label(label), label_def(label_def) { }

class bound_parameter {
    public:
    parameter::kind type;
    preg_t reg;
    bool lo_or_hi;
    std::optional<chunk> constval;

    bound_parameter(parameter::kind type, preg_t reg, bool lo_or_hi);
    bound_parameter(chunk constval);
};

bound_parameter::bound_parameter(parameter::kind type, preg_t reg, bool lo_or_hi)
    : type(type), reg(reg), lo_or_hi(lo_or_hi), constval(std::nullopt) { }

bound_parameter::bound_parameter(chunk constval)
    : type(parameter::PARAM_CONST), reg(kcpu::REG_ID), constval(constval) { }

class inst_assembler {
    private:
    std::istream &in;
    std::vector<chunk> &buff;
    uint32_t line;

    [[noreturn]]
    void throw_parse_error(std::string msg);

    [[noreturn]]
    void throw_internal_error(std::string msg);

    std::optional<bound_parameter> lookup_reg(std::string s);

    std::optional<bound_parameter> parse_param();
    void handle_label(std::string &tk);
    void handle_instruction(std::string &tk);
    void bind_virtual(virtual_instruction uo, std::vector<bound_parameter> ai);

    public:
    inst_assembler(std::istream &in, std::vector<chunk> &buff, uint32_t line);
    void parse();
};

inst_assembler::inst_assembler(std::istream &in, std::vector<chunk> &buff, uint32_t line) : in(in), buff(buff), line(line) { }

void inst_assembler::throw_parse_error(std::string msg) {
    throw assembler::parse_error(line, msg);
}

void inst_assembler::throw_internal_error(std::string msg) {
    throw assembler::internal_error(line, msg);
}

std::optional<bound_parameter> inst_assembler::lookup_reg(std::string s) {
    if(s.size() == 0) {
        return std::nullopt;
    }
    
    if(s[0] != '%') {
        return std::nullopt;
    }

    parameter::kind type;
    bool lo_or_hi;
    if(s[1] == 'r') {
        type = parameter::PARAM_WREG;
        lo_or_hi = false;
    } else if(s[1] == 'l') {
        type = parameter::PARAM_BLREG;
        lo_or_hi = false;
    } else if(s[1] == 'h') {
        type = parameter::PARAM_BHREG;
        lo_or_hi = true;
    } else {
        std::stringstream ss;
        ss << "unknown register prefix " << s[1] << " in " << s;
        throw_parse_error(ss.str());
    }

    std::string trunc = s.substr(2);
    for(int i = 0; i < NUM_PREGS; i++) {
        if(trunc == PREG_NAMES[i]) {
            switch(i) {
                case REG_ID: throw_parse_error("cannot refer to REG_ID!");
                case REG_ONE: throw_parse_error("cannot refer to REG_ONE!");
                default: return bound_parameter(type, (preg_t) i, lo_or_hi);
            }
        }
    }

    throw_parse_error("unknown register " + s.substr(1));
}

std::optional<bound_parameter> inst_assembler::parse_param() {
    std::string tk;
    in >> tk;

    if(!tk.length()) {
        return std::nullopt;
    }

    if(tk[0] == '$') {
        regval_t val;
        if(tk.size() > 3 && tk.compare(1, 2, "0x") == 0) {
            val = std::stoi(tk.substr(3), 0, 16);
        } else if(tk.size() > 3 && tk.compare(1, 2, "0b") == 0) {
            val = std::stoi(tk.substr(3), 0, 2);
        } else if(tk.size() > 2 && tk.compare(1, 1, "0") == 0) {
            val = std::stoi(tk.substr(2), 0, 8);
        } else {
            val = std::stoi(tk.substr(1));
        }
        return bound_parameter(val);
    }

    std::optional<bound_parameter> reg = lookup_reg(tk);
    if(reg) {
        return *reg;
    }

    return bound_parameter(chunk(tk, false));
}

void inst_assembler::handle_label(std::string &tk) {
    chunk o(tk.substr(0, tk.length() - 1), true);
    buff.push_back(o);
}

void inst_assembler::bind_virtual(virtual_instruction uo, std::vector<bound_parameter> params) {
    std::vector<preg_t> ius(uo.bi.size());

    std::optional<chunk> constval;
    for(int j = 0; j < uo.bi.size(); j++) {
        if(j >= NUM_IUS) {
            throw_parse_error("too many args!");
        }

        switch(uo.bi[j].kind) {
            case slot::SLOT_REG: {
                ius[j] = uo.bi[j].val.reg;
                break;
            }
            case slot::SLOT_ARG: {
                if(uo.bi[j].val.argidx >= params.size()) {
                    throw_internal_error("can't bind opcode: desired arg number too great");
                }

                ius[j] = params[uo.bi[j].val.argidx].reg;

                if(params[uo.bi[j].val.argidx].type == parameter::PARAM_CONST) {
                    if(constval) {
                        throw_parse_error("attempting to bind user constvalue when constvalue already assigned");
                    }
                    
                    constval = params[uo.bi[j].val.argidx].constval;
                }
                break;
            }
            case slot::SLOT_CONSTVAL: {
                ius[j] = REG_ID;

                if(constval) {
                    throw_parse_error("attempting to bind alias (not user) constvalue when constvalue already assigned (probably by the user)");
                }

                constval = uo.bi[j].val.constval;
                break;
            }
            default: {
                throw_internal_error("unknown slot kind");
            }
        }
    }

    buff.push_back(uo.build_inst(constval.has_value(), ius));

    if(constval) {
        buff.push_back(*constval);
    }
}

void inst_assembler::handle_instruction(std::string &tk) {
    std::optional<family> f = arch::self().lookup_family(tk);
    if(!f) {
        throw_parse_error("no such instruction '" + tk + "'");
    }

    std::vector<bound_parameter> params;
    std::vector<kcpu::parameter::kind> param_kinds;
    while(true) {
        std::optional<bound_parameter> arg = parse_param();
        if(!arg) {
            break;
        }
        params.push_back(*arg);
        param_kinds.push_back(arg->type);
    }

    std::optional<std::string> an = f->match(param_kinds);
    if(!an) {
        throw_parse_error("bad argument types for instruction '" + tk + "'");
    }

    // FIXME issue a warning if a constant > size of a btye has been bound to a byte register parameter.

    // TODO in the error path, try to find a partial match to explain that you can't pass a const somewhere.
    // if(a->args.maybeconst != j) {
    //     throw_parse_error("Const arg not allowed in that place!");
    // }

    std::optional<alias> a = arch::self().lookup_alias(*an);
    if(!a) {
        throw_parse_error("no such alias '" + *an + "'");
    }

    for(auto j = a->insts.begin(); j < a->insts.end(); j++) {
        bind_virtual(*j, params);
    }
}

void inst_assembler::parse() {
    std::string tk;
    if(!(in >> tk)) {
        throw_parse_error("no token");
    }
    
    if(!tk.length()) {
        return;
    }

    if(tk[0] == '#') {
        return;
    }

    if(tk[tk.length() - 1] == ':') {
        handle_label(tk);
        return;
    }
    
    if(arch::self().inst_is_prefix(tk)) {
        std::string tk2;
        if(!(in >> tk2)) {
            throw_parse_error("no second token");
        }

        tk += " ";
        tk += tk2;
    }

    handle_instruction(tk);
}

static std::unordered_map<std::string, regval_t> build_label_table(std::vector<chunk> ocs) {
    std::unordered_map<std::string, regval_t> labels;

    regval_t pos = 0;
    for(auto i = ocs.begin(); i < ocs.end(); i++) {
        if(!i->concrete && i->label_def) {
            labels.emplace(i->label, pos);
            continue;
        }

        pos += 2;
    }

    return labels;
}

static std::vector<regval_t> resolve_labels(std::unordered_map<std::string, regval_t> labels, std::vector<chunk> ocs) {
    std::vector<regval_t> ret;

    for(auto i = ocs.begin(); i < ocs.end(); i++) {
        if(i->concrete) {
            ret.push_back(i->val);
            continue;
        }

        if(i->label_def) {
            continue;
        }
        
        auto lbl = labels.find(i->label);
        if(lbl == labels.end()) {
            throw assembler::parse_error(0 /* FIXME store, then recall lineno */, "unknown label: " + i->label);
        }

        ret.push_back(lbl->second);
    }

    return ret;
}

std::vector<regval_t> assemble(std::istream *in) {
    std::vector<chunk> ops;

    uint32_t lines = 0;
    std::string line;
    while(std::getline(*in, line)) {
        lines++;
        std::stringstream ssl(line);
        std::string token;
        while(std::getline(ssl, token, ';')) {
            std::stringstream st(line);
            inst_assembler ia(st, ops, lines);
            ia.parse();
        }
    }

    return resolve_labels(build_label_table(ops), ops);
}


}