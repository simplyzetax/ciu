# Bill of Materials

All listed MPNs are **UNVERIFIED**: confirm the manufacturer datasheet, lifecycle, footprint land pattern, and source availability before release. U1 typical TX current is 17 mA at 3.3 V; all passive, connector, crystal, antenna, and test-point rows have no operating quiescent current or DC leakage, so they do not consume the 3 mA margin to the <20 mA incremental-TX budget.

| Refdes | Value | Footprint | MPN | Rationale |
| --- | --- | --- | --- | --- |
| U1 | AT86RF233-ZU | Package_DFN_QFN:QFN-32-1EP_5x5mm_P0.5mm_EP3.3x3.3mm | AT86RF233-ZU | **UNVERIFIED**; datasheet-verifiable 2.4 GHz O-QPSK transceiver in the mandated QFN-32 package, with 17 mA typical TX current at 3.3 V. |
| J1 | Conn_01x09 | Connector_PinHeader_2.54mm:PinHeader_1x09_P2.54mm_Vertical | 61300911121 | **UNVERIFIED**; datasheet-verifiable 9-position, 2.54 mm through-hole header for the required host signals; passive and zero quiescent current. |
| Y1 | 16MHz | Crystal:Crystal_SMD_Abracon_ABM8G-4Pin_3.2x2.5mm | ABM8G-16.000MHZ-4Y-T3 | **UNVERIFIED**; datasheet-verifiable 16 MHz crystal compatible with the AT86RF233 clock requirement; no DC operating current. |
| C1 | 100nF | Capacitor_SMD:C_0402_1005Metric | GRM155R71C104KA88D | **UNVERIFIED**; datasheet-verifiable 50 V X7R local DVDD decoupler; capacitor DC leakage is negligible. |
| C2 | 100nF | Capacitor_SMD:C_0402_1005Metric | GRM155R71C104KA88D | **UNVERIFIED**; datasheet-verifiable 50 V X7R local AVDD decoupler; capacitor DC leakage is negligible. |
| C3 | 100nF | Capacitor_SMD:C_0402_1005Metric | GRM155R71C104KA88D | **UNVERIFIED**; datasheet-verifiable 50 V X7R local EVDD decoupler; capacitor DC leakage is negligible. |
| C4 | 1uF | Capacitor_SMD:C_0402_1005Metric | GRM155R60J105KE19D | **UNVERIFIED**; datasheet-verifiable 6.3 V X5R DVDD bulk decoupler; capacitor DC leakage is negligible. |
| C5 | 1uF | Capacitor_SMD:C_0402_1005Metric | GRM155R60J105KE19D | **UNVERIFIED**; datasheet-verifiable 6.3 V X5R AVDD bulk decoupler; capacitor DC leakage is negligible. |
| C6 | 1uF | Capacitor_SMD:C_0402_1005Metric | GRM155R60J105KE19D | **UNVERIFIED**; datasheet-verifiable 6.3 V X5R EVDD bulk decoupler; capacitor DC leakage is negligible. |
| C7 | 12pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H120JA01D | **UNVERIFIED**; datasheet-verifiable C0G crystal-load capacitor; zero DC operating current. |
| C8 | 12pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H120JA01D | **UNVERIFIED**; datasheet-verifiable C0G crystal-load capacitor; zero DC operating current. |
| R1 | 100k | Resistor_SMD:R_0402_1005Metric | RC0402FR-07100KL | **UNVERIFIED**; datasheet-verifiable RESET pull-up; worst-case continuous current is 33 uA at 3.3 V. |
| FB1 | 600R | Inductor_SMD:L_0402_1005Metric | BLM15AG601SN1D | **UNVERIFIED**; datasheet-verifiable 600 ohm ferrite bead for AVDD filtering; DC resistance causes no quiescent draw. |
| L1 | 3.9nH | Inductor_SMD:L_0402_1005Metric | LQG15HS3N9S02D | **UNVERIFIED**; datasheet-verifiable 0402 RF inductor reserved for the Microchip reference matching network; no DC operating current. |
| L2 | 3.9nH | Inductor_SMD:L_0402_1005Metric | LQG15HS3N9S02D | **UNVERIFIED**; datasheet-verifiable 0402 RF inductor reserved for the Microchip reference matching network; no DC operating current. |
| C9 | 1pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H1R0CA01D | **UNVERIFIED**; datasheet-verifiable C0G RF matching capacitor; no DC operating current. |
| C10 | 1.5pF | Capacitor_SMD:C_0402_1005Metric | GRM1555C1H1R5CA01D | **UNVERIFIED**; datasheet-verifiable C0G RF matching capacitor; no DC operating current. |
| AE1 | 2.4GHz | RF_Antenna:Johanson_2450AT18x100 | 2450AT18A100E | **UNVERIFIED**; datasheet-verifiable 2.4 GHz chip antenna with reference layout; passive and zero quiescent current. |
| TP1 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | **UNVERIFIED**; datasheet-verifiable single-pin test point for MOSI; passive and zero quiescent current. |
| TP2 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | **UNVERIFIED**; datasheet-verifiable single-pin test point for MISO; passive and zero quiescent current. |
| TP3 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | **UNVERIFIED**; datasheet-verifiable single-pin test point for SCLK; passive and zero quiescent current. |
| TP4 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | **UNVERIFIED**; datasheet-verifiable single-pin test point for /SEL; passive and zero quiescent current. |
| TP5 | TestPoint | TestPoint:TestPoint_Keystone_5015_Micro_Mini | 5015 | **UNVERIFIED**; datasheet-verifiable single-pin RF-feed test point; passive and zero quiescent current. |
