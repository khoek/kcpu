MOV $3 %ra
MOV $-3 %rb
ADD %ra %rb
JZ l1
ABRT

l1:
TST $1
JNZ l2
ABRT

l2:
JNZ l3
ABRT

l3:
MOV $-3 %rb
JNZ l4
ABRT

l4:
ADDNF %ra %rb
JNZ l5
ABRT

l5:

HLT