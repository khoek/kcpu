MOV $3 %ra
MOV $88 %rb

TST $0
ADD %ra %rb %rc
JZ fail

CMP $3 %ra
JNE fail
CMP $88 %rb
JNE fail
CMP $91 %rc
JNE fail

TST $0
ADDNF %ra %rb %rc
JNZ fail

CMP $3 %ra
JNE fail
CMP $88 %rb
JNE fail
CMP $91 %rc
JNE fail

HLT

fail:
    ABRT

