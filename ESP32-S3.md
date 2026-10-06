
## ESP32-S3 Pins & Peripherals

| Pin / Peripheral | Purpose in our project                               |
| ---------------- | ---------------------------------------------------- |
| `GPIO48`         | Data signal to onboard SK68XX RGB LED                |
| `RMT`            | Generates precise timing for the RGB LED protocol    |
| `USB` / `UART`   | Serial communication, flashing and logs              |
| `GPIO`           | General digital input/output                         |
| `ADC`            | Reading analog voltage                               |
| `PWM`            | Generating variable-duty-cycle signals               |
| `I2C`            | Communication with sensors/displays                  |
| `SPI`            | High-speed communication with displays/flash/sensors |
| `UART`           | Serial communication with external devices           |
| `Wi-Fi`          | Network communication                                |
| `Bluetooth`      | Wireless communication                               |

ESP32-S3
┌───────────────────────────────┐
│                               │
│  CPU                          │
│                               │
│  RMT    ← timing generator    │
│  UART   ← serial communication│
│  SPI    ← SPI communication   │
│  I2C    ← I2C communication   │
│  ADC    ← analog measurement  │
│  PWM    ← pulse generation    │
│  Timer  ← timing              │
│                               │
└───────────────────────────────┘

Peripheral = CPU/GPU/network card inside the computer
Pin        = USB/Ethernet/HDMI port on the outside