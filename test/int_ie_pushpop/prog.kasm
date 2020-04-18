# Check that the interrupt enable bit remains enabled in program,
# if it is disabled in the interrupt handler.

LIHP int

MOV $0 %ra
ST data.int_count %ra

MOV $0x8006 %rbp
IOW $0x0001 %rbp

MOV $0xC004 %ra
IOW $0x0001 %ra

EI

NOP
NOP
NOP

MOV $0xC004 %ra
IOW $0x0001 %ra

NOP
NOP
NOP

LD data.int_count %ra
CMP $2 %ra
JNE fail

HLT

fail:
    ABRT

int:
    DI

    LD data.int_count %ra
    ADD $1 %ra
    ST data.int_count %ra

    MOV $0x4000 %ra
    IOW $0x0001 %ra

    IRET

data.int_count:
    NOP