import capstone, struct
SEC1=open('/tmp/sena_r3/sec1.bin','rb').read()
md=capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_THUMB)
def dis(va, n=0x60, label=''):
    off=va-BASE
    for i in md.disasm(SEC1[off:off+n], va):
        print(f"0x{i.address:08X} {i.bytes.hex():<12} {i.mnemonic} {i.op_str}")
dis(0x081A4FD8, 0x60)
