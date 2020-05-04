JMP l0
ABRT

l0:
TST $0
JZ l1
ABRT

l1:
JNZ fail
TST $1
JZ fail
JNZ l2
ABRT

l2:
MOV $0xFFFF %ra
MOV %ra %rb
ADD %ra %rb
JNC fail
JC l3
ABRT

l3:
MOV $1000 %ra
MOV l4 %rb
STW %ra %rb
# FIXME, allow this in one go instead:
# STW %ra l4
LDJMP $1000
ABRT

l4:
HLT

fail:
ABRT