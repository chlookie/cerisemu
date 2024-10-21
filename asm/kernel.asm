
; The kernel returns an exit code in r0 after terminating.
;   0: everything went well and the program halted correctly 
;   1: the program failed
;   2: the kernel encountered an unexpected internal error

; The machine starts out with PC = (RWX, 0, MAX, 0) (the master capability)

os_init:
	; R255 = MASTER CAPABILITY
	mov R255 PC
	
	; Save master capability to memory
	mov R1 R255
	lea R1 [master]
	store R1 R255
	
	; Setup interrupt pointers
	; Setup internal_err
	mov R2 R255
	lea R2 [internal_err]
	mov R1 R255
	lea R1 [p_internal_err]
	store R1 R2
		
	; Set internal_err as the FAIL interrupt
	mov R1 R255
	lea R1 0xFFFE
	store R1 R2
	
	; Setup the rest of interrupt pointers
	; Setup program_fail
	mov R2 R255
	lea R2 [program_fail]
	mov R1 R255
	lea R1 [p_program_fail]
	store R1 R2
	
	; Setup program_halt
	mov R2 R255
	lea R2 [program_halt]
	mov R1 R255
	lea R1 [p_program_halt]
	store R1 R2
	
	; Setup p_invalid
	mov R2 R255
	subseg R2 0x0 0x0
	restrict R2 O
	mov R1 R255
	lea R1 [p_invalid]
	store R1 R2
	
	
	
	; Setup memcpy
		; Create pointer to memcpy()
		mov R2 R255
		lea R2 0x100 ; memcpy() is at 0x100
		subseg R2 0x100 0x200
		restrict R2 E
		; Save pointer to memcpy()
		mov R1 R255
		lea R1 [p_memcpy]
		store R1 R2
	
	; Setup malloc
		; Setup args for init()
			; r0: integer that will be used to return to the caller routine
			mov R0 [-(0x200 - malloc_init_return) - 1] ; init() is at 0x200
			; r1: (RWX, low, high, pointer)
			mov R1 R255
			lea R1 0x1000
			subseg R1 0x1000 0xF000 ; heap will be in the range [0x1000..0xF000[
		; Call init()
		; (A lea-jump is used instead of a jmp so that PC stays RWX instead of E->RX, and same to return back from init())
		malloc_init_call:
			lea PC [0x200 - malloc_init_call - 1]
		malloc_init_return:
		; Fix up the pointer to malloc
		subseg R1 0x200 0x300
		restrict R1 E
		; Save pointer to malloc()
		mov R2 R255
		lea R2 [p_malloc]
		store R2 R1

	; Downgrade PC to RX
	restrict PC RX
	
	; Delete master capability from R255
	mov R255 0
	
	; Clear other registers
	mov R0 0
	mov R1 0
	mov R2 0
	
	; Setting up OS done!
	
	; Now go run a program
	GOTO run_program



os_storage:
	master:         empty, ; MASTER CAPABILITY (RWX, 0, MAX, 0)
	
	p_memcpy:       empty, ; pointer to memcpy
	p_malloc:       empty, ; pointer to malloc
	
	p_internal_err: empty, ; pointer to internal_err
	p_program_fail: empty, ; pointer to program_fail
	p_program_halt: empty, ; pointer to program_halt
	
	p_invalid:      empty, ; purposefully invalid capability (O, 0, 0, 0)



internal_err:
	; Set "exit code" to 2, the OS had an internal error
	mov R0 2
	GOTO exit
	
program_fail:
	; Set "exit code" to 1, a program failed
	mov R0 1
	GOTO exit
	
program_halt:
	; Set "exit code" to 0, all good
	mov R0 0
	GOTO exit

exit:
	; Unbind the halt interrupt to make sure we can actually exit
	
	mov R1 PC
	lea R1 [master - exit]
	load R2 R1 ; r2 = master capability
	lea R1 [p_invalid - master]
	load R1 R1 ; r1 = p_invalid
	lea R1 R0 ; add "exit code" to address of p_invalid so it shows up in the backtrace :)
	lea R2 0xFFFF ; halt interrupt
	store R2 R1 ; write p_invalid to the halt interrupt
	
	; Goodbye world!
	halt



run_program:
	; We are assuming:
	; - the program we are running is located at address 0x300
	; - the program is smaller than 255 rows (not a hard limit, but we are restricting its PC space to [0x300..0x400[ )
	; - the program expects memcpy() in R32
	; - the program expects malloc() in R33
	
	mov R0 PC
	mov R3 R0
	mov R4 R0
	mov R32 R0
	mov R33 R0
	
	; Setup the jump address in R0
	lea R0 [-run_program + 0x300]
	subseg R0 0x300 0x400
	restrict R0 E
	
	; Load memcpy into R32
	lea R32 [-run_program + p_memcpy]
	load R32 R32
	
	; Load malloc into R33
	lea R33 [-run_program + p_malloc]
	load R33 R33
	
	; Load master capability in R3
	lea R3 [-run_program + master]
	load R3 R3
	
	; Set interrupt for fail
	lea R4 [-run_program + p_program_fail]
	load R5 R4
	lea R3 0xFFFE ; interrupt for fail is at 0xFFFE
	store R3 R5
	
	; Set interrupt for halt
	lea R4 [-p_program_fail + p_program_halt]
	load R5 R4
	lea R3 1 ; interrupt for halt is at 0xFFFF
	store R3 R5
	
	; Delete master capability from R3 and cleanup R4 & R5
	mov R3 0
	mov R4 0
	mov R5 0
	
	; Run program
	jmp R0

	



