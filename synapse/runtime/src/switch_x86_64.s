    // rdi: where to save this stack pointer. rsi: the stack to continue on.
    // Everything the System V ABI makes a callee preserve.
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    mov  qword ptr [rdi], rsp
    mov  rsp, rsi
    pop  r15
    pop  r14
    pop  r13
    pop  r12
    pop  rbx
    pop  rbp
    ret
