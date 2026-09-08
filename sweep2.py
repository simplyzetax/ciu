# Q2 completion part 2: dump the raw code around 0x081A4F80-0x081A5060 with correct alignment.
import capstone, struct
SEC1=open('/tmp/sena_r3/sec1.bin','rb').read()
md=capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_THUMB)
def dis(va, n=0x80):
    off=va-0x08133000
    for i in md.disasm(SEC1[off:off+n], va):
        print(f"0x{i.address:08X} {i.bytes.hex():<12} {i.mnemonic} {i.op_str}")
print('=== 0x081A4F80 (probe survey fn) ===')
dis(0x081A4F80, 0x60)
