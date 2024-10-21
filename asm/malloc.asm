; PROGRAM LENGTH: 39 rows

; PC: (RWX, ...)
; r0: integer that will be used to return to the caller routine
; r1: (RWX, low, high, pointer)
; returns r1: pointer to malloc subroutine
init:
	mov r2 PC ; important: expects PC to be RWX at least 
	restrict r2 RW
	lea r2 [heap - init]
	store r2 r1 ; store at [heap]
	mov r3 r2
	lea r3 [-1]
	store r3 r2 ; store at [heap_access]
	
	; return the pointer for malloc
	marker0:
	mov r1 PC
	lea r1 [malloc - marker0]
	
	; clear registers just in case
	mov r2 0
	mov r3 0
	
	; Jump back to caller
	marker1:
	sub r0 r0 [marker1 - init + 1]
	lea PC r0


heap_access:
	empty ; Will contain (RW, _, _, heap)
heap:
	empty ; Will contain (RW, low, high, pointer)



; PC: (RX, ...)
; r0: return
; r1: integer determining the number of words to allocate
;
; returns r1: capability to the allocated memory
;
; fails if size <= 0 or if it does not have enough space left
malloc:
	; get access to the heap
	mov r4 PC
	lea r4 [-2]
	load r4 r4 ; r4 = heap_access: (RW, _, _, heap)
	load r5 r4 ; r5 = heap: (RW, low, high, pointer)

	; check that size > 0
	lt r3 0 r1 ; if everything is correct, r3 = 1
	mov r2 pc
	lea r2 4
	jnz r2 r3 ; if r3 = 0, that means it's bad, so don't jump ahead and fail
	fail
	
	mov r2 r1 ; r2 = number of words to allocate
	
	geta r7 r5 ; r7 = future base of the return capability
	
	mov r6 r5 ; r6 = copy of (RW, low, high, pointer)
	lea r6 r2 ; increment the heap capability
	store r4 r6 ; store the heap capability back at [heap]
	
	geta r8 r6 ; r8 = future end of the return capability
	subseg r5 r7 r8
	
	; copy to r1, we're done
	mov r1 r5
	
	; clear registers just in case
	mov r2 0
	mov r3 0
	mov r4 0
	mov r5 0
	mov r6 0
	mov r7 0
	mov r8 0
	
	; return
	jmp r0
