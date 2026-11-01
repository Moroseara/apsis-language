# Memory Model: Mutable Value Semantics (MVS)

## 1. แกนหลักของ MVS (Core Concept)

ระบบหน่วยความจำยึดหลักการ **"ทุกสิ่งคือ Value (ค่า) อิสระ แต่สามารถแก้ไขได้แบบ In-place"**

- **Logical Independence**: การกำหนดค่า `b = a` ในเชิงตรรกะคือการสร้างสำเนาที่เป็นอิสระจากกัน การแก้ `b` จะไม่มีผลต่อ `a` โดยเด็ดขาด
- **No Shared Mutable Aliasing**: ห้ามไม่ให้พอยน์เตอร์สองตัวชี้ไปยังหน่วยความจำที่แก้ไขได้ก้อนเดียวกันในเวลาเดียวกัน

```
let a = [1, 2, 3]
let mut b = a        ## ในทางตรรกะ b คือสำเนาใหม่
b.push(4)            ## a ยังเป็น [1, 2, 3] เสมอ — ไม่มี "ใครแก้ List ของเราลับหลัง" แบบ Python
```

---

## 2. กฎเหล็ก Law of Exclusivity

ตัวแปรใดๆ ที่กำลังถูก "เขียน/แก้ไข" จะต้องมีสิทธิ์เข้าถึงแต่เพียงผู้เดียว (Exclusive Access) ณ ช่วงเวลานั้น

```
[ อ่าน (Read) ]    ──► สามารถมีผู้อ่านพร้อมกันได้หลายคน (Shared Immutable Access)
[ เขียน (Modify) ]  ──► มีผู้เข้าถึงได้เพียงคนเดียวเท่านั้น (Exclusive `mut`)
```

- ระหว่างที่ตัวแปรถูกส่งเข้าไปในฟังก์ชันแบบ `mut` (ดู Syntax §3) ฟังก์ชันผู้เรียกจะไม่สามารถอ่านหรือเขียนตัวแปรนั้นได้ จนกว่าฟังก์ชันที่ถูกเรียกจะทำงานเสร็จ
- กฎนี้ถูกตรวจสอบตั้งแต่ขั้นตอน **Compile-Time** สำหรับ Value ทั้งหมด จึงไม่มี Runtime Overhead

```
fn transfer(mut from Account, mut to Account, amount i64) { ... }

let mut acc = Account(balance: 100)
transfer(mut acc, mut acc, 50)   ## ❌ Compile Error: `acc` ถูกยืมแบบ mut ซ้อนกันสองครั้ง
```

---

## 3. การเพิ่มประสิทธิภาพทางกายภาพ (Physical Optimization)

แม้ในทางตรรกะจะเหมือนการ Copy ข้อมูล แต่ตัวคอมไพเลอร์จะหลีกเลี่ยงการก๊อปปี้จริงด้วยเทคนิค:

1. **In-place Mutation via Uniqueness Analysis**:
   - ถ้าคอมไพเลอร์วิเคราะห์แล้วว่าข้อมูลนั้นมีผู้ถือครองอยู่เพียงตัวเดียว (Unique) การแก้ไขจะกระทำลงบนหน่วยความจำเดิมทันที ($O(1)$)
2. **Copy-on-Write (CoW) สำหรับ Dynamic Buffers**:
   - สำหรับโครงสร้างข้อมูลแบบไดนามิก เช่น Array หรือ String หากยังไม่มีการแก้ไขจะแชร์ Buffer ร่วมกัน เมื่อเริ่มมีการแก้ไขจึงจะทำการ Copy Buffer จริง
3. **Deterministic Destruction (RAII)**:
   - ไม่ต้องมี Garbage Collector เมื่อตัวแปรหลุดจาก Scope คอมไพเลอร์จะแทรกโค้ดทำลาย (Deallocation) ลงในตำแหน่งนั้นอย่างแน่นอน
4. **Arena Allocator Support**:
   - รองรับ Memory Arena สำหรับการจัดสรรหน่วยความจำแบบเป็นกลุ่มตาม Lifecycle ของงาน (เช่น 1 หน้าจอ UI หรือ 1 Request) แล้วเคลียร์ทิ้งทั้งก้อนในครั้งเดียว

---

## 4. ต้นทุนจริงของ Copy-on-Write

CoW ไม่ฟรี — Buffer ที่แชร์ได้ต้องมี **Reference Count** ติดอยู่ที่หัว Buffer:

| จังหวะ | สิ่งที่เกิดขึ้น | ต้นทุน |
| :--- | :--- | :--- |
| `let b = a` (Buffer บน Heap) | เพิ่ม Ref Count 1 ครั้ง | 1 คำสั่ง (ไม่ Copy ข้อมูล) |
| แก้ไข `b` | เช็คว่า Ref Count == 1 ไหม → ถ้าใช่แก้ในที่เดิม, ถ้าไม่ใช่ Copy ก่อน | 1 Branch ต่อการแก้ไข / Copy ทั้ง Buffer ครั้งแรก |
| ตัวแปรหลุด Scope | ลด Ref Count, ถ้าเป็น 0 คืนหน่วยความจำ | 1 คำสั่ง |
| ค่าที่เก็บบน Stack (SBO, Syntax §36) | ไม่มี Ref Count | 0 |

กฎที่ทำให้ต้นทุนนี้ต่ำ:
- **ส่วนใหญ่ถูกตัดทิ้งตอนคอมไพล์**: Uniqueness Analysis พิสูจน์ได้ว่าตัวแปรส่วนใหญ่ถูกถือโดยคนเดียว จึงไม่ต้องเช็ค Ref Count เลย
- **`mut` Parameter ไม่เพิ่ม Ref Count**: การส่ง `mut x` เป็นการยืมแบบ Exclusive ไม่ใช่การแชร์
- **Ref Count ของ Buffer เป็นแบบ Non-atomic**: เพราะ Value ข้าม Thread ได้ด้วยการ **Move** ผ่าน Channel (Syntax §18) เท่านั้น ไม่มีสอง Thread ถือ Buffer เดียวกัน จึงไม่ต้องจ่ายราคา Atomic

---

## 5. เขียนเหมือนมี GC โดยไม่มี GC (GC-like Ergonomics)

เป้าหมายสำคัญที่สุดของ Memory Model คือ **ผู้ใช้ต้องรู้สึกเหมือนเขียนภาษาที่มี GC** — ไม่มี Lifetime, ไม่มี `clone()`, ไม่มี `Box`, ไม่มี `Rc` / `Weak` — แต่ได้ผลลัพธ์เป็น Native Code ที่ไม่มี GC

Apsis **ไม่มี Reference Type ที่ผู้ใช้จับต้องได้เลย** (ไม่มี `Shared[T]`, `Weak[T]`, `Rc`, `Arc`) ทุกอย่างเป็น Value ความสะดวกแบบภาษา GC มาจาก 7 กลไกต่อไปนี้แทน:

### 5.1 ไม่มี Lifetime: Reference เป็น "พลเมืองชั้นสอง" (Second-class References)

การยืมแบบ `mut` (และการอ่านแบบไม่ Copy) **มีอยู่แค่ระหว่างการเรียกฟังก์ชันหรือภายใน Expression เดียว** — เก็บลง Struct ไม่ได้, คืนค่าออกจากฟังก์ชันไม่ได้, ใส่ใน List ไม่ได้

เพราะ Reference ไม่มีวันมีชีวิตยาวกว่าตัวเจ้าของ คอมไพเลอร์จึง **ไม่ต้องให้ผู้ใช้เขียน Lifetime เลย** (ไม่มี `'a` แบบ Rust) และไม่มีทางเกิด Dangling Pointer

```
fn longest(a string, b string) string {     ## ไม่ต้องมี <'a> — คืนค่าเป็น Value เสมอ
    return if a.length() > b.length() { a } else { b }
}
```

### 5.2 ไม่มี `clone()` และไม่มี `move`: คอมไพเลอร์ตัดสินใจเอง

`let b = a` ใช้ได้เสมอ ไม่มี Error แบบ "value moved" — คอมไพเลอร์เลือกวิธีที่ถูกที่สุดให้อัตโนมัติ:

| สถานการณ์ | สิ่งที่คอมไพเลอร์ทำ |
| :--- | :--- |
| `a` ไม่ถูกใช้อีกหลังจากนี้ (Last Use) | **Move** — ไม่ Copy อะไรเลย |
| `a` ยังถูกใช้ต่อ แต่ไม่มีใครแก้ไข | **แชร์ Buffer** (CoW ภายใน, §4) |
| `a` ยังถูกใช้ต่อ และ `b` ถูกแก้ไข | **Copy ตอนแก้ครั้งแรก** เท่านั้น |

```
let users = load_users()
let admins = users |> list.filter(|u| u.is_admin)
print(users.length())        ## ✅ ใช้ต่อได้ ไม่ต้อง users.clone()
send_to_thread(admins)       ## ✅ Last Use → Move ข้าม Thread โดยไม่ Copy
```

### 5.3 Recursive Types ได้ตรงๆ (ไม่ต้องมี `Box`)

Struct และ Enum อ้างถึงตัวเองได้ทันที คอมไพเลอร์ใส่ Indirection (เก็บบน Heap + CoW ภายใน) ให้เอง:

```
struct TreeNode {
    name string
    children [TreeNode] = []        ## Tree ทั้งต้นเป็น Value ก้อนเดียว
}

enum Expr {
    Num(f64)
    Add(Expr, Expr)                 ## ไม่ต้อง Box[Expr]
    Mul(Expr, Expr)
}

fn eval(e Expr) f64 {
    match e {
        Expr.Num(n) => { return n }
        Expr.Add(a, b) => { return eval(a) + eval(b) }
        Expr.Mul(a, b) => { return eval(a) * eval(b) }
    }
}
```

Tree, JSON, AST, Scene ที่เป็นลำดับชั้น **เขียนเป็น Value ได้ 100%** — ซึ่งครอบคลุมโครงสร้างข้อมูลส่วนใหญ่ในแอปจริง

### 5.4 แก้ไขลึกหลายชั้นได้ในที่เดิม (Mutable Projections)

Path ใดๆ ที่เริ่มจากตัวแปร `let mut` แก้ไขได้ตรงๆ โดยคอมไพเลอร์แก้ในหน่วยความจำเดิม (ไม่ Copy ออกมาแล้วเขียนกลับ):

```
let mut app = AppState.load()

app.users[3].profile.name = "Alice"          ## แก้ลึก 4 ชั้นในที่เดิม
app.tree.children[0].children.push(node)     ## เรียก Method แบบ mut ได้เลย ไม่ต้องเขียน mut ซ้ำ
rename(mut app.users[3].profile, "Bob")      ## ส่งส่วนย่อยเข้าฟังก์ชันแบบ mut ได้
```

- ที่จุดเรียก Method (`x.push(...)`) **ไม่ต้องเขียน `mut`** — ต้องเขียน `mut` เฉพาะตอนส่ง Argument เข้าฟังก์ชัน (Syntax §3)
- Struct ที่เขียน `[]` เองใช้ `__index_mut` เพื่อรองรับ Projection แบบนี้ (Syntax §20)

### 5.5 Closures: จับตัวแปรแบบที่ "ทำในสิ่งที่คาดหวัง"

| ชนิด Closure | จับตัวแปรแบบ | แก้ตัวแปรภายนอกได้ไหม |
| :--- | :--- | :--- |
| **ไม่หลุด Scope** (ส่งเข้า `for_each`, `map`, `scope`) | ยืมตรงๆ | ✅ ได้ |
| **หลุด Scope** (เก็บไว้เรียกทีหลัง, Event Handler, ส่งข้าม Thread) | Copy / Move ค่าเข้าไป | ❌ ไม่ได้ (คอมไพเลอร์บอกให้คืนค่าหรือส่ง Message แทน) |

```
let mut total = 0_i64
numbers.for_each(|n| total += n)       ## ✅ ไม่หลุด Scope → แก้ total ได้

let greeting = "Hi"
let handler = || print(greeting)       ## ✅ หลุด Scope → greeting ถูก Copy เข้าไป
```

คอมไพเลอร์รู้เองว่า Closure ไหนหลุด Scope ผู้ใช้ไม่ต้องเขียน `move` หรือคำกำกับใดๆ

### 5.6 Graph และการอ้างถึงกันข้ามโครงสร้าง: `Arena[T]` + `Id[T]`

สำหรับโครงสร้างที่ไม่ใช่ Tree (Graph, Node ที่ชี้กลับหาแม่, Entity ในเกมที่อ้างถึงกัน) ใช้ `Arena[T]` จาก Standard Library — เก็บ Object ทั้งหมดไว้ที่เดียว แล้วอ้างถึงกันด้วย `Id[T]`:

```
import "std/arena" { Arena, Id }

struct Node {
    name string
    parent Option[Id[Node]] = Option.None
    children [Id[Node]] = []
}

fn main() {
    let mut tree = Arena[Node]()
    let root = tree.add(Node(name: "root"))
    let child = tree.add(Node(name: "child", parent: Option.Some(root)))

    tree[root].children.push(child)               ## แก้ในที่เดิมผ่าน Projection (§5.4)

    if let Option.Some(p) = tree[child].parent {
        print(f"parent = {tree[p].name}")         ## ชี้กลับหาแม่ได้ ไม่ต้องมี Weak
    }

    tree.remove(child)
    let gone = tree.get(child)                    ## Option.None — Id เก่าใช้ไม่ได้แล้ว
}
```

คุณสมบัติของ `Id[T]`:
- **Type-safe**: `Id[Node]` ใช้กับ `Arena[User]` ไม่ได้ (Compile Error)
- **Generational**: Id เก็บ "รุ่น" ของช่องไว้ด้วย ถ้า Object ถูกลบแล้วมีของใหม่มาแทนที่ช่องเดิม Id เก่าจะได้ `Option.None` แทนการเข้าถึงข้อมูลผิดตัว (ไม่มี Use-after-free)
- **Copy ได้ฟรี**: เป็นแค่ตัวเลข 2 ตัว เก็บไว้ที่ไหนก็ได้ ส่งข้าม Thread ได้ ไม่ติดกฎ Exclusivity
- **ไม่มี Memory Leak จาก Cycle**: Node อ้างกันวนไปมากี่รอบก็ได้ เมื่อ `tree` หลุด Scope ทุกอย่างถูกคืนพร้อมกัน

### 5.7 State ที่หลายส่วนของโปรแกรมใช้ร่วมกัน

แทนที่จะให้หลายที่ถือ Reference ไปยัง Object เดียวกัน Apsis ใช้ 3 รูปแบบ:

**(1) เจ้าของเดียว + ส่ง `mut` ลงไป** — เหมาะกับ App State ทั่วไป:
```
fn main() {
    let mut app = AppState()
    loop {
        let event = poll_event()
        update(mut app, event)        ## ทุกส่วนแก้ State ผ่านการส่ง mut ลงไป
        render(app)
    }
}
```

**(2) Message แทน Callback ที่แก้ State** (แบบ Elm / SwiftUI / Iced) — เหมาะกับ UI:
```
enum Msg {
    Increment
    SetName(string)
}

fn view(state AppState) Widget {
    return Button(label: "+1", on_click: Msg.Increment)   ## Handler คืน Message ไม่ได้แก้ State เอง
}

fn update(mut state AppState, msg Msg) {
    match msg {
        Msg.Increment => { state.count += 1 }
        Msg.SetName(n) => { state.name = n }
    }
}
```

**(3) ข้าม Thread: Scoped Threads และ Channel**
```
import "std/thread" { scope }

## Scoped Thread: ทุก Thread จบก่อนออกจาก scope จึงยืม mut ส่วนย่อยไปได้อย่างปลอดภัย
scope(|s| {
    for mut chunk in pixels.chunks_mut(4096) {
        s.spawn(|| brighten(mut chunk))
    }
})

## งานที่อยู่ยาว: ให้ Thread เดียวเป็นเจ้าของ State แล้วคนอื่นส่ง Message ผ่าน Channel (Syntax §18)
```

ไม่มี `Mutex` ที่ต้องแชร์ไปมา ไม่มี Data Race และไม่มี Deadlock จาก Lock ซ้อนกัน

---

## 6. เปรียบเทียบกับ Python

| งาน | Python | Apsis |
| :--- | :--- | :--- |
| สร้าง List แล้วส่งต่อ | `b = a` (แชร์กัน แก้ b แล้ว a เปลี่ยน) | `let b = a` (เป็นอิสระต่อกันเสมอ) |
| Tree / JSON / AST | Class + List | `struct` / `enum` อ้างถึงตัวเอง (§5.3) |
| Node ชี้กลับหาแม่ | `self.parent = parent` | `parent Option[Id[Node]]` (§5.6) |
| แก้ข้อมูลลึกหลายชั้น | `app.users[3].name = "A"` | `app.users[3].name = "A"` (เหมือนกัน) |
| Callback ที่แก้ State | `lambda: state.inc()` | คืน `Msg` แล้ว `update` (§5.7) |
| คืนหน่วยความจำ | GC (เวลาไม่แน่นอน) | ทันทีที่หลุด Scope (Deterministic) |
| Cycle รั่ว | GC เก็บให้ | เกิดไม่ได้ตั้งแต่แรก |

---

## 7. เลือกใช้อะไรเมื่อไร (Decision Guide)

```
ข้อมูลเป็นลำดับชั้น (Tree, JSON, AST, Config)?
├─ ใช่ ──► Value ธรรมดา (struct / enum ที่อ้างถึงตัวเอง)      ◄── 90% ของโค้ด
└─ ไม่ใช่ (มีการอ้างถึงข้ามกัน / ชี้กลับ / วนกัน)
    └─► Arena[T] + Id[T]

หลายส่วนของโปรแกรมต้องแก้ State เดียวกัน?
├─ Thread เดียว ──► เจ้าของเดียว + ส่ง mut ลงไป / Message (Elm-style)
└─ หลาย Thread
    ├─ งานชั่วคราวแบ่งข้อมูลเป็นส่วน ──► scope + chunks_mut
    └─ State อยู่ยาว ──► Thread เจ้าของ + Channel
```

---

## 8. สิ่งที่แลกมา (Trade-offs ที่ยอมรับ)

- **Linked List / Graph แบบ Pointer ตรงๆ เขียนไม่ได้ใน Safe Code** → ใช้ `Arena[T]` แทน (เร็วกว่าด้วยเพราะข้อมูลเรียงติดกันใน Cache) หรือใช้ `unsafe` + Raw Pointer สำหรับผู้เขียน Standard Library
- **Observer Pattern แบบเก็บ Reference ของผู้ฟัง** → ใช้ Message / Channel แทน
- **Reference Count ยังมีอยู่ภายใน** Buffer ของ `string` / `[T]` / `{K: V}` เพื่อทำ CoW (§4) แต่เป็นรายละเอียดของคอมไพเลอร์ ผู้ใช้ไม่เห็นและไม่ต้องจัดการ และไม่มีทางเกิด Cycle เพราะ Value อ้างถึงตัวเองแบบวนไม่ได้
