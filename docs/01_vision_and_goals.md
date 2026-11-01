# Apsis: Vision and Core Principles

## 1. Vision & Identity

**Apsis (`.apsi`)** คือภาษาโปรแกรมยุคใหม่ที่ออกแบบมาเพื่อ **การพัฒนาแอปพลิเคชันทั่วไปและไลบรารีประสิทธิภาพสูง (General-Purpose Application & High-Performance Library Language)** 

### เป้าหมายหลัก 4 ประการ (The 4 Pillars):
1. **Safety by Default**: ปลอดภัยต่อหน่วยความจำ (Memory Safety), ปราศจาก Data Race (Concurrency Safety), และ Type Safety 100%
2. **Zero Runtime / No GC / No VM**: ไม่มี Garbage Collector, ไม่มี Virtual Machine, ไม่มี Runtime Overhead ฝังมากับโปรแกรม
3. **C-Level Performance or Faster**: ประสิทธิภาพและความเร็วในการทำงานเท่ากับหรือเร็วกว่าภาษา C ด้วยคุณสมบัติ No-Aliasing และการออปติไมซ์ขั้นสูงของ LLVM
4. **App-Oriented & High Productivity**: เขียนง่าย ลื่นไหล ลดภาระทางสมองของนักพัฒนา (Cognitive Load) ให้ความรู้สึกใกล้เคียงภาษาที่มี GC

---

## 2. Positioning & Comparison

| มิติการเปรียบเทียบ | C / C++ | Rust | Go / Dart / C# | ภาษาของเรา (MVS-based) |
| :--- | :--- | :--- | :--- | :--- |
| **Memory Safety** | ❌ Manual (เสี่ยงแครช) | ✅ Borrow Checker | ✅ Garbage Collector | ✅ **Mutable Value Semantics** |
| **Runtime Overhead** | 0% | 0% | 10% - 30% (GC/VM) | **0% (Native Machine Code)** |
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
