# Compiler Architecture & Self-Hosting Roadmap

## 1. Compiler Pipeline

```
[ Source Code (.apsis / .lang) ]
               │
               ▼
   [ Frontend: Lexer + Parser ]       ──► สร้าง Abstract Syntax Tree (AST)
               │
               ▼
     [ High-Level IR (HIR) ]          ──► Type Checking & Law of Exclusivity Analysis
               │
               ▼
     [ Low-Level IR (LIR) ]           ──► Inlining, Copy-on-Write Optimization, Monomorphization
               │
               ▼
         [ LLVM IR ]                  ──► Extreme Loop Optimization, SIMD, No-Alias Attributes
               │
               ▼
[ Native Binary / .so / .dylib / .dll ]
```

---

## 2. Bootstrapping Strategy (เส้นทางสู่ Self-Hosting)

เราใช้กลยุทธ์ **"Rust-First to v1.0 Stable"** เพื่อความเสถียรและความเร็วในการพัฒนาภาษา โดยไม่ตกหลุมพรางของการรีบ Self-Host ก่อนเวลาอันควร:

1. **Phase 1: Rust-Powered Compiler (v0.1 -> v1.0 Stable)**:
   - พัฒนาคอมไพเลอร์ด้วย **Rust** ตลอดทั้งช่วงเริ่มต้นจนถึงเวอร์ชันเสถียร v1.0
   - ใช้ประโยชน์จาก Crates ของ Rust เช่น `logos`/`chumsky` (Lexer/Parser), `rowan` (Syntax Tree), `inkwell` (LLVM Bindings), และ `ariadne`/`miette` (Error Diagnostic สวยงาม)
   - มุ่งเป้าหมายที่การพัฒนา Standard Library, ระบบ FFI C-ABI, Tooling (LSP/Formatter), และการทดสอบเขียนแอปพลิเคชันจริง
   - **ปล่อย v1.0 Stable ที่คอมไพเลอร์เขียนด้วย Rust** ให้นักพัฒนานำไปใช้งานใน Production ได้อย่างมั่นใจ
2. **Phase 2: Self-Hosting Transition (v2.0 Long-term)**:
   - หลังจากภาษาและ Type System เสถียรสมบูรณ์แล้ว จึงเริ่มเขียนคอมไพเลอร์ด้วยภาษาของเราเอง (Self-Host)
   - ใช้ Compiler Rust จาก Phase 1 ทำหน้าที่เป็น Bootstrapper คอมไพล์ภาษาเราขึ้นมาเป็น Stage 1 Compiler
   - เข้าสู่กระบวนการ Full Self-Host Verification (Stage 2 และ Stage 3)
