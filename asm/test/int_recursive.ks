EI

MOV $0 %ra
ST data.had_int %ra

MOV $0x80FF %rbp
IOW $0x0001 %rbp

MOV $0xBEEF %ra

test_read:
    LIHP int_norm
    MOV $0x0008 %rb
    MOV $0x8020 %rbp
    MOV $0xD1 %rc

    # begin time critical section

    IOW %rc %rb
    IOW %rc %rbp

    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP

    # end time critical section

    CMP $0xBEEF %ra
    JNE fail

LD data.had_int %ra
CMP $1 %ra
JNE fail

LD data.had_nmi %ra
CMP $1 %ra
JNE fail

HLT

int_fail:
fail:
    ABRT

int_norm:
    LIHP int_nmi

    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP
    NOP

    LD data.had_nmi %rbp
    CMP $1 %rbp
    JNE fail

    LD data.had_int %rbp
    ADD $1 %rbp
    ST data.had_int %rbp

    MOV $0x4000 %rbp
    IOW $0x0001 %rbp

    LIHP int_fail

    IRET

int_nmi:
    LD data.had_nmi %rbp
    ADD $1 %rbp
    ST data.had_nmi %rbp

    MOV $0x4000 %rbp
    IOW $0x0001 %rbp
    IRET

data.had_int:
    NOP

data.had_nmi:
    NOP