; This program copies a hello world string from its local memory into the heap.
; It also uses register r42 as a marker that can be seen in the backtrace,
;   r42 = '+' on start,
;   r42 = '-' at the end.

; We expect from the OS:
; R32 = memcpy()
; R33 = malloc()

program:
	geta R10 PC
	mov R42 '+'
	
	; Call malloc
	mov R1 [payload_end - payload]
	mov R0 pc, lea R0 4, restrict R0 E
	jmp R33
	
	mov R2 R1
	
	; Prep source memory
	marker0:
	mov R1 PC
	lea R1 [payload - marker0]
	add R11 R10 [payload]
	add R12 R10 [payload_end]
	subseg R1 R11 R12
	restrict R1 RO
	
	; Call memcpy
	mov R0 pc, lea R0 4, restrict R0 E
	jmp R32
	
	mov R42 '-'
	halt

payload:
	"Hello World!"
payload_end: