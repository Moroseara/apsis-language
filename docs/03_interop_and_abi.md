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

### การจับคู่ Type ระหว่าง C และ Apsis:

| ภาษา C | ภาษา Apsis | พฤติกรรม |
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

---

## 4. Plugin ABI: ส่ง Trait Object ข้าม `.so` / `.dll`

Apsis **ไม่มี Native ABI ที่เสถียร** (เหมือน Rust และ C++) — Layout ของ Struct, vtable, และการ Mangle ชื่อ อาจเปลี่ยนระหว่างเวอร์ชันคอมไพเลอร์ เพื่อให้คอมไพเลอร์ออปติไมซ์ได้เต็มที่ การส่งข้อมูลข้ามไฟล์ Library จึงมี 2 ระดับ:

| ระดับ | ใช้เมื่อ | เงื่อนไข |
| :--- | :--- | :--- |
| **Native (Unstable)** | exe + dll ที่ Build มาพร้อมกันในโปรเจกต์เดียว (`link "engine"` ใน doc 07 §6.2) | ต้องคอมไพล์ด้วย `apsic` เวอร์ชันเดียวกันทุกไฟล์ — ตรวจสอบตอนโหลด ถ้าไม่ตรงจะปฏิเสธการโหลด |
| **C-ABI (Stable)** | Plugin ที่คนอื่น Build แยก, Plugin จากเวอร์ชันคอมไพเลอร์ต่างกัน, ภาษาอื่นเรียกใช้ | ใช้ Trait ที่มี `#[c_abi]` |

### `#[c_abi]` Trait: Plugin Interface ที่เสถียร

```
## plugin_api/init.apsi — แพ็กเกจกลางที่ทั้งตัวแอปและ Plugin ใช้ร่วมกัน
#[c_abi(version = 1)]
pub trait AudioPlugin {
    fn name(self) *const u8
    fn process(mut self, samples *mut f32, len usize)
}
```

คอมไพเลอร์จะสร้างให้อัตโนมัติ:
- **vtable แบบ C Struct** ที่ Layout ตายตัว (Function Pointer เรียงตามลำดับที่ประกาศ + Field `version` ไว้หัว)
- **Header C** (`apsis-bindgen`, doc 07 §7.2) ให้ Plugin ที่เขียนด้วย C / C++ / Zig ทำตาม Interface เดียวกันได้

กฎของ `#[c_abi]` Trait:
- Parameter และค่าที่คืนต้องเป็น Type ที่ใช้กับ C ได้เท่านั้น (Primitive, Raw Pointer, `#[packed]` Struct) — ส่ง `string`, `[T]`, `{K: V}` ตรงๆ ไม่ได้ (ส่งเป็น Pointer + ความยาว แทน)
- เพิ่ม Method ใหม่ได้เฉพาะ **ท้าย Trait** แล้วขึ้น `version` — ตัวแอปเช็ค `version` ก่อนเรียก Method ใหม่ ทำให้ Plugin เก่ายังใช้ได้
- ห้ามลบหรือสลับลำดับ Method (ต้องสร้าง Trait ใหม่แทน)

### ฝั่ง Plugin

```
## plugins/reverb/init.apsi
import "plugin_api" { AudioPlugin }

struct Reverb {
    room_size f32 = 0.5
}

impl AudioPlugin for Reverb {
    fn name(self) *const u8 { return b"Reverb\0".as_ptr() }
    fn process(mut self, samples *mut f32, len usize) { ... }
}

## จุดเข้าที่ตัวแอปจะค้นหาตอน dlopen
#[export_plugin]
fn create() AudioPlugin {
    return Reverb()
}
```

### ฝั่งตัวแอป

```
import "std/dylib" { Library }
import "plugin_api" { AudioPlugin }

fn load_plugins() Result[[AudioPlugin], LoadError] {
    let mut plugins [AudioPlugin] = []
    for path in fs.glob("plugins/*.so")? {
        let lib = Library.open(path)?                       ## ตรวจ version ของ vtable ให้อัตโนมัติ
        plugins.push(lib.create_plugin[AudioPlugin]()?)
    }
    return Result.Ok(plugins)
}
```

ภายในตัวแอป `[AudioPlugin]` ใช้งานเหมือน Trait Object ปกติทุกอย่าง (Syntax §33) — ความต่างมีแค่ vtable ที่ Layout ตายตัวตามมาตรฐาน C
