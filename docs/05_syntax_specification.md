# Syntax Specification (Draft v0.1)

## 1. Variable & Constant Declarations

ภาษาใช้หลักการ **Immutable by Default** เพื่อความปลอดภัยสูงสุด

```
## Compile-time Constant (คำนวณตอน Compile ไม่กิน Memory Runtime)
const MAX_BUFFER_SIZE i64 = 4096
const PI f64 = 3.1415926535

fn example() {
    ## Immutable Variable (อ่านอย่างเดียว แก้ไขไม่ได้)
    let name = "Alice"
    let age i64 = 20

    ## Mutable Variable (ต้องใส่ mut ชัดเจนเพื่ออนุญาตให้แก้ไขได้)
    let mut counter = 0
    counter += 1
}
```

---

## 2. Function Definitions & Generics

ใช้คีย์เวิร์ด `fn` ผสานกับการใช้ `[T]` สำหรับ Generics และรองรับ Default Arguments + Named Arguments

### ไวยากรณ์พื้นฐาน:
```
fn add(a i64, b i64 = 0) i64 {
    return a + b
}
```

### Generics & Default Values:
```
fn get_or_default[T](item T, fallback T = T.default()) T {
    if item == T.default() {
        return fallback
    }
    return item
}
```

### Variadic Functions (พารามิเตอร์แบบไม่จำกัดจำนวน `...`):
ฟังก์ชันสามารถรับ Argument ได้ไม่จำกัดจำนวน โดยคอมไพเลอร์จะมองพารามิเตอร์นั้นเป็น Zero-Copy Slice `[T]`:

```
## 1. Type-Safe Variadic ใน Apsis
fn sum(numbers ...i64) i64 {
    let mut total = 0_i64
    for n in numbers {
        total += n
    }
    return total
}

fn print_all(messages ...string) {
    for msg in messages {
        print(msg)
    }
}

fn example() {
    let total = sum(1, 2, 3, 4, 5) ## ส่งกี่ตัวก็ได้
    print_all("Hello", "World", "from", "Apsis")
}

## 2. C-FFI Variadic (สำหรับเรียก libc printf)
extern "C" {
    fn printf(format *const u8, ...) i32
}
```

### การเรียกใช้ฟังก์ชัน (Call Site):
```
## เรียกแบบปกติ
let res1 = add(10, 20)

## เรียกโดยละเว้นค่า default
let res2 = add(10)

## เรียกโดยระบุชื่อ Parameter (Named Arguments)
let res3 = add(b: 5, a: 10)

## ระบุ Generic แบบเจาะจง หรือให้ Compiler ทำ Type Inference อัตโนมัติ
let res4 = get_or_default[string]("Data", fallback: "Empty")
let res5 = get_or_default("Data", fallback: "Empty") ## Auto-inferred T = string
```

---

## 3. Mutable Value Semantics (MVS Parameter Passing)

ฟังก์ชันที่ต้องการแก้ไขค่าของ Argument ต้นทาง (In-place Mutation แบบ Exclusive) จะต้องระบุ `mut` ทั้งที่หัวฟังก์ชันและจุดเรียก:

```
fn increment(mut value i64, step i64 = 1) {
    value += step
}

fn main() {
    let mut score = 100
    
    ## บังคับใส่ mut ที่จุดเรียก เพื่อความชัดเจนในการอ่านโค้ด
    increment(mut score, step: 5)
    
    ## score จะมีค่าเป็น 105
}
```

### กฎความปลอดภัยของคอมไพเลอร์:
- ห้ามส่งตัวแปรที่ประกาศด้วย `let` (Immutable) เข้า Parameter ที่เป็น `mut`
- ในช่วงที่ตัวแปรถูกยืมผ่าน `mut` ตัวแปรนั้นจะถูกล็อค ห้ามนำไปอ่านหรือเขียนที่อื่นพร้อมกัน (Law of Exclusivity)

---

## 4. Structs, Packed Structs, and Methods

ภาษาใช้ `struct` เป็นหน่วยข้อมูลหลักที่เป็น Value Type พร้อมรองรับ `#[packed]` สำหรับงานด้าน Binary และโครงข่าย

```
struct Vector2 {
    x f64 = 0.0
    y f64 = 0.0

    ## Immutable Method
    fn length(self) f64 {
        return math.sqrt(self.x * self.x + self.y * self.y)
    }

    ## Mutable Method (MVS)
    fn add(mut self, other Vector2) {
        self.x += other.x
        self.y += other.y
    }
}

## โครงสร้างแบบ Packed: บังคับไม่มี Padding Byte (สำหรับ Network Packets / File Headers)
#[packed]
struct NetworkHeader {
    magic u16     ## 2 bytes
    version u8    ## 1 byte
    length u32    ## 4 bytes
}
```

---

## 5. Enums & Pattern Matching (`match`)

ภาษาไม่มีค่า `null` แต่ใช้ Algebraic Data Types (`enum` ที่บรรจุข้อมูลได้) ร่วมกับ `match` ที่บังคับตรวจสอบครบทุกเคส (Exhaustive Pattern Matching):

```
enum Option[T] {
    Some(T)
    None
}

enum Event {
    Quit
    KeyPress(key i64)
    MouseMove(x i64, y i64)
}

fn handle_event(event Event) {
    match event {
        Event.Quit => {
            print("Exiting...")
        }
        Event.KeyPress(k) => {
            print("Key pressed: {k}")
        }
        Event.MouseMove(x, y) => {
            print("Mouse at: {x}, {y}")
        }
    }
}
```

---

## 6. Control Flow (`if let`, `while let`, `for`, `while`)

รองรับการแกะค่า Pattern (Pattern Binding) พร้อมเงื่อนไขต่อเนื่องด้วยเครื่องหมายจุลภาค `,`

```
fn process_data(opt Option[i64]) {
    ## if let พร้อมเงื่อนไขหลายชั้น
    if let Option.Some(data) = opt, data > 10 {
        print("Valid high data: {data}")
    } else {
        print("Invalid or too low")
    }

    ## while let สำหรับการอ่านค่าซ้ำจนกว่าจะหมด
    while let Option.Some(msg) = queue.pop(), msg.is_active() {
        msg.process()
    }

    ## for .. in
    for item in [1, 2, 3, 4, 5] {
        print("Item: {item}")
    }
}
```

---

## 7. Attributes (`#[attributes]`)

ใช้สำหรับการสั่งการคอมไพเลอร์, การกำหนด Memory Layout, การส่งออก C-ABI, และการเชื่อมต่อ Compiler Internals:

- `#[packed]`: ปิด Memory Padding จัดเรียงไบต์ติดกัน
- `#[export]`: ส่งออกฟังก์ชันตามมาตรฐาน C-ABI (`extern "C"`) สำหรับทำ `.so`, `.dylib`, `.dll`
- `#[link("libname")]`: สั่ง Linker เชื่อมโยงกับ Dynamic Shared Library ภายนอก (เช่น libc, sqlite3, raylib)
- `#[inline]`: แนะนำให้คอมไพเลอร์ทำ Inlining เพื่อความเร็วสูงสุด
- `#[test]`: ประกาศฟังก์ชันสำหรับ Unit Test
- `#[intrinsic("name")]`: เชื่อมโยงฟังก์ชันเข้ากับคำสั่งฮาร์ดแวร์ของ CPU หรือคำสั่งภายในของคอมไพเลอร์โดยตรง (เช่น LLVM Intrinsics) โดยไม่ต้องเขียน Body ของฟังก์ชัน:

```
### เชื่อมต่อกับคำสั่งฮาร์ดแวร์ระดับ CPU ตรงๆ (Zero-Cost Hardware Primitive)
#[intrinsic("sqrt_f64")]
pub fn sqrt(val f64) f64

#[intrinsic("popcount_u64")]
pub fn count_ones(val u64) u64

#[intrinsic("atomic_cas_u64")]
pub fn compare_and_swap(ptr *mut u64, old_val u64, new_val u64) bool

#[intrinsic("breakpoint")]
pub fn breakpoint() void
```

---

## 8. Number Literals & Radixes (เลขฐานและตัวเลข)

รองรับการเขียนตัวเลขในหลากหลายฐาน, สัญกรณ์วิทยาศาสตร์, พร้อมเครื่องหมาย Underscore (`_`) เพื่อให้อ่านค่าง่าย:

```
## 1. ฐานสิบ (Decimal)
let decimal = 1_000_000
let float_val = 3.14159_f32

## 2. สัญกรณ์วิทยาศาสตร์ (Scientific / Exponential Notation: e / E)
let speed_of_light = 3.0e8       ## 3.0 x 10^8 = 300,000,000.0
let micro_value = 1.5e-4_f64     ## 1.5 x 10^-4 = 0.00015
let large_int = 1e2              ## 1 x 10^2 = 100.0 (Float เสมอเมื่อมี e)

## 3. ฐานสิบหก (Hexadecimal: 0x)
let hex = 0xFF_AA_00
let address = 0x7FFF_FFFF_FFFF

## 4. ฐานสอง (Binary: 0b) สำหรับงานระดับ Bitwise
let bitmask = 0b1111_0000
let flags = 0b0000_0001_u8

## 5. ฐานแปด (Octal: 0o) สำหรับงาน File Permissions
let file_mode = 0o755
```

---

## 9. Bitwise Operators (ตัวดำเนินการระดับบิต)

รองรับงานระดับ Low-Level และ Network/Binary Protocol ครบถ้วน:

| Operator | ความหมาย | ตัวอย่าง |
| :--- | :--- | :--- |
| `&` | Bitwise AND | `flags & 0b0001` |
| `\|` | Bitwise OR | `flags \| 0b0010` |
| `^` | Bitwise XOR | `a ^ b` |
| `~` | Bitwise NOT (Invert) | `~mask` |
| `<<` | Left Shift | `value << 2` |
| `>>` | Right Shift | `value >> 2` |
| `&=`, `\|=`, `^=`, `<<=`, `>>=` | Compound Bitwise Assignment | `mask &= ~0b0001` |

---

## 10. Pipeline Operator (`|>`)

ยกระดับความอ่านง่ายของโค้ดด้วย **Pipe Operator** สำหรับส่งผลลัพธ์จากซ้ายไปเป็น Argument แรกของฟังก์ชันทางขวา:

```
fn double(n i64) i64 {
    return n * 2
}

fn add(a i64, b i64) i64 {
    return a + b
}

fn main() {
    ## แบบเดิมที่ซ้อนกันจนอ่านยาก: add(double(5), 10)
    ## แบบใช้ Pipeline Operator:
    let result = 5
        |> double()       ## ได้ 10
        |> add(10)        ## ได้ 20

    print("Result: {result}")
}
```

### กลไก Zero-Cost ของ Pipe:
- Pipeline Operator จะถูก **Desugar (แปลงกลับ)** ให้เป็นฟังก์ชันคอลล์ธรรมดาตั้งแต่ตอน Parse/Compile
- **ไม่มี Overhead หรือตัวหุ้มใดๆ ในระดับ Runtime รันเร็วเท่าฟังก์ชันปกติ 100%**

---

## 11. Collections: Lists & Maps (โครงสร้างข้อมูลพื้นฐาน)

คอลเลกชันในภาษาเป็น **Value Type** ทั้งหมด (จัดการหน่วยความจำแบบ In-place และ Copy-on-Write ไร้ GC) พร้อมรองรับ **Trailing Comma** (ใส่เครื่องหมายจุลภาคคั่นตัวสุดท้ายได้ เพื่อ Git Diff ที่สะอาด):

### 1. Map (Dictionary / Key-Value Store)
- **Type Annotation**: ใช้ `{K: V}` หรือ `{K, V}`
- **Literal**: ใช้ปีกกา `{key: value,}`

```
## การประกาศ Map พร้อมกำหนด Type และรองรับ Trailing Comma
let scores {string: i64} = {
    "Alice": 100,
    "Bob": 85,
    "Charlie": 92,
}

## Map ที่มี Type Inference อัตโนมัติ
let mut config = {
    "timeout": 30,
    "retries": 3,
}

## แก้ไขค่าใน Map แบบ MVS (In-place Mutation)
config["retries"] = 5

## ประกาศ Map เปล่า
let empty_map {string: i64} = {}
```

### 2. List (Dynamic Array / Vector)
- **Type Annotation**: ใช้ `[T]`
- **Literal**: ใช้ก้ามปู `[item1, item2,]`

```
let names [string] = [
    "Alpha",
    "Beta",
    "Gamma",
]

let mut numbers = [1, 2, 3, 4]
numbers.push(5)
```

### 3. การเข้าถึงข้อมูลและการวนลูป (Map Iteration):
```
for key, value in scores {
    print("User: {key}, Score: {value}")
}

## ตรวจสอบว่ามี Key หรือไม่ด้วย if let
if let value = scores.get("Alice") {
    print("Alice's score is {value}")
}
```

---

## 12. Tuples & Destructuring

ใช้ `(T1, T2, ...)` สำหรับจับกลุ่มข้อมูลหลากหลายชนิดโดยไม่ต้องสร้าง Struct ชั่วคราว:

```
## 1. ประกาศ Tuple และ Type
let point (i64, i64) = (10, 20)
let single_item = (42,) ## ทูเพิล 1 ตัว ต้องมีจุลภาคต่อท้ายเพื่อแยกจากวงเล็บคำนวณ

## 2. คืนค่าหลายค่าจากฟังก์ชัน
fn get_user_status() (string, i64, bool) {
    return ("Morose", 100, true)
}

## 3. Tuple Destructuring
let (name, score, is_active) = get_user_status()

## 4. ไม่สนใจค่าบางตัวด้วย _ (Wildcard)
let (name, _, _) = get_user_status()
```

---

## 13. Closures (Anonymous Functions)

ใช้ไวยากรณ์ `|args|` ซึ่งเข้ากันได้ดีเยี่ยมกับ Pipeline Operator (`|>`):

```
let factor = 2

## 1. Single expression closure
let double = |x| x * factor

## 2. Multi-line closure with types
let calculate = |a i64, b i64| i64 {
    let temp = a * 2
    return temp + b
}

## 3. ใช้งานร่วมกับ Pipe Operator
let numbers = [1, 2, 3, 4, 5]
let evens = numbers
    |> list.filter(|x| x % 2 == 0)
    |> list.map(|x| x * 10)
```

---

## 14. Module & Import System

ภาษาใช้ระบบโมดูลที่มีโฟลเดอร์ `lib/` ติดตั้งไปพร้อมกับตัวคอมไพเลอร์ ทำให้การจัดการโค้ดและการแชร์ไลบรารีมีความยืดหยุ่นสูงสุด:

### โครงสร้าง `lib/` ของระบบคอมไพเลอร์:
```
lib/
├── builtin/             ## โหลดให้อัตโนมัติ (Prelude) เช่น print, assert, Option, Result ไม่ต้อง import
└── global/              ## แหล่งรวม Global Modules
    ├── math.apsi        ──► import "math" {*} (แบบไฟล์เดี่ยว)
    └── std/
        ├── math/        ──► import "std/math" {*} (แบบโฟลเดอร์แพ็กเกจ)
        │   ├── init.apsi     ## จุดเริ่มต้นหลักของแพ็กเกจ (Package Entrypoint)
        │   ├── matrix.apsi   ## ซับโมดูลย่อย
        │   └── vector.apsi   ## ซับโมดูลย่อย
        └── net.apsi     ──► import "std/net" {*}
```

### กลไกการทำงานของ `init.apsi` (Package Entrypoint):
เมื่อมีการสั่ง `import "std/math"` คอมไพเลอร์จะมีลำดับการค้นหาดังนี้:
1. ตรวจสอบไฟล์เดี่ยว `std/math.apsi`
2. หากเป็นโฟลเดอร์ จะเปิดไฟล์ **`std/math/init.apsi`** เป็นจุดเริ่มต้นหลักทันที
3. **การเข้าถึงซับโมดูล**: ยังคงเข้าถึงไฟล์ย่อยภายในได้โดยตรง เช่น `import "std/math/matrix" { Matrix }`
4. **Re-export จาก `init.apsi`**: ไฟล์ `init.apsi` สามารถนำเข้าและส่งออกต่อ (`pub import`) เพื่อให้ผู้เรียกจากภายนอก import ที่จุดเดียวแล้วได้ของครบทั้งหมด:

```
## ตัวอย่างโค้ดใน std/math/init.apsi
pub import "./matrix" { Matrix }
pub import "./vector" { Vector }

pub fn calculate_root(val f64) f64 {
    return val.sqrt()
}
```

### ไวยากรณ์การ Import:
```
## 1. นำเข้าจาก lib/global/ พร้อม Wildcard
import "math" { * }
import "std/net" { TcpListener, TcpStream }

## 2. Relative Path สำหรับไฟล์ภายในโปรเจกต์ปัจจุบัน
import "./models/user" { User }

## 3. Import พร้อมตั้งชื่อ Alias
import "./network/socket" as net
```

---

## 15. Traits (Zero-Cost Polymorphism)

ใช้ `trait` ในการกำหนดข้อกำหนดร่วมของ Structs โดยไม่มี Runtime Overhead (Compile-time Monomorphization):

```
trait Printable {
    fn to_string(self) string
}

struct User {
    name string
    age i64
}

## ผูก Trait เข้ากับ Struct
impl Printable for User {
    fn to_string(self) string {
        return f"User({self.name}, {self.age})"
    }
}

## ฟังก์ชันที่รับ Generic ที่มีเงื่อนไขตาม Trait
fn print_item[T: Printable](item T) {
    print(item.to_string())
}
```

---

## 16. Strings: f-strings, Multiline, Raw Strings, and Escapes

```
let name = "Morose"
let score = 99.5_f64

## 1. f-string ปกติ
let message = f"Player {name} has score {score:.1f}"

## 2. Multiline f-string (f""" ... """)
let query = f"""
SELECT * FROM users
WHERE name = '{name}'
AND score >= {score}
ORDER BY id DESC;
"""

## 3. Raw String (r"...") ไม่ตีความ Escape Character เหมาะสำหรับ Regex และ Windows Paths
let regex_pattern = r"C:\new_folder\test\d+"
let json_raw = r#"{"name": "Alice", "age": 20}"#

## 4. String Escape Sequences ที่รองรับ:
## \n (ขึ้นบรรทัดใหม่), \t (แท็บ), \r (แครี่เอจ), \\ (แบ็กสแลช), \" (เครื่องหมายคำพูด)
## \0 (Null byte), \x7F (Hex byte), \u{1F600} (Unicode Codepoint / Emoji)
let emoji_string = "Hello \u{1F600}\n"

## 5. Byte string สำหรับงาน Binary & Protocol
let ping_packet = b"PING\r\n"
```

---

## 17. Control & Safety Primitives: `defer`, `unsafe`, `?`, and `extern "C"`

```
## 1. Error Propagation ด้วยเครื่องหมาย ?
fn read_config(path string) Result[Config, FileError] {
    let file = File.open(path)?
    
    ## 2. defer: ทำงานตอนฟังก์ชันสิ้นสุดเสมอ (Deterministic Cleanup)
    defer file.close()

    let content = file.read_all()?
    return Result.Ok(Config.parse(content)?)
}

## 3. extern "C" Block: ประกาศฟังก์ชันภายนอกจาก libc หรือ Shared Libraries (.so, .dll)
#[link("c")]
extern "C" {
    fn puts(str *const u8) i32
    fn getpid() i32
}

## 4. unsafe Block: สำหรับเรียกใช้ฟังก์ชัน extern หรือจัดการ Raw Pointer (*const T, *mut T)
fn print_process_info() {
    unsafe {
        puts(b"Running in Apsis native process\0".as_ptr())
        let pid = getpid()
        print(f"PID: {pid}")
    }
}
```

---

## 18. Concurrency: Channels (Data-Race Free via MVS)

ใช้ Channel ในการส่งข้อมูลข้าม Thread โดยอาศัยคุณสมบัติ Value Movement ของ MVS ทำให้ปราศจาก Data Race 100%:

```
import "std/sync" { Channel }
import "std/thread" { spawn }

fn main() {
    let (tx, rx) = Channel[string].create()

    spawn(|| {
        let msg = "Hello from background thread!"
        tx.send(msg) ## msg ถูก Move ข้ามเธรดอย่างปลอดภัย
    })

    let received = rx.recv()
    print(f"Received: {received}")
}
```

---

## 19. Function & Method Overloading (การนิยามฟังก์ชันชื่อซ้ำ)

เพื่อความสะดวกในการพัฒนาแอปพลิเคชัน (Ergonomics) ภาษารองรับการนิยามฟังก์ชันและ Method ชื่อเดียวกันได้ ตราบใดที่ **Parameter Types หรือจำนวน Parameter แตกต่างกันอย่างชัดเจน**:

```
## 1. Function Overloading ตามประเภทข้อมูล
fn print_value(val i64) {
    print(f"Integer: {val}")
}

fn print_value(val string) {
    print(f"String: {val}")
}

fn print_value(val bool) {
    print(f"Boolean: {val}")
}

## 2. Overloading ร่วมกับ Named Arguments
struct Canvas {
    fn draw_circle(self, radius f64) {
        ## วาดที่จุด (0, 0)
    }

    fn draw_circle(self, x f64, y f64, radius f64) {
        ## วาดตามพิกัดที่กำหนด
    }
}

fn main() {
    print_value(100)      ## เรียก print_value(val i64)
    print_value("Hello")  ## เรียก print_value(val string)

    let canvas = Canvas()
    canvas.draw_circle(radius: 5.0)
    canvas.draw_circle(x: 10.0, y: 20.0, radius: 5.0)
}
```

### กฎการเลือกฟังก์ชันของคอมไพเลอร์ (Disambiguation):
1. เลือกฟังก์ชันที่มี Type ตรงกันทุกประการก่อน (Exact Match)
2. ป้องกันความกำกวม: หากมีฟังก์ชันที่มี Default Arguments ซ้อนทับกับอีกฟังก์ชันหนึ่งพอดี คอมไพเลอร์จะฟ้อง Compile Error ทันทีเพื่อความปลอดภัย

---

## 20. Operator Overloading & Special Functions (`__add`, `__init`, etc.)

ภาษา Apsis ใช้แนวคิด **Special Functions (Dunder Methods)** สำหรับการโอเวอร์โหลดตัวดำเนินการและกำหนดวงจรชีวิตของข้อมูล โดยคอมไพเลอร์จะทำการแปลงรูป (Desugar) และทำ Inlining ตรงเข้าสู่คำสั่ง CPU โดยไม่มี Overhead:

### 1. การโอเวอร์โหลดตัวดำเนินการทางคณิตศาสตร์:
เมื่อมีคำสั่ง `a + b` คอมไพเลอร์จะเรียกเมธอด `a.__add(b)` ทันที:

```
struct Vector2 {
    x f64
    y f64

    ## โอเวอร์โหลดเครื่องหมาย +
    fn __add(self, other Vector2) Vector2 {
        return Vector2(x: self.x + other.x, y: self.y + other.y)
    }

    ## โอเวอร์โหลดเครื่องหมาย -
    fn __sub(self, other Vector2) Vector2 {
        return Vector2(x: self.x - other.x, y: self.y - other.y)
    }

    ## โอเวอร์โหลดการหารปัดเศษลง //
    fn __floordiv(self, scalar f64) Vector2 {
        return Vector2(x: self.x // scalar, y: self.y // scalar)
    }

    ## โอเวอร์โหลดการเปรียบเทียบ ==
    fn __eq(self, other Vector2) bool {
        return self.x == other.x && self.y == other.y
    }
}
```

### 2. การเข้าถึง Index ด้วยก้ามปู `[]` (`__index`, `__set_index`):
```
struct IntBuffer {
    data [i64]

    ## เมื่อเรียก buffer[idx]
    fn __index(self, idx usize) i64 {
        return self.data[idx]
    }

    ## เมื่อเรียก buffer[idx] = val (MVS Exclusive Mutation)
    fn __set_index(mut self, idx usize, val i64) {
        self.data[idx] = val
    }
}
```

#### Mutable Projection (`__index_mut`):
เมื่อต้องการแก้ไข **ส่วนย่อยภายใน** สมาชิกที่เข้าถึงด้วย `[]` ในที่เดิม (เช่น `world[id].hp -= 10`) ให้กำหนด `__index_mut` ที่คืนค่าเป็น `mut T` — ค่านี้เป็น **Projection** ที่มีชีวิตอยู่แค่ใน Expression นั้น (เก็บลงตัวแปร/Struct หรือ return ต่อไม่ได้ จึงไม่ต้องมี Lifetime — ดู doc 02 §5.1):

```
struct Entities {
    items [Entity]

    fn __index_mut(mut self, idx usize) mut Entity {
        return mut self.items[idx]
    }
}

let mut world = Entities(items: load_entities())
world[3].hp -= 10                    ## เรียก __index_mut แล้วแก้ hp ในที่เดิม ไม่ Copy Entity ออกมา
world[3].inventory.push(sword)
heal(mut world[3], amount: 50)       ## ส่ง Projection เข้า mut Parameter ได้
```

ถ้าไม่ได้กำหนด `__index_mut` คอมไพเลอร์จะแปลง `a[i].field = v` เป็น `__index` → แก้ → `__set_index` ให้อัตโนมัติ (ถูกต้องเสมอแต่อาจ Copy)

### 3. วงจรชีวิตของข้อมูล (`__init`, `__drop`, `__str`):
- `__init(mut self, ...)`: Custom Constructor เมื่อสร้างอินสแตนซ์
- `__drop(mut self)`: Custom Destructor ทำงานอัตโนมัติก่อนตัวแปรหลุดออกจาก Scope `{}` (Deterministic Cleanup)
- `__str(self) string`: สำหรับแปลงข้อมูลเป็นข้อความใน f-strings `f"{object}"`

```
struct DatabaseConnection {
    handle *mut void

    fn __init(mut self, connection_str string) {
        self.handle = open_db(connection_str)
    }

    fn __drop(mut self) {
        close_db(self.handle) ## คลีนอัปอัตโนมัติ 100% เมื่อพ้น Scope
    }
}
```

### สารบัญ Special Functions:

| ตัวดำเนินการ / เหตุการณ์ | Special Function | ตัวอย่างเมื่อเขียนโค้ด |
| :--- | :--- | :--- |
| `a + b` | `a.__add(b)` | `v1 + v2` |
| `a - b` | `a.__sub(b)` | `v1 - v2` |
| `a * b` | `a.__mul(b)` | `v1 * 2` |
| `a / b` | `a.__div(b)` | `v1 / 2.0` |
| `a // b` | `a.__floordiv(b)` | `v1 // 2.0` |
| `a % b` | `a.__mod(b)` | `a % b` |
| `a == b` | `a.__eq(b)` | `if v1 == v2` |
| `a[i]` | `a.__index(i)` | `let x = arr[0]` |
| `a[i] = v` | `a.__set_index(i, v)` | `arr[0] = 10` |
| `a[i].field = v` / `f(mut a[i])` | `a.__index_mut(i)` | `world[3].hp -= 10` |
| f-string `"{a}"` | `a.__str()` | `print(f"{player}")` |
| Constructor | `a.__init(...)` | `let db = DatabaseConnection(...)` |
| Destructor | `a.__drop()` | ทำงานอัตโนมัติเมื่อถึง `}` |

---

## 21. `interface` Keyword (Type Aliases & Contracts)

ใช้คีย์เวิร์ด `interface` สำหรับการตั้งชื่อเล่นให้กับ Type ที่ซับซ้อน (Type Alias) หรือกำหนดรูปร่างของ Type:

```
## 1. Type Aliases พื้นฐาน
interface UserID = u64
interface StringMap = {string: string}
interface NumberList = [f64]

## 2. Closure Signature Alias
interface EventHandler = |event string, code i64| void

## การนำไปใช้งาน
let id UserID = 100928_u64
let config StringMap = {
    "host": "localhost",
    "port": "8080",
}
```

---

## 22. `when` Keyword & Type Specialization (`is`)

คีย์เวิร์ด `when` ใช้สำหรับการตรวจสอบเงื่อนไขและประเภทข้อมูล ทั้งในระดับ **Compile-Time (Zero-Cost Monomorphization)** และ **Runtime Type Inspection**:

### 1. Compile-Time Type Specialization (`when T is ...`):
ช่วยให้นักพัฒนาเขียนฟังก์ชัน Generic ที่มี **Fast-Path เฉพาะทางสำหรับ Type นั้นๆ** ได้ โดยที่คอมไพเลอร์จะตัดโค้ดส่วนที่ไม่เกี่ยวข้องทิ้งทันที (Dead Code Elimination) ทำให้ไม่มี Overhead ตอนรัน:

```
fn serialize[T](item T) string {
    ## ตรวจสอบ Type ตั้งแต่ตอน Compile
    when T is string {
        return item ## ไม่ต้องแปลง คืนค่าได้ทันที
    } else when T is i64 {
        return i64.to_string(item)
    } else {
        return item.to_string()
    }
}
```

### 2. Type Checking & Value Inspection (`when type(a) is ...`):
ใช้ตรวจสอบ Type ของตัวแปรขณะทำงาน หรือใช้เป็น Expression-based Conditional:

> **ขอบเขตของ Runtime Inspection**: `when type(x) is ...` ตรวจตอนรันได้ **เฉพาะค่าที่เป็น `Any` หรือ Trait Object** (§33) เท่านั้น เพราะค่าเหล่านี้พก Type ID ไว้ใน Fat Pointer อยู่แล้ว ถ้า `x` เป็น Type ที่รู้แน่นอนตอนคอมไพล์ คอมไพเลอร์จะตัดสินใจทันทีตอน Compile (เหมือนแบบที่ 1) — ค่าทั่วไปจึง **ไม่ต้องฝังข้อมูล Type** ไว้ในหน่วยความจำ และหลัก Zero Runtime ยังคงอยู่

```
fn inspect_value(val Any) {
    when type(val) is string {
        print(f"String of length {val.length()}") ## Smart Cast อัตโนมัติ
    } else when type(val) is i64 {
        print(f"Number multiplied: {val * 2}")
    } else {
        print("Unknown type")
    }
}

## 3. Multi-branch conditional evaluation
let status_message = when {
    score >= 90 => "Excellent",
    score >= 50 => "Passed",
    else => "Failed"
}
```

---

## 23. Compound Assignment Operators

รองรับตัวดำเนินการกำหนดค่าแบบย่อ ทั้งเชิงคณิตศาสตร์ บิต และการต่อข้อความ/คอลเลกชัน:

```
let mut count = 10
count += 5    ## 15
count -= 2    ## 13
count *= 2    ## 26
count /= 2    ## 13
count %= 5    ## 3

let mut mask = 0b0000_1111
mask &= 0b0000_0101   ## Bitwise AND
mask |= 0b1000_0000   ## Bitwise OR
mask ^= 0b0000_0001   ## Bitwise XOR
mask <<= 2            ## Left Shift
mask >>= 1            ## Right Shift

let mut text = "Hello"
text += " World"      ## ต่อข้อความ

let mut list = [1, 2]
list += [3, 4]        ## ต่อลิสต์
```

---

## 24. Ranges & Array Slicing (`..`, `..=`)

ใช้สำหรับระบุช่วงของข้อมูล และการทำ **Zero-Copy Slicing** (การตัด View ของข้อมูลโดยไม่ก๊อปปี้ Memory):

```
## 1. Range Expressions
let exclusive_range = 0..10  ## 0 ถึง 9 (ไม่รวม 10)
let inclusive_range = 1..=5  ## 1 ถึง 5 (รวม 5)

## 2. Loop ผ่าน Range
for i in 0..5 {
    print(f"Index: {i}")
}

## 3. Zero-Copy Slicing บน List
let numbers = [10, 20, 30, 40, 50]
let sub_slice = numbers[1..4]   ## ได้ [20, 30, 40] ปลอดภัย ไม่มีการจัดสรร Heap ใหม่
let from_start = numbers[..3]   ## ได้ [10, 20, 30]
let to_end = numbers[2..]       ## ได้ [30, 40, 50]
```

---

## 25. Type Casting (`as`)

การแปลงประเภทข้อมูลอย่างชัดเจน ปลอดภัย และไร้ Runtime Overhead:

```
let total i64 = 100
let average f64 = (total as f64) / 3.0

let byte_val u8 = 255_u8
let int_val u32 = byte_val as u32
```

---

## 26. Visibility (`pub`) & Comments (`##`, `###`)

ภาษาใช้หลักการ **Private by default** และใช้เครื่องหมายชาร์ป (`#`) สำหรับคอมเมนต์ เพื่อสงวนเครื่องหมาย `##` ไว้สำหรับการหารปัดเศษลง (Floor Division):

```
### คำนวณพื้นที่วงกลมจากรัศมี (Documentation Comment ใช้ ###)
pub fn circle_area(radius f64) f64 {
    ## นี่คือคอมเมนต์แบบ Single-line ใช้ ##
    return 3.1415926535 * radius * radius
}

pub struct BankAccount {
    pub account_number string  ## สาธารณะ อ่านได้จากภายนอก
    balance f64                ## ส่วนตัว (Private) เข้าถึงได้เฉพาะในโมดูลนี้
}
```

### การคำนวณการหาร (Division Operators):
- `/`: การหารปกติ (Float Division หรือตาม Type)
- `//`: การหารปัดเศษลง (Integer Floor Division แบบ Python เช่น `7 // 2` ได้ `3`)

---

## 27. Loop Constructs, Labels, and Loop Expressions (`loop`, `'label`)

ภาษาแบ่งการวนลูปออกเป็น 3 รูปแบบ โดยเฉพาะคำสั่ง `loop` ที่สามารถ **คืนค่า (Return Value)** ออกมาเป็น Expression ได้:

### 1. `loop` Expression (การวนลูปที่คืนค่าได้ด้วย `break value`):
เนื่องจาก `loop` การันตีว่าจะออกจากลูปได้ผ่าน `break` เท่านั้น คอมไพเลอร์จึงตรวจสอบ Type ได้อย่างสมบูรณ์:

```
let mut attempts = 0

## loop สามารถคืนค่าให้ตัวแปรได้โดยตรง
let result = loop {
    attempts += 1
    if try_connect() {
        break "Connected Successfully" ## คืนค่า string ออกจาก loop
    }
    if attempts >= 5 {
        break "Connection Timeout"
    }
}

print(f"Status: {result}")
```

### 2. Labelled Loops (การตั้งชื่อลูป):
รองรับทั้ง `loop`, `while`, และ `for` สามารถระบุชื่อลูปด้วย `'label` เพื่อ `break` หรือ `continue` ข้ามลูปหลายชั้น:

```
'search: loop {
    for row in 0..10 {
        for col in 0..10 {
            if grid[row][col] == target {
                break 'search (row, col) ## คืนค่าพิกัด Tuple ออกจาก loop นอกสุด
            }
        }
    }
    break 'search (-1, -1)
}
```

---

## 28. Primitive Types Catalog (สารบัญชนิดข้อมูลพื้นฐาน - ไม่มี Alias แฝง)

ภาษาเน้นความชัดเจนและแม่นยำสูง (Strict Explicit Types) **ไม่มีการแอบใส่ Alias อัตโนมัติ** (หากผู้ใช้ต้องการ สามารถสร้างผ่าน `interface` เองได้):

| ชนิดข้อมูล | ขนาดหน่วยความจำ | คำอธิบาย |
| :--- | :--- | :--- |
| `i8`, `i16`, `i32`, `i64`, `i128` | 1, 2, 4, 8, 16 Bytes | จำนวนเต็มมีเครื่องหมายที่ระบุบิตชัดเจน |
| `u8`, `u16`, `u32`, `u64`, `u128` | 1, 2, 4, 8, 16 Bytes | จำนวนเต็มไม่มีเครื่องหมายที่ระบุบิตชัดเจน |
| `f32`, `f64` | 4, 8 Bytes | เลขทศนิยมความละเอียดเดี่ยว/คู่ (IEEE 754) |
| `bool` | 1 Byte | ค่าความจริง (`true`, `false`) |
| `char` | 4 Bytes | อักขระ Unicode (UTF-32 scalar value) |
| `void` | 0 Bytes | Unit Type / ค่าว่าง (ไม่ต้องเขียนกำกับก็ได้) |

### การละเว้น `void` ในฟังก์ชัน:
ฟังก์ชันที่ไม่ต้องการคืนค่า สามารถเขียน `void` หรือละเว้นได้ตามสะดวก:
```
## เขียนแบบระบุ void ชัดเจน
fn main() void {
    print("Hello")
}

## เขียนแบบละเว้น void (ได้ผลลัพธ์เหมือนกัน 100%)
fn main() {
    print("Hello")
}
```

---

## 29. Comparison, Logical Operators, and Wildcard Discard (`_`)

### 1. Comparison Operators (ตัวดำเนินการเปรียบเทียบ):
`==` (เท่ากับ), `!=` (ไม่เท่ากับ), `<` (น้อยกว่า), `<=` (น้อยกว่าหรือเท่ากับ), `>` (มากกว่า), `>=` (มากกว่าหรือเท่ากับ)

### 2. Logical Operators with Short-Circuiting (ตรรกศาสตร์แบบลัดวงจร):
- `&&`: Logical AND (ถ้าข้างซ้ายเป็น `false` จะไม่ประเมินค่าข้างขวา)
- `||`: Logical OR (ถ้าข้างซ้ายเป็น `true` จะไม่ประเมินค่าข้างขวา)
- `!`: Logical NOT (กลับค่าความจริง)

```
if user.is_active && user.age >= 18 {
    print("Authorized")
}
```

### 3. Wildcard Discard (`_`):
ใช้สำหรับการละเว้นตัวแปรที่ไม่ต้องการใช้ เพื่อไม่ให้เกิดคำเตือนจากคอมไพเลอร์:
```
let _ = calculate_something() ## รันฟังก์ชันแต่ละเว้นค่าส่งกลับ
let (result, _) = get_pair()  ## สนใจเฉพาะค่าตัวแรก
```

---

## 30. Primitive Methods & Extensions (เมธอดบนข้อมูลพื้นฐาน)

ภาษาของเราสนับสนุนการเรียก Method บน Primitive Types ได้โดยตรงเหมือนอ็อบเจกต์ระดับสูง โดย **ไม่มี Overhead (Zero-Cost / No Boxing)** เพราะคอมไพเลอร์จะแปลงเป็นการเรียกฟังก์ชันสแตติกหรือแปลงเป็นคำสั่ง CPU Intrinsics ทันที:

### 1. Built-in Primitive Methods ทั่วไป:
```
## เมธอดบนตัวเลข
let abs_val = (-42_i64).abs()           ## ได้ 42
let clamped = 150_i64.clamp(0, 100)     ## ได้ 100
let as_str = 100_i64.to_string()        ## ได้ "100"

## เมธอดบนเลขทศนิยม
let rounded = 3.75_f64.round()          ## ได้ 4.0
let square_root = 16.0_f64.sqrt()       ## แปลงตรงเป็นคำสั่ง CPU SQRT
let is_valid = float_val.is_nan()

## เมธอดระดับบิตและฮาร์ดแวร์ (Hardware Intrinsics)
let ones = 0b1011_0001_u8.count_ones()  ## Population Count (คำสั่ง POPCNT ระดับ CPU)
let be_bytes = 0x1234_u16.to_be_bytes() ## แปลงเป็น Big-Endian Bytes Array

## เมธอดบน char และ bool
let is_digit = '9'.is_numeric()         ## true
let upper = 'a'.to_uppercase()          ## 'A'
let bool_str = true.to_string()         ## "true"
```

### 2. ไวยากรณ์ Lexer: การแยกแยะจุดทศนิยมกับ Method Call:
- `10.5` $\rightarrow$ จุดตามด้วยตัวเลข: คอมไพเลอร์มองเป็น Float
- `10.to_string()` $\rightarrow$ จุดตามด้วยตัวอักษร: คอมไพเลอร์มองเป็น Integer Literal `10` เรียกเมธอด `.to_string()` ทันทีโดยไม่ต้องใส่วงเล็บครอบ

### 3. การเพิ่ม Method ให้กับ Primitive Types (Extension via `impl`):
ผู้ใช้สามารถเขียนเพิ่ม Method ใหม่ให้กับ Primitive Type ได้อย่างอิสระ:

```
impl i64 {
    fn is_even(self) bool {
        return self % 2 == 0
    }

    fn square(self) i64 {
        return self * self
    }
}

fn example() {
    let num = 8_i64
    if num.is_even() {
        print(f"Square of {num} is {num.square()}")
    }
}
```

---

## 31. Asynchronous Programming (`async / await` via Stackless Coroutines)

รองรับงาน High-Concurrency Network Server โดย **ไม่ต้องมี Go Runtime แฝง**:

- ฟังก์ชัน `async fn` จะถูกคอมไพเลอร์แปลงเป็น **State Machine Struct** ขนาดเล็กตอน Compile
- สลับงานบน Event Loop ได้ระดับแสนการเชื่อมต่อพร้อมกัน (C100k)
- **ภาษาไม่มี Executor ฝังในตัว**: `async fn` แค่สร้าง State Machine — ตัวที่สั่งให้มันเดินคือ **Executor ที่ผู้ใช้เลือกเอง** โปรแกรมที่ไม่ใช้ `async` จึงไม่มีภาระใดๆ
  - `std/async` มี Executor มาตรฐาน (Event Loop บน epoll / kqueue / IOCP) ให้ใช้ได้ทันที
  - ไลบรารีภายนอกเขียน Executor ของตัวเองได้ (เช่น Executor ของ Game Engine ที่เดินงานทีละเฟรม) ผ่าน Trait `Future` และ `Waker` ใน `std/async`
  - `.so` / `.dll` ที่ส่งออกฟังก์ชัน `async` ต้องห่อด้วยฟังก์ชันธรรมดา (Blocking หรือ Callback) ก่อนส่งข้าม C-ABI

```
import "std/net" { TcpListener, TcpStream }
import "std/async" { Executor, spawn_task }

async fn handle_client(mut stream TcpStream) Result[void, NetError] {
    let mut buffer = [u8].with_capacity(1024)
    let bytes_read = stream.read_async(mut buffer).await?
    
    stream.write_async(b"HTTP/1.1 200 OK\r\n\r\nHello World!").await?
    return Result.Ok(())
}

async fn start_server() Result[void, NetError] {
    let listener = TcpListener.bind("127.0.0.1:8080").await?
    while let Result.Ok(stream) = listener.accept_async().await {
        spawn_task(handle_client(stream))
    }
    return Result.Ok(())
}

fn main() {
    ## จุดเดียวที่ Executor ถูกสร้าง — ทั้งโปรแกรมรันบน Event Loop นี้
    Executor.default().block_on(start_server())
}
```

---

## 32. SIMD Vector Types & Hardware Alignment (`#[align]`)

สำหรับงาน Game Engine, 3D Graphics, Audio DSP, และ AI Inference:

```
## 1. Memory Alignment บังคับเรียงชิดขอบ CPU Cache Line (64 bytes) เพื่อป้องกัน Cache False-Sharing
#[align(64)]
struct CacheAlignedData {
    counter u64
}

## 2. SIMD Vector Types (คำนวณ 4 หรือ 8 ตัวเลขพร้อมกันใน 1 คำสั่ง CPU)
fn process_audio_buffer(mut samples [f32x4]) {
    let volume_multiplier = f32x4.splat(1.5_f32)
    for i in 0..samples.length() {
        samples[i] = samples[i] * volume_multiplier ## คูณ 4 Float พร้อมกันระดับ Hardware
    }
}
```

---

## 33. Dynamic Dispatch & Trait Objects (`dyn Trait`)

ใช้สำหรับกรณีที่ต้องการเก็บอ็อบเจกต์ **ต่างชนิดกันไว้ในคอลเลกชันเดียวกัน** (เช่น ระบบ GUI Widgets หรือ Plugin System) ผ่าน Fat Pointer (ขนาด 16 ไบต์: Data Pointer + vtable):

```
trait Widget {
    fn render(self)
    fn on_click(mut self)
}

struct Button {
    label string
}
impl Widget for Button {
    fn render(self) { print(f"Render Button: {self.label}") }
    fn on_click(mut self) { print("Button Clicked!") }
}

struct Slider {
    value f64
}
impl Widget for Slider {
    fn render(self) { print(f"Render Slider: {self.value}") }
    fn on_click(mut self) { print("Slider dragged!") }
}

## ใช้ชื่อ Trait เป็น Type ได้ตรงๆ = Dynamic Dispatch (ไม่ต้องเขียน dyn)
fn render_screen(widgets [Widget]) {
    for widget in widgets {
        widget.render() ## Dynamic Dispatch ผ่าน vtable
    }
}

## เขียน dyn กำกับก็ได้ ถ้าอยากให้คนอ่านเห็นชัดว่าตรงนี้ใช้ vtable (ความหมายเหมือนกัน 100%)
fn render_all(widgets [dyn Widget]) { ... }

## ถ้าต้องการ Static Dispatch (Monomorphization, เร็วสุด) ให้ใช้ Generic
fn render_one[T: Widget](widget T) {
    widget.render() ## เรียกตรง ไม่ผ่าน vtable
}
```

### กฎการเลือก Dispatch:
| เขียนแบบ | Dispatch | ใช้เมื่อ |
| :--- | :--- | :--- |
| `[T: Widget]` (Generic) | Static (Compile-time) | รู้ Type ตอนคอมไพล์ ต้องการความเร็วสูงสุด |
| `Widget` หรือ `dyn Widget` (ชื่อ Trait เป็น Type) | Dynamic (vtable) | เก็บหลายชนิดในคอลเลกชันเดียว, Plugin |

> Plugin ที่โหลดจาก `.so` / `.dll` ต่างไฟล์กัน ต้องส่ง Trait Object ผ่านขอบเขต C-ABI ตามกฎใน doc 03 §4

---

## 34. Compile-Time Derivation (`#[derive(...)]`)

ลดการเขียน Boilerplate ด้วย Macro ในระดับคอมไพเลอร์ที่สร้างโค้ดที่มีประสิทธิภาพสูงให้โดยอัตโนมัติ:

```
#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
struct Player {
    id u64
    name string
    score f64
}

fn test_serialization() {
    let player = Player(id: 1, name: "Morose", score: 100.0)
    
    ## ฟังก์ชัน .to_json() ถูกสร้างให้อัตโนมัติในระดับคอมไพล์ ไร้ Runtime Reflection
    let json_str = player.to_json()
    print(f"JSON: {json_str}")
}
```

---

## 35. Embedded Resources (`include_bytes`, `include_string`)

ฟังก์ชัน Built-in สำหรับฝังไฟล์ Assets, Fonts, Config ลงในตัวไฟล์ Binary โดยตรง:

```
const APP_FONT = include_bytes("assets/fonts/Inter.ttf")
const DEFAULT_CONFIG_JSON = include_string("config/default.json")

fn load_font() {
    print(f"Font size embedded: {APP_FONT.length()} bytes")
}
```

---

## 36. Smart Memory Strategy: Small Buffer Optimization (SBO) & Reuse

แนวทางจัดการหน่วยความจำที่ก้าวล้ำกว่าการส่ง Arena Allocator ด้วยมือ:

1. **Small Buffer Optimization (SBO) by Default**:
   - `string` หรือ `[T]` ขนาดเล็ก ($\le 24$ Bytes) จะถูกเก็บอยู่บน **CPU Stack** ภายในตัวแปรโดยตรง ไม่มีการเรียก Heap Allocator จากระบบ
   - ข้อมูล 80% ของแอปพลิเคชันทั่วไปจึงทำงานด้วยความเร็วระดับ Stack เสมอ
2. **Compile-Time Buffer Reuse (Functional In-Place Update)**:
   - หากตัวแปรชิ้นหนึ่งกำลังจะถูกทิ้ง (Unique/Dropped) และมีการจองหน่วยความจำใหม่ที่มีขนาดเท่ากันในสโคปนั้น คอมไพเลอร์จะ **"ยึด Buffer เดิมมาใช้ใหม่ทันที"** โดยไม่ส่งคำสั่ง Free/Malloc ไปหาระบบปฏิบัติการ
