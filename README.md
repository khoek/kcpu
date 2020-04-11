# kCPU Assembler and Virtual Machine ![C/C++ CI](https://github.com/khoek/kcpu-vm/workflows/C/C++%20CI/badge.svg)

An assembler and hardware-simulating virtual machine for my CPU architecture `kCPU`.

It exists to test the robustness of the design of the electrical hardware (over at [khoek/komputer](https://github.com/khoek/komputer)), simulating buss\es (detecting collisions, etc.) and different hardware modules, so a much faster soft-implementation is possible.

## Compiling and Testing

To build use `make` in the repository root. There is a test suite for the VM/assembler combination, which is invoked with `make test`.
