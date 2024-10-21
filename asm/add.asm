; This program loads some numbers into registers r1 and r2,
; adds r1 and r2 together and saves the result to r0

; We expect from the OS:
; R32 = memcpy()
; R33 = malloc()

program:
	mov r1 pc
	mov r2 r1
	
	lea r1 [num_r1 - program]
	lea r2 [num_r2 - program]
	
	load r1 r1
	load r2 r2
	
	add r0 r1 r2
	
	halt

num_r1: 42
num_r2: 31415926