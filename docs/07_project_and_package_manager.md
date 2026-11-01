# Apsis: Project Manifest, BuildTool & Ecosystem Architecture

เอกสารข้อกำหนดระบบโปรเจกต์ `Apsis.kdl`, การจัดการ Dependencies, และสถาปัตยกรรม BuildTool Ecosystem ของภาษา **Apsis**

---

## 1. ปรัชญาการออกแบบ: สถาปัตยกรรม 2 ชั้น (The Dual-Layer Architecture)

ภาษา **Apsis** ถูกออกแบบด้วยหลักการแยกระหว่าง **"Core Compiler (`apsic`)"** และ **"Project Manager & BuildTool (`apsis`)"**:

```
[ ชั้นบน: Apsis Developer Tooling ]
├── Apsis.kdl (Manifest พื้นฐานสำหรับผู้ใช้ทั่วไป: ดึง library, pack รวมไฟล์เดียว)
└── Advanced BuildTool (สำหรับงานระดับสูง: แยก exe, dll, dylib, so, plugins)
                │
                ▼ (ส่งต่อคำสั่งคอมไพล์)
[ ชั้นล่าง: Core Compiler Engine (`apsic`) ]
└── ทำงานแบบเดียวกับ C / Clang: รับไฟล์ .apsi, ตรวจสอบ Safety MVS, พ่น Machine Code
    (เปิดโอกาสให้ CMake, Make, Ninja, หรือ Custom Tool มาเรียกใช้ได้อย่างอิสระ)
```

---

## 2. ยุคแรก: พื้นฐานที่เรียบง่ายและจบในตัว (`Apsis.kdl`)

ในเฟสแรก โปรเจกต์จะถูกขับเคลื่อนด้วยไฟล์ **`Apsis.kdl`** ที่มุ่งเน้นความเรียบง่ายและแพ็กทุกอย่างจบในไฟล์เดียว (Standalone Executable) เพื่อแก้ปัญหา DLL Hell:

```kdl
project name="my_app" version="0.1.0" {
    entry "src/main.apsi"
}

## ดึง Library ง่ายๆ จากโฟลเดอร์ในเครื่อง หรือ Git Repo
dependencies {
    math path="../libs/math"
    raylib git="github.com/apsi-lang/raylib" tag="v1.0.0"
}

## ในระดับพื้นฐาน: Pack ทุกอย่างลงใน Executable เดียว ดับเบิลคลิกเปิดได้ทันที
build {
    mode "standalone"    ## Static Link ทุกอย่างรวมเป็น 1 Executable
    opt_level 3
}
```

---

## 3. ยุคถัดไป: BuildTool และ Ecosystem ขั้นสูง

เมื่อโปรเจกต์เติบโตขึ้นและต้องการแยกโครงสร้างซับซ้อน เช่น:
- แยกตัวโปรแกรมหลักออกจาก Dynamic Library (`.exe` + `.dll` / `.dylib` / `.so`)
- ระบบ Plugin สำหรับเกมหรือแอปพลิเคชัน
- Custom Preprocessing หรือ Code Generation

เราจะพัฒนา **Apsis BuildTool** ขึ้นมารองรับการสร้าง Ecosystem ของตัวเองโดยเฉพาะ โดยตัว BuildTool จะทำงานร่วมกับ **`apsic`** ผ่านทางเดินอิสระ คล้ายกับที่ CMake เติบโตขึ้นมาควบคู่กับภาษา C

---

## 4. โครงสร้างโฟลเดอร์โปรเจกต์มาตรฐาน

```
my_app/
├── Apsis.kdl                 ## Manifest ควบคุมโปรเจกต์
├── apsi.lock                 ## การันตีความปลอดภัยและเวอร์ชันของโค้ด
├── src/
│   ├── main.apsi             ## จุดเริ่มต้นโปรแกรม
│   └── helper.apsi
└── dist/                     ## ไฟล์ผลลัพธ์หลังการ Build
    └── my_app                ## Standalone Executable (จบในก้อนเดียว)
```

---

## 5. คำสั่ง CLI ประจำตัวภาษา Apsis

| คำสั่ง | คำอธิบาย |
| :--- | :--- |
| `apsis new <name>` | สร้างโปรเจกต์ใหม่พร้อมไฟล์ `Apsis.kdl` และ `src/main.apsi` |
| `apsis build` | คอมไพล์โปรเจกต์ตามที่กำหนดใน `Apsis.kdl` |
| `apsis run` | คอมไพล์และรันโปรแกรมทันที |
| `apsis test` | สั่งรัน Unit Test ทุกฟังก์ชันที่มี `#[test]` |
| `apsic <file> [flags]` | สั่งคอมไพล์ระดับ Low-Level แบบตรงไปตรงมา (สไตล์ Clang/GCC) |
