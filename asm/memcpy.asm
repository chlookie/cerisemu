; PROGRAM LENGTH: 33 rows

; r0: return
; r1: source memory (RO, a, b, a)
; r2: destination memory (RW, c, d, c)

memcpy:
	; make sure that address of r1 is =base
	geta r3 r1
	getb r4 r1
	sub r3 r4 r3 ; if address is bigger than base by x, now r3=-x
	lea r1 r3 ; done
	
	; make sure that address of r2 is =base
	geta r3 r2
	getb r4 r2
	sub r3 r4 r3 ; if address is bigger than base by x, now r3=-x
	lea r2 r3 ; done
	
	; check that the size of the memory zones r1 and r2 are the same
	getb r3 r1
	gete r4 r1
	sub r3 r4 r3 ; r3=end-base
	getb r4 r2
	gete r5 r2
	sub r4 r5 r4 ; r4=end-base
	sub r3 r4 r3 ; r3 should now be zero. if not, fail
		mov r4 PC
		lea r4 4
		jnz r4 r3
	GOTO ok
		fail
	ok:
	
	
	; r3 will be our buffer
	loop:
		; copy source into the buffer
		load r3 r1
		; copy buffer into destination
		store r2 r3
		; i++
		lea r1 1
		lea r2 1
		
		; check if the loop should continue
		gete r4 r1
		geta r5 r1
		sub r4 r5 r4
		
		; if r4 is zero, we break the loop
		mov r5 PC
		lea r5 4
		jnz r5 r4
		GOTO break
		GOTO loop
	break:
	
	; we're done!
	jmp r0

