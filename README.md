`typedpack` is an interface description language and code generators
to convert that language into Rust and TypeScript types and
serialization/deserialization code.

## Example

Here is a simple `typedpack` file:
```typedpack
struct Email {
    string address = 0;
    bool password_recovery_enabled = 1;
}

# not a gender! this specifies if the user has sex
enum Sex {
    No = 0;
    Yes = 1;
}

struct User {
    bytes16 id = 0;
    string username = 2;
    optional Sex has_sex = 3;
    nullable bytes image = 4;
    Email[] emails = 5;
}
```

From this, approximately the following Rust code would be generated:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Email {
    address: String,
    password_recovery_enabled: bool,
}

impl serde::Serialize for Email { /* ... */ }
impl<'de> serde::Deserialize<'de> for Email { /* ... */ }

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserializer_repr,
)]
#[repr(u8)]
enum Sex {
    No = 0,
    Yes = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct User {
    id: [u8; 16],
    username: String,
    has_sex: Option<Sex>,
    image: Option<serde_bytes::ByteBuf>,
    emails: Box<[Email]>,
}

impl serde::Serialize for User { /* ... */ }
impl<'de> serde::Deserialize<'de> for User { /* ... */ }
```

As well as the following TypeScript code:
```typescript
export namespace Types {
    export interface Email {
        address: string;
        password_recovery_enabled: boolean;
    }

    export enum Sex {
        No = 0,
        Yes = 1,
    }

    export interface User {
        id: ArrayBuffer;
        username: string;
        has_sex?: Sex;
        image: ArrayBuffer | null;
        emails: Email[];
    }
}

export namespace Encode {
    export const Email = (value: Types.Email): DataView => { /* ... */ };
    export const Sex = (value: Types.Sex): DataView => { /* ... */ };
    export const User = (value: Types.User): DataView => { /* ... */ };
}

export namespace Decode {
    export const Email = (data: DataView): Types.Email => { /* ... */ };
    export const Sex = (data: DataView): Types.Sex => { /* ... */ };
    export const User = (data: DataView): Types.User => { /* ... */ };
}
```

The types are intended to be serializable with MessagePack. To save bytes
on the wire, field names are replaced with small integers (similar to Protobuf);
this is what the field ID `2` in `string username = 2;` is used for.

## Data types

<table>
<tr>
<th>

`typedpack` type

</th>
<th>

Rust type

</th>
<th>

TypeScript type

</th>
</tr>
<tr>
<td>

`bool`

</td>
<td>

`bool`

</td>
<td>

`boolean`

</td>
</tr>
<tr>
<td>

`uint8`

</td>
<td>

`u8`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`int8`

</td>
<td>

`i8`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`uint16`

</td>
<td>

`u16`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`int16`

</td>
<td>

`i16`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`uint32`

</td>
<td>

`u32`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`int32`

</td>
<td>

`i32`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`uint64`

</td>
<td>

`u64`

</td>
<td>

`bigint`

</td>
</tr>
<tr>
<td>

`int64`

</td>
<td>

`i64`

</td>
<td>

`bigint`

</td>
</tr>
<tr>
<td>

`float32`

</td>
<td>

`f32`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`float64`

</td>
<td>

`f64`

</td>
<td>

`number`

</td>
</tr>
<tr>
<td>

`string`

</td>
<td>

`String`

</td>
<td>

`string`

</td>
</tr>
<tr>
<td>

`bytes`

</td>
<td>

`serde_bytes::ByteBuf`

</td>
<td>

`ArrayBuffer`

</td>
</tr>
<tr>
<td>

`bytesN`

</td>
<td>

`serde_bytes::ByteArray<N>`

</td>
<td>

`ArrayBuffer`

</td>
</tr>
<tr>
<td>

`T[]`

</td>
<td>

`Box<[T]>`

</td>
<td>

`Array<T>`

</td>
</tr>
<tr>
<td>

```typedpack
struct Foo {
    T a = 0;
    optional T b = 1;
    nullable T c = 2;
    optional nullable T d = 3;
}
```

</td>
<td>

```rust
struct Foo {
    a: T,
    b: Option<T>,
    c: Option<T>,
    d: Option<Option<T>>,
}
```

</td>
<td>

```typescript
interface Foo {
    a: T;
    b?: T;
    c: T | null;
    d?: T | null;
}
```

</td>
</tr>
<tr>
<td>

Untagged `enum`:

```typedpack
enum Foo {
    A = 0;
    B = 1;
}
```

</td>
<td>

```rust
#[repr(u8)]
enum Foo {
    A = 0,
    B = 1,
}
```

</td>
<td>

```typescript
enum Foo {
    A = 0,
    B = 1,
}
```

</td>
</tr>
<tr>
<td>

Tagged `enum` (each variant contains a `struct`; other types are not allowed):

```typedpack
struct Foo {}
struct Bar {}

enum Baz {
    Foo A = 0;
    Bar B = 1;
}
```

</td>
<td>

```rust
struct Foo {}
struct Bar {}

enum Baz {
    A(Foo),
    B(Bar),
}
```

</td>
<td>

```typescript
// empty structs are `object`
// rather than `interface {}`
type Foo = object;
type Bar = object;

enum Baz {
    A = 0,
    B = 1,
}

// the type that encoding functions
// take and decoding functions
// return is not actually named,
// but it is a type union like so:
type ActualBaz =
    [Baz.A, Foo] | [Baz.B, Bar];
```

</td>
</tr>
</table>

Notes:
- `struct` field IDs and `enum` variant IDs are limited to
7-bit integers (0–127). They may not contain leading zeros.
- `bytesN` is a fixed-length byte array, where `N` is an unsigned 32-bit
integer with no leading zeros.
- Arrays can be nested, i.e. multidimensional, i.e. `T[][][]` works.
- All `struct`s and `enum`s referenced must be contained in the same file;
there is no inclusion mechanism.

## Usage

See [`example`](./example) for an example project with a Rust server
and a TypeScript client.
