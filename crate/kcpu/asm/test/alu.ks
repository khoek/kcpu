# TODO THIS


MOV $3 %ra
ADD $2 %ra



MOV $1243 %ra
MOV %ra %rb
NEG %rb
ADD3 %ra %rb %rc

TST %rc
JNZ fail




MOV %ra %rb
NOT %rb
AND %ra %rb

TST %rb
JNZ fail

MOV %ra %rb
NOT %rb
NOT %rb
AND %ra %rb

CMP %ra %rb
JNE fail

HLT



fail:
    ABRT
