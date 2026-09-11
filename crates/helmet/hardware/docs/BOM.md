# Bill of materials

Every MPN below is **UNVERIFIED** pending distributor availability, datasheet revision, and footprint-land-pattern review before release. Passive parts add no quiescent current; capacitor leakage is negligible relative to the 40 mA board budget. U1 is the only active load and its datasheet-typical TX current is approximately 17 mA, below the 20 mA incremental TX limit.

| Refdes | Value | Footprint | MPN | Rationale |
| --- | --- | --- | --- | --- |
| U1 | AT86RF233-ZU | Package_DFN_QFN:QFN-32-1EP_5x5mm_P0.5mm_EP3.3x3.3mm | AT86RF233-ZU | Required QFN-32 2.4 GHz transceiver; datasheet pinout matches installed `RF_ZigBee:AT86RF233-Z`; typical TX current is about 17 mA and no other active load threatens the 40 mA average budget. |
| B1 | 2450BM15A0015 | ciu:Johanson_2450BM15A0015 | 2450BM15A0015 | Required 2.45 GHz balun/filter; datasheet pins 1=unbalanced, 2/5/6=GND, 3/4=balanced are capturable with installed `Connector_Generic:Conn_01x06`; passive, so no quiescent-current impact. |
| AE1 | 2.4GHz | RF_Antenna:Johanson_2450AT18x100 | 2450AT18A100 | 2.4 GHz chip antenna compatible with the selected footprint and installed `Device:Antenna_Chip`; passive and requires vendor keepout/matching verification. |
| J1 | Conn_01x09 | Connector_PinHeader_2.54mm:PinHeader_1x09_P2.54mm_Vertical | TSW-109-07-G-S | Sourceable 2.54 mm vertical 1x9 header; nine real pins confirmed in installed `Connector_Generic:Conn_01x09`; passive with no quiescent-current impact. |
| Y1 | 16MHz | Crystal:Crystal_SMD_Abracon_ABM8G-4Pin_3.2x2.5mm | ABM8G-16.000MHZ-18-D2Y-T | 16 MHz crystal in the selected 3.2 mm x 2.5 mm package; installed `Device:Crystal` exposes the two electrical pins; passive and load-capacitance compatibility must be verified. |
| FB1 | 600R | Inductor_SMD:L_0402_1005Metric | BLM15HG601SN1D | 600 ohm-at-100 MHz 0402 ferrite bead for supply isolation; installed `Device:FerriteBead` has two passive pins and adds no quiescent load beyond negligible DCR loss. |
| R1 | 100k | Resistor_SMD:R_0402_1005Metric | RC0402FR-07100KL | 1% 0402 reset bias resistor; worst-case 33 uA if continuously across 3.3 V, negligible against the 40 mA budget. |
| C1 | 100nF | Capacitor_SMD:C_0402_1005Metric | GRM155R71C104KA88D | 16 V X7R 0402 local bypass capacitor; negligible leakage. |
| C2 | 100nF | Capacitor_SMD:C_0402_1005Metric | GRM155R71C104KA88D | 16 V X7R 0402 local bypass capacitor; negligible leakage. |
| C3 | 100nF | Capacitor_SMD:C_0402_1005Metric | GRM155R71C104KA88D | 16 V X7R 0402 local bypass capacitor; negligible leakage. |
| C4 | 1uF | Capacitor_SMD:C_0402_1005Metric | GRM155R61A105KE15D | 10 V X5R 0402 bulk/rail bypass capacitor; negligible leakage. |
| C5 | 1uF | Capacitor_SMD:C_0402_1005Metric | GRM155R61A105KE15D | 10 V X5R 0402 bulk/rail bypass capacitor; negligible leakage. |
| C6 | 1uF | Capacitor_SMD:C_0402_1005Metric | GRM155R61A105KE15D | 10 V X5R 0402 bulk/rail bypass capacitor; negligible leakage. |
| C7 | 12pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H120JA01D | 50 V C0G 0402 crystal load capacitor; passive with negligible leakage; final value depends on crystal and stray capacitance. |
| C8 | 12pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H120JA01D | 50 V C0G 0402 crystal load capacitor; passive with negligible leakage; final value depends on crystal and stray capacitance. |
| C9 | 1pF | Capacitor_SMD:C_0402_1005Metric | GJM1555C1H1R0BB01D | High-Q 50 V C0G 0402 RF matching capacitor; passive with negligible leakage; tune only from RF validation. |
| C10 | 1.5pF | Capacitor_SMD:C_0402_1005Metric | GJM1555C1H1R5BB01D | High-Q 50 V C0G 0402 RF matching capacitor; passive with negligible leakage; tune only from RF validation. |
| C11 | 22pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H220JA01D | Required 50 V C0G 0402 series coupling capacitor from RFP to balun; passive with negligible leakage. |
| C12 | 22pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H220JA01D | Required 50 V C0G 0402 series coupling capacitor from RFN to balun; passive with negligible leakage. |
| L1 | 3.9nH | Inductor_SMD:L_0402_1005Metric | LQG15HS3N9S02D | High-frequency 0402 RF matching inductor; passive with no quiescent-current impact; tune only from RF validation. |
| L2 | 3.9nH | Inductor_SMD:L_0402_1005Metric | LQG15HS3N9S02D | High-frequency 0402 RF matching inductor; passive with no quiescent-current impact; tune only from RF validation. |
| TP1 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | Micro-mini single-pin test point; installed `Connector:TestPoint` pin 1 confirmed; passive. |
| TP2 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | Micro-mini single-pin test point; installed `Connector:TestPoint` pin 1 confirmed; passive. |
| TP3 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | Micro-mini single-pin test point; installed `Connector:TestPoint` pin 1 confirmed; passive. |
| TP4 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | Micro-mini single-pin test point; installed `Connector:TestPoint` pin 1 confirmed; passive. |
| TP5 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | Micro-mini single-pin test point; installed `Connector:TestPoint` pin 1 confirmed; passive. |
