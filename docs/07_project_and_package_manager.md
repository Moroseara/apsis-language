# Apsis: Project Manifest, BuildTool & Ecosystem Architecture

เอกสารข้อกำหนดระบบโปรเจกต์ `apsis.kdl`, การจัดการ Dependencies, และสถาปัตยกรรม BuildTool Ecosystem ของภาษา **Apsis**

---

## 1. ปรัชญาการออกแบบ: "Batteries Included, But Removable"

Apsis มี **BuildTool ทางการ (`apsis`) ที่ดีพอให้คนส่วนใหญ่ไม่ต้องหาเครื่องมืออื่น** (แบบ Cargo / Go) แต่ทุกสิ่งที่ `apsis` ทำ **ต้องทำได้ด้วยเครื่องมือของคนอื่นด้วย** (แบบที่ C ทำงานได้กับ Make, CMake, Ninja, Bazel)

หลักการ 3 ข้อ:

1. **No Privileged Tool**: `apsis` ไม่มี "ทางลับ" ในการคุยกับคอมไพเลอร์ ทุกอย่างที่มันใช้ (Flag ของ `apsic`, รูปแบบ Lockfile, Build Plan) เป็นสัญญาสาธารณะที่มีเอกสารและเวอร์ชันกำกับ
2. **The Official Tool is Just a Library + CLI**: ตรรกะของ `apsis` (อ่าน Manifest, Resolve Dependencies, สร้าง Build Graph, Cache) อยู่ในไลบรารี `apsis/build` ที่ใครก็ import ไปสร้าง Tool ของตัวเองได้ ไม่ต้องเขียนใหม่ตั้งแต่ศูนย์
3. **Manifest is the Shared Contract**: `apsis.kdl` และ `apsi.lock` คือภาษากลางของ Ecosystem — Tool ใดๆ ที่อ่านสองไฟล์นี้ถูกต้อง จะ Build แพ็กเกจของคนอื่นได้เสมอ ไม่ว่าเจ้าของแพ็กเกจจะใช้ Tool อะไร

> **ทำไมไม่ปล่อยให้ Community ทำ BuildTool เองทั้งหมด?** เพราะภาษาที่ไม่มี Tool ทางการ (C/C++) ลงเอยด้วย Ecosystem ที่แตกเป็นเสี่ยง — แต่ละไลบรารีใช้ Build System คนละแบบ และการดึงไลบรารีมาใช้คือฝันร้าย
> **ทำไมไม่ล็อคให้ใช้แค่ Tool ทางการ?** เพราะงานระดับสูง (Game Engine, Monorepo หลายภาษา, Embedded) มักมี Build System ของตัวเองอยู่แล้ว (Bazel, Buck2, CMake, Ninja) Apsis ต้องเสียบเข้าไปในนั้นได้โดยไม่ต้องอ้อมผ่าน `apsis`

---

## 2. สถาปัตยกรรมแบบชั้น (The Layered Architecture)

```
┌───────────────────────────────────────────────────────────────────────┐
│ Layer 4: Extensions                                                   │
│   build.apsi (Build Script) · apsis-<name> (Subcommand Plugins)       │
│   tool "<name>" { ... } ใน apsis.kdl                                  │
├───────────────────────────────────────────────────────────────────────┤
│ Layer 3: Official CLI (`apsis`)          │ Third-party Tools          │
│   apsis new / build / run / test / ...   │   apsis-bazel, CMake       │
│                                          │   module, Custom Tool      │
├──────────────────────────────────────────┴────────────────────────────┤
│ Layer 2: Build Library (`apsis/build`)                                │
│   Manifest Parser · Resolver · Lockfile · Fetcher · Graph · Cache     │
├───────────────────────────────────────────────────────────────────────┤
│ Layer 1: Build Plan (JSON, versioned schema)                          │
│   Graph ของ "ต้องเรียก apsic ด้วยอะไร ลำดับไหน ได้ไฟล์อะไร"              │
├───────────────────────────────────────────────────────────────────────┤
│ Layer 0: Core Compiler (`apsic`)                                      │
│   CLI สไตล์ Clang · JSON Diagnostics · Depfiles · --print queries     │
└───────────────────────────────────────────────────────────────────────┘
```

Third-party Tool **เลือกเสียบได้ทุกชั้น**:

| ต้องการ | เสียบที่ชั้น | ตัวอย่าง |
| :--- | :--- | :--- |
| คุมทุกอย่างเอง ไม่สน Manifest | Layer 0 | Makefile เรียก `apsic` ตรงๆ |
| ใช้ Build System เดิม แต่อยากได้ Dependency Resolution ของ Apsis | Layer 1 | Generator แปลง Build Plan เป็น `build.ninja` / `BUILD.bazel` |
| สร้าง BuildTool ใหม่ทั้งตัว (UX ต่าง, Remote Cache, ฯลฯ) | Layer 2 | Tool ที่ import `apsis/build` |
| แค่เพิ่มความสามารถให้ `apsis` | Layer 4 | `apsis-bundle`, `build.apsi` |

---

## 3. Layer 0: สัญญาของคอมไพเลอร์ (`apsic` Contract)

`apsic` ทำงานแบบ Clang/GCC — รับไฟล์, ไม่รู้จัก Manifest, ไม่โหลดอะไรจากเน็ต เพื่อให้ Tool ใดๆ ควบคุมได้ 100%

### 3.1 Flag หลัก (Stable ตั้งแต่ v1.0)

```
apsic src/main.apsi -o target/my_app              ## คอมไพล์ไฟล์ Entry เป็น Executable
apsic src/lib.apsi --crate-type sharedlib -o libfoo.so
apsic src/main.apsi --target aarch64-linux-gnu -O3

## บอกคอมไพเลอร์ว่า import "raylib" อยู่ที่ไหน (แทน lib/global)
apsic src/main.apsi --module raylib=deps/raylib/init.apsi

## เพิ่มโฟลเดอร์ค้นหา Module (ต่อจาก lib/global ตามลำดับใน doc 06)
apsic src/main.apsi -M vendor/

## ลิงก์กับไลบรารี C ภายนอก
apsic src/main.apsi -L /opt/sqlite/lib -l sqlite3
```

| Flag | หน้าที่ |
| :--- | :--- |
| `-o <path>` | ไฟล์ผลลัพธ์ |
| `--crate-type <bin\|staticlib\|sharedlib\|apsilib>` | ชนิดผลลัพธ์ (`apsilib` = ไลบรารี Apsis ที่คอมไพล์แล้ว พร้อม Metadata ของ Type) |
| `--emit <obj,asm,llvm-ir,apsilib,metadata>` | เลือกผลลัพธ์ระหว่างทาง (ใช้ทำ Incremental / Pipelining) |
| `--module <name>=<path>` | ผูกชื่อ import กับไฟล์ (Tool ใช้แทนการพึ่ง lib/global) |
| `-M <dir>` | เพิ่ม Module Search Path |
| `--target <triple>` | Cross-compile |
| `-O0 / -O1 / -O2 / -O3 / -Os` | ระดับการออปติไมซ์ |
| `--cfg <key>[=<value>]` | ส่งค่า Conditional Compilation (ใช้กับ `#[cfg(...)]`) |
| `--depfile <path>` | เขียนไฟล์ `.d` สไตล์ Make ระบุทุกไฟล์ที่การคอมไพล์ครั้งนี้อ่าน (รวม `include_bytes`) |
| `--message-format <human\|json>` | รูปแบบ Error/Warning |
| `@<file>` | Response File (อ่าน Flag จากไฟล์ เลี่ยงข้อจำกัดความยาว Command Line บน Windows) |

### 3.2 Query Commands (`--print`)

Tool ภายนอกไม่ควรต้องเดาว่าคอมไพเลอร์ติดตั้งไว้ที่ไหนหรือรองรับอะไร:

```
apsic --print version          ## 1.2.0
apsic --print sysroot          ## /usr/local/lib/apsis
apsic --print target-list      ## x86_64-linux-gnu, aarch64-apple-darwin, ...
apsic --print host-target      ## aarch64-apple-darwin
apsic --print cfg --target x86_64-windows-msvc   ## os="windows", arch="x86_64", ...
apsic --print imports src/main.apsi              ## รายชื่อ import ทั้งหมด (ไม่คอมไพล์)
```

### 3.3 JSON Diagnostics

`--message-format json` พ่น 1 JSON object ต่อ 1 บรรทัด (NDJSON) เพื่อให้ Tool/IDE แสดงผล Error เองได้:

```json
{"kind":"error","code":"E0412","message":"cannot mutate `score` declared with `let`",
 "spans":[{"file":"src/main.apsi","line":12,"col":15,"len":5,"label":"declared immutable here"}],
 "help":"declare it as `let mut score`"}
```

### 3.4 กฎที่ `apsic` สัญญาไว้

- **Hermetic**: ผลลัพธ์ขึ้นกับ Input ที่ส่งเข้าไปเท่านั้น (ไฟล์, Flag, เวอร์ชันคอมไพเลอร์) — ไม่อ่าน Environment Variable ลับ, ไม่ใส่เวลาลงไฟล์ผลลัพธ์ → **Reproducible Build** และ Cache ได้ปลอดภัย
- **Exit Code**: `0` สำเร็จ, `1` โค้ดผิด (Compile Error), `2` ใช้ Flag ผิด, `101` คอมไพเลอร์แครช (ICE)
- **Stability**: Flag ที่ไม่ขึ้นต้นด้วย `-Z` ถือเป็น Stable API เปลี่ยนแปลงได้เฉพาะ Major Version; Flag `-Z...` เป็น Experimental

---

## 4. Layer 1: Build Plan (สะพานไปหา Build System อื่น)

`apsis` แยกงานเป็น 2 จังหวะชัดเจน: **วางแผน (Plan)** แล้วค่อย **ลงมือ (Execute)** — และเปิดให้คนอื่นเอาแผนไปลงมือเองได้

```
apsis plan --format json > plan.json
apsis plan --bin apsic --profile release --target x86_64-linux-gnu
```

### 4.1 โครงสร้าง Build Plan

```json
{
  "schema": "apsis.plan/v1",
  "compiler": { "path": "/usr/local/bin/apsic", "version": "1.2.0" },
  "profile": "release",
  "target": "x86_64-linux-gnu",
  "units": [
    {
      "id": "raylib@1.0.0#lib",
      "package": "raylib",
      "kind": "apsilib",
      "entry": "~/.apsis/cache/git/raylib-3f2a9c/src/init.apsi",
      "args": ["--crate-type", "apsilib", "-O3", "--target", "x86_64-linux-gnu"],
      "deps": [],
      "outputs": ["target/release/deps/libraylib-3f2a9c.apsilib"],
      "link": { "native": ["raylib", "m"] }
    },
    {
      "id": "apsis_suite@0.1.0#bin:apsic",
      "package": "apsis_suite",
      "kind": "bin",
      "entry": "src/compiler/main.apsi",
      "args": ["--crate-type", "bin", "-O3", "--module",
               "raylib=target/release/deps/libraylib-3f2a9c.apsilib"],
      "deps": ["raylib@1.0.0#lib", "apsis_suite@0.1.0#lib:apsis_core"],
      "outputs": ["target/release/apsic"]
    }
  ],
  "scripts": [
    { "id": "apsis_suite@0.1.0#build-script", "before": ["apsis_suite@0.1.0#bin:apsic"] }
  ]
}
```

- `units` คือ DAG — Tool ภายนอกเรียง Topological Order แล้วเรียก `apsic <entry> <args> -o <outputs>` ได้ทันที
- ทุก Path ที่ resolve แล้ว (Dependency ถูก Fetch ลงเครื่องเรียบร้อยตอน `apsis plan`) → Executor ภายนอก **ไม่ต้องยุ่งกับเน็ตหรือ Git เลย**
- `schema` มีเวอร์ชันกำกับ: เพิ่ม Field ใหม่ได้ใน v1 แต่ลบ/เปลี่ยนความหมายต้องขึ้น v2

### 4.2 Generator สำเร็จรูป

```
apsis gen ninja      ## สร้าง build.ninja จาก Plan
apsis gen compdb     ## สร้าง compile_commands.json สำหรับ LSP / Editor
apsis gen make       ## สร้าง Makefile (สำหรับ CI ที่มีแค่ make)
```

Generator พวกนี้เป็นแค่ตัวแปลง Plan → ไฟล์อื่น ใครจะเขียน `apsis-gen-bazel` หรือ `apsis-gen-cmake` เพิ่มก็ใช้ Plan ตัวเดียวกัน

---

## 5. Layer 2: Build Library (`apsis/build`)

ไลบรารีที่ `apsis` CLI ใช้เอง เปิดเป็น Public API ใน Standard Distribution:

```
import "apsis/build" { Workspace, Manifest, Resolver, Lockfile, Plan, Executor }
```

| Module | หน้าที่ |
| :--- | :--- |
| `apsis/build/manifest` | อ่าน/ตรวจ `apsis.kdl` เป็น Struct ที่ Type-safe (รวม Section `tool` ของ Tool อื่น) |
| `apsis/build/resolve` | Resolve เวอร์ชัน Dependencies, อ่าน/เขียน `apsi.lock` |
| `apsis/build/fetch` | ดาวน์โหลด Git / Path / Registry ลง Cache กลาง (`~/.apsis/cache`) พร้อมตรวจ Checksum |
| `apsis/build/plan` | สร้าง Build Plan จาก Workspace |
| `apsis/build/exec` | Executor แบบ Parallel พร้อม Incremental Cache (ใช้ Depfile จาก `apsic`) |
| `apsis/build/toolchain` | หา `apsic` และถาม `--print` ต่างๆ |

### ตัวอย่าง: Custom BuildTool ที่ส่งงานไป Remote Build Farm

```
## my-remote-build/src/main.apsi
import "apsis/build" { Workspace, Plan }
import "./farm" { FarmClient }

fn main() Result[void, Error] {
    let ws = Workspace.load(".")?               ## อ่าน apsis.kdl + apsi.lock
    let plan = Plan.create(ws, profile: "release")?

    let mut farm = FarmClient.connect("build-farm.internal:9000")?
    for unit in plan.topological() {
        farm.submit(unit.entry, unit.args, unit.outputs)?
    }
    farm.wait_all()?
    return Result.Ok(())
}
```

Tool นี้ได้ Dependency Resolution, Lockfile, และ Build Graph ที่ **ตรงกับ `apsis` ทุกประการ** โดยเขียนเองแค่ส่วนที่ต่าง (การส่งงาน)

---

## 6. Layer 3: Official CLI (`apsis`)

### 6.1 Manifest พื้นฐาน (`apsis.kdl`)

> **รูปแบบไฟล์**: `apsis.kdl` ใช้โครงสร้างแบบ KDL แต่ **Comment ใช้ `##` เหมือนซอร์สโค้ด Apsis** (Syntax §26) เพื่อให้ทั้ง Ecosystem ใช้ Comment แบบเดียวกัน — ภาษา Apsis สงวน `//` ไว้เป็นตัวดำเนินการหารปัดเศษ (`2 // 3`) Parser KDL ทั่วไปจึงอ่านไฟล์นี้ตรงๆ ไม่ได้ Tool ภายนอกควรใช้ `apsis/build/manifest` (§5) หรือ `apsis metadata --format json` แทนการเขียน Parser เอง

ใน `apsis.kdl` สามารถกำหนด **หลาย Executable (`bin`) และ Library (`lib`) ได้ในไฟล์เดียว** (เหมือน CMake ที่สั่ง `add_executable` ได้หลายตัว) ช่วยให้เราสร้างชุดเครื่องมือคอมไพเลอร์ เช่น `apsic` + `apsis-analyzer` + `apsis-fmt` ได้พร้อมกันในการ Build รอบเดียว:

```kdl
project name="apsis_suite" version="0.1.0"

## 1. Executable ตัวที่ 1: ตัวคอมไพเลอร์หลัก
bin "apsic" {
    entry "src/compiler/main.apsi"
}

## 2. Executable ตัวที่ 2: Language Server / Analyzer
bin "apsis-analyzer" {
    entry "src/analyzer/main.apsi"
}

## 3. Executable ตัวที่ 3: Formatter
bin "apsis-fmt" {
    entry "src/formatter/main.apsi"
}

## 4. Shared Core Library ที่ทุกตัวดึงไปใช้ร่วมกัน
lib "apsis_core" type="staticlib" {
    entry "src/core/init.apsi"
}

dependencies {
    raylib git="github.com/apsi-lang/raylib" tag="v1.0.0"
}

build {
    mode "standalone"    ## Static Link ทุก binary รวม dependencies เป็นไฟล์เดี่ยว
    opt_level 3
}
```

### การสั่ง Build:
* `apsis build` $\rightarrow$ คอมไพล์ Executables ทุกตัวออกมาพร้อมกันใน `target/dist/`
* `apsis build --bin apsic` $\rightarrow$ สั่งคอมไพล์เฉพาะ Executable `apsic`
* `apsis run --bin apsis-analyzer` $\rightarrow$ เลือกรันเฉพาะตัวที่ต้องการ

### 6.2 Manifest ขั้นสูง: exe + dll / Plugin / Profile / Platform

```kdl
project name="my_game" version="0.3.0"

## ตัวเกมหลัก โหลด Engine แบบ Dynamic
bin "my_game" {
    entry "src/main.apsi"
    link "engine"                     ## ลิงก์กับ lib "engine" ด้านล่างแบบ Dynamic
}

## Engine เป็น Shared Library (.so / .dylib / .dll) ใช้ C-ABI ตาม doc 03
lib "engine" type="sharedlib" {
    entry "src/engine/init.apsi"
}

## Plugin โหลดตอน Runtime ผ่าน dyn Trait (ดู Syntax §33)
lib "plugin_audio" type="sharedlib" {
    entry "plugins/audio/init.apsi"
    install "plugins/"                ## วางไว้ที่ target/dist/plugins/
}

dependencies {
    raylib git="github.com/apsi-lang/raylib" tag="v1.0.0"
    math path="../libs/math"
}

## Dependency เฉพาะบาง Platform
dependencies platform="windows" {
    win32 git="github.com/apsi-lang/win32" tag="v0.4.0"
}

## Dependency ที่ใช้แค่ตอน test หรือใน build.apsi
dev-dependencies {
    bench git="github.com/apsi-lang/bench" tag="v0.2.0"
}
build-dependencies {
    shaderc git="github.com/apsi-lang/shaderc" tag="v1.1.0"
}

profile "dev" {
    opt_level 0
    debug_info #true
}
profile "release" {
    opt_level 3
    lto "thin"
    strip #true
}
```

### 6.3 Workspace (หลายแพ็กเกจในโปรเจกต์เดียว)

```kdl
## apsis.kdl ที่ Root ของ Monorepo
workspace {
    members "compiler" "analyzer" "fmt" "libs/*"
}
```

ทุก Member ใช้ `apsi.lock` และ `target/` ร่วมกันที่ Root — เหมาะกับโปรเจกต์ชุดเครื่องมือภาษา Apsis เอง

### 6.4 คำสั่ง CLI

| คำสั่ง | คำอธิบาย |
| :--- | :--- |
| `apsis new <name>` | สร้างโปรเจกต์ใหม่พร้อมไฟล์ `apsis.kdl` และ `src/main.apsi` |
| `apsis build` | คอมไพล์โปรเจกต์ตามที่กำหนดใน `apsis.kdl` |
| `apsis run` | คอมไพล์และรันโปรแกรมทันที |
| `apsis test` | สั่งรัน Unit Test ทุกฟังก์ชันที่มี `#[test]` |
| `apsis fetch` | ดาวน์โหลด Dependencies ทั้งหมดลง Cache (สำหรับ Build แบบ Offline) |
| `apsis update` | อัปเดตเวอร์ชันใน `apsi.lock` |
| `apsis plan` | พ่น Build Plan (Layer 1) ออกมาเป็น JSON |
| `apsis metadata` | พ่นเนื้อหา `apsis.kdl` ที่ parse แล้วเป็น JSON (สำหรับ Tool ที่ไม่อยากเขียน Parser เอง) |
| `apsis gen <ninja\|make\|compdb>` | แปลง Build Plan เป็นไฟล์ของ Build System อื่น |
| `apsis clean` | ลบ `target/` |
| `apsis <name>` | ถ้าไม่ใช่คำสั่งในตัว จะเรียก `apsis-<name>` บน PATH (ดู §7.2) |
| `apsic <file> [flags]` | สั่งคอมไพล์ระดับ Low-Level แบบตรงไปตรงมา (สไตล์ Clang/GCC) |

---

## 7. Layer 4: จุดขยาย (Extension Points)

### 7.1 Build Script (`build.apsi`)

สำหรับงานที่ Manifest ประกาศไม่ได้: Code Generation, คอมไพล์ Shader, คอมไพล์โค้ด C ประกอบ, ตรวจหาไลบรารีในเครื่อง — **เขียนด้วยภาษา Apsis เอง** ไม่ต้องใช้ภาษาที่สอง (ไม่มี CMake Language, ไม่มี Shell Script)

```kdl
project name="my_game" version="0.3.0" {
    build-script "build.apsi"
}
```

```
## build.apsi
import "apsis/build/script" { Script }
import "shaderc" { compile_glsl }

pub fn build(mut s Script) Result[void, Error] {
    ## 1. ประกาศ Input → ถ้าไฟล์เหล่านี้ไม่เปลี่ยน จะไม่รันสคริปต์ซ้ำ
    s.watch("assets/shaders/")

    ## 2. สร้างไฟล์ลงใน out_dir เท่านั้น (ห้ามเขียนทับ src/)
    for file in s.glob("assets/shaders/*.glsl") {
        let spirv = compile_glsl(s.read(file)?)?
        s.write(s.out_dir() / f"{file.stem()}.spv", spirv)?
    }

    ## 3. ส่งค่ากลับไปให้การคอมไพล์หลัก
    s.cfg("has_vulkan")                       ## ใช้กับ #[cfg(has_vulkan)]
    s.link_native("vulkan")                   ## เพิ่ม -l vulkan
    s.module("generated", s.out_dir() / "shaders.apsi")

    return Result.Ok(())
}
```

**กฎของ Build Script** (เพื่อให้ Cache และ Tool อื่นทำงานถูกต้อง):

| กฎ | เหตุผล |
| :--- | :--- |
| เขียนไฟล์ได้เฉพาะใน `s.out_dir()` | ไม่ทำลายซอร์สโค้ด, ลบ `target/` แล้วได้สถานะสะอาด |
| ต้องประกาศทุก Input ผ่าน `s.watch()` / `s.read()` | Tool รู้ว่าเมื่อไรต้องรันใหม่ |
| ไม่มี Network Access โดย Default | Build ต้อง Reproducible และ Offline ได้ |
| ผลลัพธ์ส่งกลับผ่าน API (`cfg`, `link_native`, `module`) เท่านั้น | Tool ภายนอกอ่านผลลัพธ์ได้เป็น JSON ไม่ต้อง parse stdout |

**สำหรับ Third-party Tool**: Build Script คอมไพล์เป็น Executable ธรรมดาได้ (`apsis plan` จะบอก unit ของมันใน `scripts`) รันโดยส่ง Environment ต่อไปนี้ แล้วอ่าน JSON ที่สคริปต์พ่นออก stdout:

```
APSIS_OUT_DIR=target/release/build/my_game-1a2b/out
APSIS_TARGET=x86_64-linux-gnu
APSIS_HOST=aarch64-apple-darwin
APSIS_PROFILE=release
APSIS_MANIFEST_DIR=/home/me/my_game
```

```json
{"schema":"apsis.script/v1","cfg":["has_vulkan"],"link_native":["vulkan"],
 "modules":{"generated":"target/release/build/my_game-1a2b/out/shaders.apsi"},
 "watch":["assets/shaders/"]}
```

### 7.2 Subcommand Plugins (`apsis-<name>`)

แบบเดียวกับ `git` และ `cargo`: ถ้าผู้ใช้พิมพ์ `apsis bundle` แล้วไม่มีคำสั่ง `bundle` ในตัว `apsis` จะค้นหา Executable ชื่อ `apsis-bundle` บน PATH (หรือใน `~/.apsis/bin`) แล้วเรียกมัน:

```
apsis install apsis-bundle     ## ติดตั้ง Tool จาก Git/Registry ลง ~/.apsis/bin
apsis bundle --format dmg      ## ── เรียก ──► apsis-bundle bundle --format dmg
```

Plugin จะได้รับ Environment `APSIS=/path/to/apsis` และ `APSIS_MANIFEST=/path/to/apsis.kdl` เพื่อเรียก `apsis plan` หรือ import `apsis/build` ต่อได้ทันที

ตัวอย่าง Plugin ที่คาดว่า Community จะสร้าง: `apsis-bundle` (ทำ .app / .msi / .deb), `apsis-wasm`, `apsis-android`, `apsis-bindgen` (สร้าง Header C / Binding Python จาก `extern "C"` exports), `apsis-audit`

### 7.3 Section `tool` ใน Manifest

Tool อื่นเก็บ Config ของตัวเองใน `apsis.kdl` ได้โดยไม่ต้องสร้างไฟล์ใหม่ — `apsis` จะ **ไม่ตีความและไม่ Error** กับ Section นี้:

```kdl
tool "bundle" {
    icon "assets/icon.png"
    identifier "com.example.mygame"
}

tool "bazel" {
    visibility "//visibility:public"
}
```

อ่านได้ผ่าน `manifest.tool("bundle")` ใน `apsis/build/manifest` — ชื่อ Section ควรตรงกับชื่อ Tool เพื่อไม่ให้ชนกัน

---

## 8. สิ่งที่ BuildTool ทุกตัวต้องเคารพ (Conformance Rules)

เพื่อให้แพ็กเกจที่ Build ด้วย `apsis` ใช้กับ Tool อื่นได้ และกลับกัน Tool ที่อ้างว่า "รองรับ Apsis" ต้อง:

1. **อ่าน `apsis.kdl` ตามความหมายใน Spec นี้** — Section ที่ไม่รู้จักต้องเตือน ไม่ใช่เงียบ (ยกเว้น `tool`)
2. **เคารพ `apsi.lock`** — ห้ามใช้เวอร์ชัน Dependency ต่างจาก Lockfile โดยไม่บอกผู้ใช้
3. **ใช้กฎ Module Resolution ตาม doc 06** — หรือส่ง `--module` ให้ `apsic` แทน ห้ามแก้พฤติกรรม import
4. **รัน `build.apsi` ตามสัญญาใน §7.1** — Environment และ JSON Output เดียวกัน
5. **ไม่ใส่ Flag ลับ** — ทุก Flag ที่ส่งให้ `apsic` ต้องมาจาก Manifest, Profile หรือคำสั่งผู้ใช้

Standard Distribution มีชุดทดสอบ **`apsis-conformance`** (โปรเจกต์ตัวอย่าง + ผลลัพธ์ที่คาดหวัง) ให้ผู้สร้าง Tool รันตรวจได้เอง

### Versioning ของสัญญาสาธารณะ

| สัญญา | ระบุเวอร์ชันด้วย |
| :--- | :--- |
| Flag ของ `apsic` | เวอร์ชันคอมไพเลอร์ (SemVer) |
| `apsis.kdl` | `apsis-version "1"` ใน Manifest (ไม่ใส่ = เวอร์ชันล่าสุดที่ Tool รู้จัก) |
| `apsi.lock` | Field `version` บรรทัดแรก |
| Build Plan / Script Output | Field `schema` (`apsis.plan/v1`, `apsis.script/v1`) |

---

## 9. ตัวอย่าง: Build โดยไม่ใช้ `apsis` เลย

### Makefile ล้วน (Layer 0)

```make
APSIC  ?= apsic
FLAGS  := -O2 --depfile $@.d

target/my_app: src/main.apsi
	$(APSIC) $< $(FLAGS) --module math=../libs/math/init.apsi -o $@

-include target/my_app.d      ## Depfile บอก Make ว่าไฟล์ .apsi ไหนเปลี่ยนต้อง Build ใหม่
```

### Ninja ผ่าน Build Plan (Layer 1)

```
apsis fetch                  ## ดึง Dependencies ครั้งเดียว
apsis gen ninja              ## สร้าง build.ninja
ninja                        ## ไม่ต้องใช้ apsis อีกต่อไปจนกว่า Manifest จะเปลี่ยน
```

---

## 10. โครงสร้างโฟลเดอร์โปรเจกต์มาตรฐาน

```
my_app/
├── apsis.kdl                 ## Manifest ควบคุมโปรเจกต์
├── apsi.lock                 ## การันตีความปลอดภัยและเวอร์ชันของโค้ด (Commit เข้า Git)
├── build.apsi                ## (ไม่บังคับ) Build Script
├── src/
│   ├── main.apsi             ## จุดเริ่มต้นโปรแกรม
│   └── helper.apsi
└── target/                   ## ทุกอย่างที่ Build สร้าง (ไม่ Commit / ลบได้เสมอ)
    ├── dev/                  ## ผลลัพธ์ Profile dev + ไฟล์ Incremental
    ├── release/
    │   ├── deps/             ## Dependencies ที่คอมไพล์แล้ว (.apsilib)
    │   └── build/            ## out_dir ของ build.apsi
    └── dist/                 ## ไฟล์พร้อมแจกจ่าย
        └── my_app            ## Standalone Executable (จบในก้อนเดียว)
```

---

## 11. Roadmap ของ BuildTool

| ระยะ | สิ่งที่ส่งมอบ | ชั้นที่เปิดให้ Community |
| :--- | :--- | :--- |
| **v0.x** | `apsic` + `apsis build/run/test` พื้นฐาน, `bin`/`lib`, Git/Path Dependencies | Layer 0 (Flag อาจยังเปลี่ยน) |
| **v0.x ปลาย** | `apsis plan`, `apsis gen compdb`, JSON Diagnostics, Depfile | Layer 0–1 |
| **v1.0** | Lock สัญญา Layer 0–1, Profiles, Workspace, `sharedlib`, Subcommand Plugins | Layer 0, 1, 4 (Plugins) |
| **v1.x** | `build.apsi`, `apsis/build` เป็น Public Library, `apsis-conformance` | ทุกชั้น |
| **v2.0** | Self-hosted `apsis` เขียนด้วย Apsis ทั้งหมด (ตาม doc 04) | — |

> หมายเหตุ: ในช่วง Phase 1 ของ doc 04 ที่คอมไพเลอร์เขียนด้วย Rust ไลบรารี `apsis/build` จะเปิดให้ใช้ผ่าน C-ABI (`libapsis_build`) ก่อน แล้วจึงมี Binding เป็นภาษา Apsis เมื่อ Self-host
