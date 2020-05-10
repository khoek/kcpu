# kCPU Assembler and Virtual Machine

[![CI](https://github.com/khoek/kcpu/workflows/CI/badge.svg)](https://github.com/khoek/kcpu/actions?query=workflow%3ACI)

An assembler and hardware-simulating virtual machine for my CPU architecture `kCPU`.

It exists to test the robustness of the design of the electrical hardware (over at [khoek/komputer](https://github.com/khoek/komputer)), simulating buses (detecting collisions, etc.) and different hardware modules, so a much faster soft-implementation is possible. Rough tests show that the VM is about ~475x slower than bare metal, which given a 4.8GHz simulating CPU ends up about 5x faster than the planned hardware clock speed of 2MHz.

## Compiling and Testing

To build use `make` in the repository root. Artefacts appear in `bin`, the most interesting of which are; `kasm`, the assembler; `lib/libkcpu.a`, the main language spec/hardware spec/assembler library; and `run_vm`, a stub which starts up the VM from this library.

There is a test suite for the VM/assembler combination, which is invoked with `make test`.
