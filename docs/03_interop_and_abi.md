# ABI & Interoperability (C-ABI First)

## 1. Zero-Cost Foreign Function Interface (FFI)

เนื่องจากภาษาไม่มี Runtime, ไม่มี GC, และไม่มี VM การส่งข้อมูลเข้าและออกจากภาษา C จึงเป็น **Zero-Cost Abstraction** (เป็นเพียงคำสั่ง CPU Call ธรรมดา ไม่มีการแปลง Context)

---

## 2. การนำเข้าไลบรารีภายนอก (`extern "C"` & `#[link]`)

สามารถประกาศฟังก์ชันภายนอกจาก `libc` หรือ Shared Library (`.so`, `.dylib`, `.dll`) ใดๆ ด้วยบล็อก `extern "C"`:

```
## เชื่อมต่อกับ libc มาตรฐานของระบบ
extern "C" {
    fn puts(str *const u8) i32
    fn exit(status i32) void
    fn getpid() i32
}

## เชื่อมต่อกับ Dynamic Library ภายนอกด้วย #[link("name")]
#[link("sqlite3")]
extern "C" {
    fn sqlite3_open(filename *const u8, db *mut *mut void) i32
    fn sqlite3_close(db *mut void) i32
}

fn main() {
    unsafe {
        puts(b"Hello from libc puts!\0".as_ptr())
        let pid = getpid()
        print(f"Current Process ID: {pid}")
    }
}
```

### การจับคู่ Type ระหว่าง C และ Apsi:

| ภาษา C | ภาษา Apsi | พฤติกรรม |
| :--- | :--- | :--- |
| `int`, `float`, `char` | `i32`, `f32`, `u8` | ส่งผ่าน CPU Registers ทันที |
| `size_t` | `usize` / `u64` | ตามสถาปัตยกรรม CPU |
| `const T*` (Pointer อ่านอย่างเดียว) | `*const T` | Raw Pointer อ่านอย่างเดียว (ใช้ใน `unsafe`) |
| `T*` (Pointer เขียนได้) | `*mut T` | Raw Pointer เขียนได้ (ใช้ใน `unsafe`) |
| Opaque Struct Pointer (`void*`) | `*mut void` | ใช้สำหรับเก็บ Handle ของไลบรารี C |

---

## 3. การส่งออกเป็น Shared Library (Export to the World)

ภาษาของเราสามารถคอมไพล์โค้ดให้กลายเป็น Dynamic Shared Library เพื่อให้ภาษาอื่นๆ นำไปใช้งานได้:

- **Clean C-ABI Export**: กำหนดฟังก์ชันด้วยมาตรฐาน C calling convention (`extern "C"`)
- **No Runtime Dependency**: ไฟล์ `.so` หรือ `.dll` ที่ได้จะมีขนาดเล็กระดับกิโลไบต์ ไม่มี Thread Scheduler หรือ GC พ่วงไปด้วย
- **Universal Consumers**:
  - **Python**: เรียกผ่าน `ctypes` หรือ `cffi`
  - **Dart / Flutter**: เรียกผ่าน `dart:ffi`
  - **JavaScript / Node.js**: เรียกผ่าน N-API หรือ Node FFI
  - **C# / .NET**: เรียกผ่าน `DllImport` (P/Invoke)
  - **Game Engines**: เรียกใช้ได้ทั้งใน Unreal Engine, Unity, และ Godot
