MOV $3 %ra

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

CMP $3 %ra
JNE fail

HLT

fail:
    ABRT

