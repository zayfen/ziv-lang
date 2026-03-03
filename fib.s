.section .text
.globl _start

main:
    pushq %rbp
    movq %rsp, %rbp
    subq $16, %rsp

    movq $0, -8(%rbp)
    movq $1, -16(%rbp)
    movq -8(%rbp), %rax
    movq %rax, -32(%rbp)
    movq -16(%rbp), %rax
    movq %rax, -40(%rbp)
    movq -32(%rbp), %rax
    addq -40(%rbp), %rax
    movq %rax, -48(%rbp)
    movq -48(%rbp), %rax
    movq %rax, -24(%rbp)
    movq -16(%rbp), %rax
    movq %rax, -64(%rbp)
    movq -24(%rbp), %rax
    movq %rax, -72(%rbp)
    movq -64(%rbp), %rax
    addq -72(%rbp), %rax
    movq %rax, -80(%rbp)
    movq -80(%rbp), %rax
    movq %rax, -56(%rbp)
    movq -24(%rbp), %rax
    movq %rax, -96(%rbp)
    movq -56(%rbp), %rax
    movq %rax, -104(%rbp)
    movq -96(%rbp), %rax
    addq -104(%rbp), %rax
    movq %rax, -112(%rbp)
    movq -112(%rbp), %rax
    movq %rax, -88(%rbp)
    movq -56(%rbp), %rax
    movq %rax, -128(%rbp)
    movq -88(%rbp), %rax
    movq %rax, -136(%rbp)
    movq -128(%rbp), %rax
    addq -136(%rbp), %rax
    movq %rax, -144(%rbp)
    movq -144(%rbp), %rax
    movq %rax, -120(%rbp)
    movq -88(%rbp), %rax
    movq %rax, -160(%rbp)
    movq -120(%rbp), %rax
    movq %rax, -168(%rbp)
    movq -160(%rbp), %rax
    addq -168(%rbp), %rax
    movq %rax, -176(%rbp)
    movq -176(%rbp), %rax
    movq %rax, -152(%rbp)
    movq -120(%rbp), %rax
    movq %rax, -192(%rbp)
    movq -152(%rbp), %rax
    movq %rax, -200(%rbp)
    movq -192(%rbp), %rax
    addq -200(%rbp), %rax
    movq %rax, -208(%rbp)
    movq -208(%rbp), %rax
    movq %rax, -184(%rbp)
    movq -152(%rbp), %rax
    movq %rax, -224(%rbp)
    movq -184(%rbp), %rax
    movq %rax, -232(%rbp)
    movq -224(%rbp), %rax
    addq -232(%rbp), %rax
    movq %rax, -240(%rbp)
    movq -240(%rbp), %rax
    movq %rax, -216(%rbp)
    movq -184(%rbp), %rax
    movq %rax, -256(%rbp)
    movq -216(%rbp), %rax
    movq %rax, -264(%rbp)
    movq -256(%rbp), %rax
    addq -264(%rbp), %rax
    movq %rax, -272(%rbp)
    movq -272(%rbp), %rax
    movq %rax, -248(%rbp)
    movq -216(%rbp), %rax
    movq %rax, -288(%rbp)
    movq -248(%rbp), %rax
    movq %rax, -296(%rbp)
    movq -288(%rbp), %rax
    addq -296(%rbp), %rax
    movq %rax, -304(%rbp)
    movq -304(%rbp), %rax
    movq %rax, -280(%rbp)
    movq $0, %rax

    leave
    ret

_start:
    call main
    movq %rax, %rdi
    movq $60, %rax
    syscall
