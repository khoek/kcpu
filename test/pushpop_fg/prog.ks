TST $0xBEEF

PUSH $0xDEAD
PUSH $0x1337
PUSHFG

POPFG
PUSHFG

POP %rc
POP %rb
POP %ra

CMP $0xDEAD %ra
JNE fail

CMP $0x1337 %rb
JNE fail

CMP $0xF %rc
JNE fail

HLT

fail:
    ABRT