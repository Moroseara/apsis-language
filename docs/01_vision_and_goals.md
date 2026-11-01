# Apsis: Vision and Core Principles

## 1. Vision & Identity

**Apsis (`.apsi`)** คือภาษาโปรแกรมยุคใหม่ที่ออกแบบมาเพื่อ **การพัฒนาแอปพลิเคชันทั่วไปและไลบรารีประสิทธิภาพสูง (General-Purpose Application & High-Performance Library Language)** 

### เป้าหมายหลัก 4 ประการ (The 4 Pillars):
1. **Safety by Default**: ปลอดภัยต่อหน่วยความจำ (Memory Safety), ปราศจาก Data Race (Concurrency Safety), และ Type Safety 100%
2. **No GC / No VM / Minimal Runtime**: ไม่มี Garbage Collector, ไม่มี Virtual Machine, ไม่มี Thread Scheduler แฝง — ต้นทุนที่เหลือทั้งหมด **มองเห็นได้และคาดเดาได้** (ดูตารางด้านล่าง)
3. **C-Level Performance**: โค้ดส่วนที่ต้องเร็ว (Hot Path) ทำความเร็วได้เท่า C ด้วยคุณสมบัติ No-Aliasing และการออปติไมซ์ขั้นสูงของ LLVM — บางกรณีเร็วกว่า C เพราะคอมไพเลอร์รู้ว่าไม่มี Pointer ชี้ซ้อนกัน
4. **App-Oriented & High Productivity**: เขียนง่าย ลื่นไหล ลดภาระทางสมองของนักพัฒนา (Cognitive Load) ให้ความรู้สึกใกล้เคียงภาษาที่มี GC — โค้ดทั่วไปไม่ต้องคิดเรื่อง Lifetime หรือ Pointer เลย

### ต้นทุนที่มีจริง (Honest Cost Model)

Apsis ไม่ได้อ้างว่า "0% Overhead ทุกกรณี" แต่สัญญาว่า **ไม่มีต้นทุนแฝงที่ผู้ใช้มองไม่เห็น**:

| ต้นทุน | เกิดเมื่อ | ปิด/หลีกเลี่ยงได้ไหม |
| :--- | :--- | :--- |
| ตรวจ Index เกินขอบ (Bounds Check) | `list[i]` | คอมไพเลอร์ตัดทิ้งเมื่อพิสูจน์ได้ (เช่นใน `for i in 0..list.length()`), หรือใช้ `unsafe` |
| ตรวจ Reference Count ตอนแก้ไข (CoW) | แก้ `string` / `[T]` / `{K: V}` ที่อาจแชร์อยู่ | ส่วนใหญ่ถูกตัดทิ้งด้วย Uniqueness Analysis (doc 02 §3) |
| ตรวจ Generation ของ Handle | `arena[id]` (doc 02 §5.6) | 1 การเปรียบเทียบตัวเลข — ตัดทิ้งได้เมื่อคอมไพเลอร์พิสูจน์ว่า Id ยังใช้ได้ |
| vtable Lookup | ใช้ Trait เป็น Type (Dynamic Dispatch) | ใช้ Generic `[T: Trait]` แทน |
| ไม่มีต้นทุนเลย | Struct, Enum, Generic, `mut` Parameter, Closure ที่ไม่หลุด Scope | — |

---

## 2. Positioning & Comparison

| มิติการเปรียบเทียบ | C / C++ | Rust | Go / Dart / C# | ภาษาของเรา (MVS-based) |
| :--- | :--- | :--- | :--- | :--- |
| **Memory Safety** | ❌ Manual (เสี่ยงแครช) | ✅ Borrow Checker | ✅ Garbage Collector | ✅ **Mutable Value Semantics** |
| **Runtime Overhead** | 0% | ~0% (Bounds Check) | 10% - 30% (GC/VM) | **~0% ใน Hot Path, ต้นทุนเล็กและมองเห็นได้ในโค้ดทั่วไป** |
| **ความง่ายในการเขียน UI/App** | ปานกลาง (บั๊กง่าย) | ยาก (ติด Lifetime/Borrow) | ง่ายมาก | **ง่ายและเป็นธรรมชาติ** |
| **การส่งออก C-ABI (.dll, .so)** | ดีเยี่ยม | ดีมาก (แต่ Binding ซับซ้อน) | หนัก (พก GC ไปด้วย) | **ดีเยี่ยม ไร้ภาระ (Clean C-ABI)** |
| **ความเร็วในการเปิดแอป** | ทันที (Instant) | ทันที (Instant) | มีหน่วงเล็กน้อย | **ทันที (Instant)** |

---

## 3. เป้าหมายการใช้งาน (Target Domains)

1. **Native Desktop & Mobile Applications**:
   - พัฒนา Core Logic ของแอป และระบบ UI (ร่วมกับสถาปัตยกรรมแบบ Declarative / Immediate UI)
2. **High-Performance Shared Libraries (.so, .dylib, .dll)**:
   - ส่งออก Core Logic ไปให้ภาษาอื่นเรียกใช้ เช่น Python, Flutter, Node.js, Unity, Godot
3. **Systems & Application Engines**:
   - Compilers, Game Engines, Audio Processing, Database Engines, High-throughput Network Services

---

## 4. จุดยืนต่อ Python และภาษาที่มี GC

**Apsis ไม่ได้มาแทน Python แต่มาเป็นส่วนที่ทำให้ Python เร็วขึ้น**

- Python ช้าเพราะ **Dynamic Typing + Interpreter + ทุกค่าเป็น Object บน Heap** ไม่ใช่เพราะ GC อย่างเดียว ภาษาที่มี Static Type และคอมไพล์เป็น Native (ไม่ว่าจะมี GC หรือไม่) ก็เร็วกว่า Python หลายสิบเท่าอยู่แล้ว
- จุดแข็งของ Python คือ Ecosystem (numpy, PyTorch, pandas) ซึ่งเอาชนะตรงๆ ไม่ได้ในระยะสั้น
- สิ่งที่ Apsis ทำได้ดีกว่าภาษาที่มี GC คือ **เป็นไลบรารีให้ภาษาอื่นเรียก**: ไฟล์ `.so` / `.dll` ไม่ต้องพก Runtime ไม่ต้องสั่งเริ่ม GC และไม่ตีกับ GC ของภาษาเจ้าบ้าน (doc 03)

ดังนั้นเส้นทางที่ตั้งใจไว้คือ:

```
[ ระยะที่ 1 ] เขียนส่วนที่ต้องเร็วด้วย Apsis ──► ให้ Python / Flutter / Unity เรียกผ่าน C-ABI
[ ระยะที่ 2 ] เขียนทั้งแอป / Tool / Engine ด้วย Apsis เพราะเขียนง่ายพอแล้ว
```

### ทำไมไม่ใช้ GC ทั้งที่ GC ทำให้เขียนง่าย?
เพราะ "เขียนง่ายแบบภาษาที่มี GC" ทำได้โดยไม่ต้องมี GC:
- **Value Semantics** (doc 02) ทำให้ไม่ต้องคิดเรื่อง Pointer, Lifetime หรือ "ใครแก้ List ของเราลับหลัง"
- **ไม่มี Lifetime, ไม่มี `clone()`, ไม่มี `Box` / `Rc` / `Weak`** (doc 02 §5): Tree เขียนเป็น Value ตรงๆ, Graph ใช้ `Arena[T]` + `Id[T]`, State ร่วมใช้ Message — ไม่มีทาง Memory Leak จาก Cycle
- คำที่ผู้ใช้ต้องเขียนเพิ่มจากภาษา GC มีแค่ `mut` ตรงที่ "ค่าถูกแก้" ซึ่งช่วยให้อ่านโค้ดง่ายขึ้นด้วย
