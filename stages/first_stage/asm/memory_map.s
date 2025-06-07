.code16

push eax
push ebx
push ecx
push edx
push bp

2: # Int_15_setup                         
    xor ebx, ebx               # Clear continuation because it is the first ca
    mov es, bx                 # Clear segment regist
    mov di, {map_address}      # Buffer addre
    mov edx, {smap}            # 'SMA

3: # Int15 loop
    mov ecx, {region_size}     # Buffer si
    mov eax, {function_code}   # Int15 function co
    clc                        # Clear carry flag just before the interru
    int 0x15
    jc 4f                      # Check if an error occured in the carry flag.
    mov edx, {smap}
    cmp eax, edx               # Check the signature to verify successful call
    jnz 4f
    test ebx, ebx              # Check if this is the last structu
    jz 4f 
    add di, 24                 # Move buffer address forward
    inc dword ptr [{len_address}]
    jmp 3b
4: # end 
    pop bp
    pop edx
    pop ecx
    pop ebx
    pop eax
    ret