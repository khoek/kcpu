LIHP int_handle

MOV $0x800C %rbp
IOW $0x0001 %rbp

EI

main:
MOV $0 %ra
ST data.int_1_count %ra
ST data.int_2_count %ra

TST $0xFFFF
MOV $0xBEEF %ra
MOV $0x0000 %rb
MOV $0xBEEF %rc
MOV $0xBEEF %rd
# need aligned stack pointer
MOV $0xBEE0 %rsp

# We skip the low two interrupt bits, since they are NMIs.
MOV $0xC004 %rbp
IOW $0x0001 %rbp
ADD $1 %rb
MOV $0xC008 %rbp
IOW $0x0001 %rbp
ADD $1 %rb
MOV $0xC004 %rbp
IOW $0x0001 %rbp
ADD $1 %rb
MOV $0xC004 %rbp
IOW $0x0001 %rbp
ADD $1 %rb
MOV $0xC008 %rbp
IOW $0x0001 %rbp
ADD $1 %rb
MOV $0xC004 %rbp
IOW $0x0001 %rbp
ADD $1 %rb
MOV $0xC008 %rbp
IOW $0x0001 %rbp
ADD $1 %rb

JZ fail

CMP $0xBEEF %ra
JNE fail
CMP $0x0007 %rb
JNE fail
CMP $0xBEEF %rc
JNE fail
CMP $0xBEEF %rd
JNE fail
CMP $0xBEE0 %rsp
JNE fail

LD data.int_1_count %ra
CMP $4 %ra
JNE fail

LD data.int_2_count %ra
CMP $3 %ra
JNE fail

HLT

int_handle:
    PUSHA

    IOR $0x0001 %rc
    AND $0x3FFF %rbp
    CMP %rc %rbp
    JNE fail

    CMP $0x0004 %rc
    JE int_handle.1

    CMP $0x0008 %rc
    JE int_handle.2

    JMP fail

    int_handle.1:
        LD data.int_1_count %ra
        ADD $1 %ra
        ST data.int_1_count %ra
        JMP+DI int_handle.exit

    int_handle.2:
        LD data.int_2_count %ra
        ADD $1 %ra
        ST data.int_2_count %ra
        JMP+DI int_handle.exit

    int_handle.exit:
        MOV $0x4000 %rbp
        IOW $0x0001 %rbp

        POPA
        IRET

fail:
    ABRT

data.int_1_count:
    NOP

data.int_2_count:
    NOP