# Module & Package System (`init.apsi`)

เอกสารข้อกำหนดระบบโมดูล การจัดวางโครงสร้างแพ็กเกจ และบทบาทของ `init.apsi` ในภาษา Apsis

---

## 1. ปรัชญาของระบบโมดูล (Module Philosophy)

ระบบโมดูลของ Apsis ยึดหลักการ:
1. **Directory-as-Namespace**: โฟลเดอร์คือตัวกำหนด Path ของโมดูล
2. **Explicit Entrypoints (`init.apsi`)**: ทุกโฟลเดอร์ที่เป็นแพ็กเกจจะมี `init.apsi` เป็นประตูหลัก
3. **No Hidden Magic**: ลำดับการ Resolve ชัดเจน ตรวจสอบได้ตั้งแต่ตอน Compile

---

## 2. โครงสร้างโฟลเดอร์ระดับคอมไพเลอร์ (`lib/`)

เมื่อติดตั้งตัวคอมไพเลอร์ `apsic` จะมีโฟลเดอร์ `lib/` ติดตั้งอยู่ในระบบเสมอ:

```
apsis-toolchain/
├── bin/
│   └── apsic                      ## ตัวคอมไพเลอร์
└── lib/
    ├── builtin/                   ## [PRELUDE] โหลดให้อัตโนมัติ ไม่ต้องสั่ง import
    │   ├── init.apsi              ## print, panic, assert, debug (ประตูหลักของ Prelude)
    │   ├── option.apsi            ## enum Option[T] { Some(T), None }
    │   └── result.apsi            ## enum Result[T, E] { Ok(T), Err(E) }
    │
    └── global/                    ## [GLOBAL PACKAGES] ค้นหาได้จากทุกที่
        ├── math.apsi              ## แพ็กเกจแบบไฟล์เดี่ยว
        └── std/                   ## โฟลเดอร์เนมสเปซมาตรฐาน
            ├── math/              ## แพ็กเกจ math แบบโฟลเดอร์
            │   ├── init.apsi      ## ◄── ประตูหลักของ "std/math"
            │   ├── vector.apsi    ## ซับโมดูลย่อย
            │   └── matrix.apsi    ## ซับโมดูลย่อย
            └── net/
                ├── init.apsi      ## ◄── ประตูหลักของ "std/net"
                ├── tcp.apsi
                └── http.apsi
```

---

## 3. หน้าที่และการทำงานของ `init.apsi`

`init.apsi` ทำหน้าที่เสมือน **"โถงกลาง (Barrel Entrypoint)"** ของแพ็กเกจนั้นๆ:

### ตัวอย่าง: โครงสร้างไฟล์ใน `lib/global/std/math/`
```
std/math/
├── init.apsi
├── vector.apsi
└── matrix.apsi
```

### โค้ดภายใน `std/math/init.apsi`:
`init.apsi` สามารถรวบรวมฟังก์ชันหลักและ Re-export ซับโมดูลออกมาเพื่อให้ผู้ใช้ภายนอกเข้าถึงได้ทันที:

```
## std/math/init.apsi

## 1. Re-export ซับโมดูลภายในออกมาสู่สาธารณะ
pub import "./vector" { Vector2, Vector3 }
pub import "./matrix" { Matrix4 }

## 2. ฟังก์ชันหลักประจำตัวแพ็กเกจ
pub const PI f64 = 3.141592653589793

pub fn sqrt(val f64) f64 {
    return val.sqrt()
}
```

---

## 4. วิธีการเรียกใช้จากภายนอก (Import Patterns)

### แบบที่ 1: Import ผ่านจุดศูนย์กลาง (`init.apsi`)
เมื่อผู้ใช้เขียนคำสั่ง:
```
import "std/math" { * }
```
คอมไพเลอร์จะตรงเข้าหา `lib/global/std/math/init.apsi` ทันที ผู้ใช้จะสามารถใช้งานได้ทั้ง `Vector2`, `Matrix4`, และ `sqrt()` โดยไม่ต้องรู้ว่าข้างในแยกไฟล์ไว้อย่างไร

### แบบที่ 2: Import เจาะจงซับโมดูลโดยตรง
หากผู้ใช้ต้องการใช้เฉพาะไฟล์ย่อย ไม่ต้องการโหลดทั้งหมด:
```
import "std/math/vector" { Vector2 }
```
คอมไพเลอร์จะวิ่งตรงไปที่ `lib/global/std/math/vector.apsi` โดยข้าม `init.apsi`

### แบบที่ 3: Local Project Imports (Relative Paths)
สำหรับโปรเจกต์ของผู้ใช้เอง ใช้เครื่องหมาย `./` หรือ `../`:
```
import "./components/button" { Button }
import "../utils/string_helper" as str
```
หาก `./components/button` เป็นโฟลเดอร์ คอมไพเลอร์จะค้นหา `./components/button/init.apsi` โดยอัตโนมัติ

---

## 5. ลำดับการค้นหาไฟล์ของคอมไพเลอร์ (Resolution Algorithm)

เมื่อคอมไพเลอร์พบคำสั่ง `import "X"`:

```
[ คำสั่ง import "X" ]
          │
          ├──► 1. เป็น Relative Path (ขึ้นต้นด้วย ./ หรือ ../) หรือไม่?
          │         ├─► ตรวจสอบไฟล์: <path>/X.apsi
          │         └─► ตรวจสอบโฟลเดอร์: <path>/X/init.apsi
          │
          └──► 2. เป็น Global / Standard Library หรือไม่?
                    ├─► ตรวจสอบไฟล์: lib/global/X.apsi
                    └─► ตรวจสอบโฟลเดอร์: lib/global/X/init.apsi
```
หากไม่พบในลำดับทั้งหมด คอมไพเลอร์จะหยุดและรายงานข้อผิดพลาด `ModuleNotFoundError` ทันทีตอน Compile-time
