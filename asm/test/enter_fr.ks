
PUSH $0x1337
PUSH $0xBEEF
CALL sample
POP %ra
POP %rb

CMP $0x1337 %rb
JNE fail
CMP $0xBEEF %ra
JNE fail

HLT

sample:
    ENTERFR $4

    MOV $0xAAAA %ra
    STWO %rbp $-2 %ra
    MOV $0xBBBB %ra
    STWO %rbp $-4 %ra

    PUSH $0xCCCC
    PUSH $0xDDDD

    POP %ra
    CMP $0xDDDD %ra
    JNE fail

    POP %ra
    CMP $0xCCCC %ra
    JNE fail

    LDWO %rbp $-2 %rb
    CMP $0xAAAA %rb
    JNE fail

    LDWO %rbp $-4 %rb
    CMP $0xBBBB %rb
    JNE fail

    LEAVE
    RET

fail:
    ABRT

