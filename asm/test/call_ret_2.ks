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

MOV $0x1000 %rsp

MOV $0x0117 %ra
MOV $0x1000 %rsp

MOV $0xBEEF %rb

PUSH %ra
MOV $0xDEAD %ra

POP %ra

CALL fun
MOV $0xDDDD %rd
CALL bar

MOV $0xCCCC %rc

CALL check
HLT

check:
    CMP $0xAAAA %ra
    JE check_1
    ABRT
    check_1:
        CMP $0xBBBB %rb
        JE check_2
        ABRT
    check_2:
        CMP $0xCCCC %rc
        JE check_3
        ABRT
    check_3:
        CMP $0xDDDD %rd
        JE check_4
        ABRT
    check_4:
        RET

fail:
    ABRT

fun:
    MOV $0xAAAA %ra
    RET

bar:
    MOV $0xBBBB %rb
    RET