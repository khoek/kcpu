
MOV $0xD10E %rsp


MOV %rsp %ra
SUB $4 %ra

PUSH $0xBEEF
PUSH %rsp
POP %rb

CMP %ra %rb
JNE fail

PUSH $0xDEAD
POP %rsp
CMP $0xDEAD %rsp
JNE fail

HLT

fail:
    ABRT