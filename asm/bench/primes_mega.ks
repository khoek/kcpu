# Test: testcases for function find_next_prime
# for functions: all arguments passed on the stack, %ra is return code

CALL run_tests

# all cases pass
HLT

run_tests:
    # test cases

    PUSH $5
    PUSH $4
    CALL try_case
    ADD $4 %rsp

    PUSH $197
    PUSH $194
    CALL try_case
    ADD $4 %rsp

    PUSH $569
    PUSH $564
    CALL try_case
    ADD $4 %rsp

    PUSH $1009
    PUSH $998
    CALL try_case
    ADD $4 %rsp

    PUSH $1931
    PUSH $1914
    CALL try_case
    ADD $4 %rsp

    PUSH $5051
    PUSH $5040
    CALL try_case
    ADD $4 %rsp

    PUSH $7537
    PUSH $7530
    CALL try_case
    ADD $4 %rsp

    PUSH $11701
    PUSH $11700
    CALL try_case
    ADD $4 %rsp

    PUSH $31337
    PUSH $31334
    CALL try_case
    ADD $4 %rsp

    PUSH $36107
    PUSH $36098
    CALL try_case
    ADD $4 %rsp

    PUSH $36919
    PUSH $36914
    CALL try_case
    ADD $4 %rsp

    RET

case_fail:
    ABRT
    HLT

try_case:
    ENTER

    LDWO %rsp $4 %ra

    PUSH %ra
    CALL find_next_prime
    ADD $2 %rsp

    LDWO %rsp $6 %rb
    CMP %rb %ra
    JNZ case_fail

    LEAVE
    RET

find_next_prime:
    ENTER

    LDWO %rsp $4 %rb
    find_next_prime_loop:
        PUSH %rb
        CALL primetest
        POP %rb

        TST %ra
        JNZ find_next_prime_out
        ADD $1 %rb
        JMP find_next_prime_loop
    find_next_prime_out:
        MOV %rb %ra
        LEAVE
        RET

# arg1 = n, arg2 = m, ret = n mod m
modulo:
    ENTER

    LDWO %rsp $6 %ra
    LDWO %rsp $4 %rb

    modulo_loop:
        CMP %ra %rb
        JL modulo_out
        SUB %rb %ra
        JMP modulo_loop

    modulo_out:
        LEAVE
        RET

# arg1 = n, ret = is_prime(n)
primetest:
    ENTER

    LDWO %rsp $4 %rc
    MOV $2 %rd

    primetest_loop:
        CMP %rd %rc
        JE primetest_out

        PUSH %rc
        PUSH %rd
        CALL modulo
        ADD $4 %rsp

        # modulo returns 0 if %rd divides %rc
        TST %ra
        JZ primetest_fail
        ADD $1 %rd

        JMP primetest_loop

    primetest_out:
        MOV $1 %ra
        LEAVE
        RET

    primetest_fail:
        MOV $0 %ra
        LEAVE
        RET