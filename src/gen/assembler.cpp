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

class arg_info {
    public:
    std::vector<preg_t> args;
    std::optional<std::pair<uint8_t, chunk>> constval;

    arg_info(std::vector<preg_t> args, std::optional<std::pair<uint8_t, chunk>> constval);
};

arg_info::arg_info(std::vector<preg_t> args, std::optional<std::pair<uint8_t, chunk>> constval)
    : args(args), constval(constval) {}

class inst_assembler {
    private:
    std::istream &in;
    std::vector<chunk> &buff;
    uint32_t line;

    void throw_parse_error(std::string msg);
    void throw_internal_error(std::string msg);

    std::optional<preg_t> lookup_reg(std::string s);

    std::pair<preg_t, std::optional<chunk>> parse_arg();
    void handle_label(std::string &tk);
    void handle_instruction(std::string &tk);
    void bind_virtual(virtual_instruction uo, arg_info ai);

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

std::optional<preg_t> inst_assembler::lookup_reg(std::string s) {
    for(int i = 0; i < NUM_PREGS; i++) {
        if(s == PREG_NAMES[i]) {
            switch(i) {
                case REG_ID: throw_parse_error("cannot refer to REG_ID!");
                case REG_ONE: throw_parse_error("cannot refer to REG_ONE!");
                default: return (preg_t) i;
            }
        }
    }
    return std::nullopt;
}

std::pair<preg_t, std::optional<chunk>> inst_assembler::parse_arg() {
    std::string tk;
    in >> tk;

    if(!tk.length()) {
        throw_parse_error("read empty token!");
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
        return std::pair(REG_ID, val);
    }

    std::optional<preg_t> reg = lookup_reg(tk);
    if(reg) {
        return std::pair(*reg, std::nullopt);
    }

    return std::pair(REG_ID, std::optional(chunk(tk, false)));
}

void inst_assembler::handle_label(std::string &tk) {
    chunk o(tk.substr(0, tk.length() - 1), true);
    buff.push_back(o);
}

void inst_assembler::bind_virtual(virtual_instruction uo, arg_info ai) {
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
                if(uo.bi[j].val.argidx >= ai.args.size()) {
                    throw_internal_error("can't bind opcode: desired arg number too great");
                }

                ius[j] = ai.args[uo.bi[j].val.argidx];

                if(ai.constval && uo.bi[j].val.argidx == ai.constval->first) {
                    if(constval) {
                        throw_parse_error("attempting to bind user constvalue when constvalue already assigned");
                    }
                    
                    constval = ai.constval->second;
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
    std::optional<alias> a = arch::self().alias_lookup(tk);
    if(!a) {
        throw_parse_error("no such instruction '" + tk + "'");
    }

    std::vector<preg_t> args;
    std::optional<std::pair<uint8_t, chunk>> constval;
    for(uint8_t j = 0; j < a->args.count; j++) {
        std::pair<preg_t, std::optional<chunk>> arg = parse_arg();
        if (arg.second) {
            if(a->args.maybeconst != j) {
                throw_parse_error("Const arg not allowed in that place!");
            }

            constval = std::pair(j, *arg.second);
        }

        args.push_back(arg.first);
    }

    for(auto j = a->insts.begin(); j < a->insts.end(); j++) {
        bind_virtual(*j, arg_info(args, constval));
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