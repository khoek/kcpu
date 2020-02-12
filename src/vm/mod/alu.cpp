#include <cassert>
#include <stdexcept>
#include "../../spec/ucode.hpp"
#include "alu.hpp"

namespace kcpu {

mod_alu::mod_alu(vm_logger &logger) : logger(logger) {
    result = {.val = 0, .flags = 0};
}

#define FLAG_CARRY      (1 << 0)
#define FLAG_N_ZERO     (1 << 1)
#define FLAG_SIGN       (1 << 2)
#define FLAG_N_OVERFLOW (1 << 3)

void mod_alu::dump_registers() {
    logger.logf("ADATA: %04X AFLAGS: %c%c%c%c\n", result.val,
      (result.flags & FLAG_CARRY      ) ? 'C' : 'c',
      (result.flags & FLAG_N_ZERO     ) ? 'z' : 'Z',
      (result.flags & FLAG_SIGN       ) ? 'S' : 's',
      (result.flags & FLAG_N_OVERFLOW ) ? 'o' : 'O');
}

static uint16_t encode_flags(bool carry, bool n_zero, bool sign, bool n_overflow) {
  return 0 | (carry ? FLAG_CARRY : 0) | (n_zero ? FLAG_N_ZERO : 0) | (sign ? FLAG_SIGN : 0) | (n_overflow ? FLAG_N_OVERFLOW : 0);
}

static op_result calc_arith_result(int16_t a, int16_t b, int16_t val, uint32_t carry_val) {
  return { .val = (uint16_t) val,
           .flags = encode_flags(carry_val > UINT16_MAX,
                                 val,
                                 val < 0,
                                 !(a >= 0 && b >= 0 && val < 0) || !(a < 0 && b < 0 && val >= 0)) };
}

static op_result calc_logic_result(int16_t a, int16_t b, uint16_t val) {
  return { .val = val,
           .flags = encode_flags(((uint16_t) val) & 0x0001,
                                 val,
                                 ((uint16_t) val) & 0x8000,
                                 !(a >= 0 && b >= 0 && ((int16_t) val) < 0) || !(a < 0 && b < 0 && ((int16_t) val) >= 0)) };
}

static op_result calc_shift_result(int16_t a, int16_t b, uint16_t val, uint16_t dropped) {
  return { .val = val,
           .flags = encode_flags(dropped,
                                 val,
                                 ((uint16_t) val) & 0x8000,
                                 !(a >= 0 && b >= 0 && ((int16_t) val) < 0) || !(a < 0 && b < 0 && ((int16_t) val) >= 0)) };
}

static op_result eval_add(uint16_t a, uint16_t b) {
  return calc_arith_result(a, b, (int16_t) (((int16_t) a) + ((int16_t) b)), ((uint32_t) a) + ((uint32_t) b));
}
static op add_op = {.nm = "+ ", .mode = 0x0, .eval = eval_add};

static op_result eval_sub(uint16_t a, uint16_t b) {
  return calc_arith_result(a, b, (int16_t) (((int16_t) b) - ((int16_t) a)), (((uint32_t) ((uint16_t) ~a)) + 1) + ((uint32_t) b));
}
static op sub_op = {.nm = "- ", .mode = 0x1, .eval = eval_sub};

static op_result eval_and(uint16_t a, uint16_t b) {
  return calc_logic_result(a, b, a & b);
}
static op and_op = {.nm = "& ", .mode = 0x2, .eval = eval_and};

static op_result eval_or(uint16_t a, uint16_t b) {
  return calc_logic_result(a, b, a | b);
}
static op or_op = {.nm = "| ", .mode = 0x3, .eval = eval_or};

static op_result eval_xor(uint16_t a, uint16_t b) {
  return calc_logic_result(a, b, a ^ b);
}
static op xor_op = {.nm = "^ ", .mode = 0x4, .eval = eval_xor};

static op_result eval_lsft(uint16_t a, uint16_t b) {
  return calc_shift_result(a, b, a << 1, a & 0x8000);
}
static op lsft_op = {.nm = "<<", .mode = 0x5, .eval = eval_lsft};

static op_result eval_rsft(uint16_t a, uint16_t b) {
  return calc_shift_result(a, b, a >> 1, a & 0x0001);
}
static op rshf_op = {.nm = ">>", .mode = 0x6, .eval = eval_rsft};

static op_result eval_arith_rsft(uint16_t a, uint16_t b) {
  return calc_shift_result(a, b, ((int16_t) (((int16_t) a) >> 1)), a & 0x0001);
}
// Not implemented in hardware:
//op arith_rshf_op = {.nm = "A>>", .mode = NULL, .eval = eval_arith_rsft};

static op_result eval_tst(uint16_t a, uint16_t b) {
  return calc_logic_result(a, b, a);
}
static op tst_op = {.nm = "TST", .mode = 0x7, .eval = eval_tst};

static op *ops[] = {&add_op, &sub_op, &and_op, &or_op, &xor_op, &lsft_op, &rshf_op, &tst_op};

void mod_alu::clock_outputs(uinst_t ui, bus_state &s) {
    if(ui & ACTRL_DATA_OUT) {
        assert(!(ui & ACTRL_INPUT_EN));
        s.assign(BUS_A, result.val);
    }

    if(ui & ACTRL_FLAGS_OUT) {
        assert(!(ui & ACTRL_INPUT_EN));
        s.assign(BUS_B, result.flags);
    }
}

void mod_alu::clock_inputs(uinst_t ui, bus_state &s) {
    if(ui & ACTRL_INPUT_EN) {
        uint8_t mode = DECODE_ACTRL_MODE(ui);
        if(mode & ~0b111) {
            throw vm_error("unknown ACTRL_MODE");
        }

        assert(ops[mode]->mode == mode);
        result = ops[mode]->eval(s.read(BUS_A), s.read(BUS_B));
    }
}

}