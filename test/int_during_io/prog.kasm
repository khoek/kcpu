EI

MOV $0 %ra
ST data.had_int %ra

MOV $0x8008 %rbp
IOW $0x0001 %rbp

MOV $0xBEEF %ra
IOW $0xF4 %ra
MOV $0xDEAD %ra

test_read:
    LIHP int_success
    MOV $5 %rb
    MOV $0xD1 %rc
    MOV $0xF4 %rd

    # begin time critical section

    IOW %rc %rb
    IOR %rd %ra
    LIHP int_fail

    # end time critical section

    CMP $0xBEEF %ra
    JNE fail

LD data.had_int %ra
CMP $1 %ra
JNE fail

test_write:
    LIHP int_success
    MOV $0x1337 %ra
    MOV $5 %rb
    MOV $0xD1 %rc
    MOV $0xF4 %rd

    # begin time critical section

    IOW %rc %rb
    IOW %rd %ra
    LIHP int_fail

    # end time critical section

    MOV $0x7007 %ra
    IOR %rd %ra

    CMP $0x1337 %ra
    JNE fail

LD data.had_int %ra
CMP $2 %ra
JNE fail

HLT

int_fail:
fail:
    ABRT

int_success:
    LD data.had_int %rbp
    ADD $1 %rbp
    ST data.had_int %rbp

    MOV $0x4000 %rbp
    IOW $0x0001 %rbp
    IRET

data.had_int:
    NOP