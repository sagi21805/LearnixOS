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
    mov ecx, {extended_region_size}     # Buffer si
    mov eax, {function_code}   # Int15 function co
    clc                        # Clear carry flag just before the interru
    int 0x15
    cmp ecx, {regular_region_size} # Check if the given output is regular or extended
    jz 4f
    test dword ptr [di + {regular_region_size}], 1    # Check if this entry should be ignored by extended attributes. 
    jz 4f 
    jmp 5f 
4:
    inc dword ptr [{len_address}]
    jc 6f                      # Check if an error occured in the carry flag.
    mov edx, {smap}
    cmp eax, edx               # Check the signature to verify successful call
    jnz 6f
5:  
    test ebx, ebx              # Check if this is the last entry
    jz 6f
    add di, {extended_region_size}      # Move buffer address forward
    jmp 3b
6: # end
    pop bp
    pop edx
    pop ecx
    pop ebx
    pop eax
    ret