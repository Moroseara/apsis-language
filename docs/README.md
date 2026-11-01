# Language Architecture Documentation

เอกสารการออกแบบสถาปัตยกรรมหลักสำหรับภาษาโปรแกรมยุคใหม่ (Safety, General-Purpose, No GC, C-Speed):

- [01. Vision & Goals](01_vision_and_goals.md) - วิสัยทัศน์ จุดเด่น และตำแหน่งแห่งที่ของภาษาเมื่อเทียบกับ C/Rust/Go
- [02. Memory Model (MVS)](02_memory_model_mvs.md) - สถาปัตยกรรม Mutable Value Semantics, Law of Exclusivity, และการจัดการหน่วยความจำ
- [03. ABI & Interoperability](03_interop_and_abi.md) - ระบบ C-ABI, FFI แบบ Zero-cost, การนำเข้าและส่งออกเป็น `.so` / `.dylib` / `.dll`
- [04. Compiler Roadmap](04_compiler_roadmap.md) - โครงสร้างคอมไพเลอร์, Pipeline, และแผนงาน Bootstrapping (Self-hosting)
- [05. Syntax Specification (Draft)](05_syntax_specification.md) - ร่างข้อกำหนดไวยากรณ์พื้นฐาน: ตัวแปร, ฟังก์ชัน, Generics, และ MVS Parameters
- [06. Module & Package System](06_module_system.md) - ระบบโมดูล, โครงสร้างโฟลเดอร์ `lib/`, และบทบาทของ `init.apsi`
- [07. Project, Manifest & Build Tool](07_project_and_package_manager.md) - ระบบโปรเจกต์ `apsis.kdl`, BuildTool ทางการแบบเปิดให้ Tool อื่นเสียบได้ทุกชั้น (Compiler Contract, Build Plan, Build Library, Plugins), และการจัดการ Dependencies
