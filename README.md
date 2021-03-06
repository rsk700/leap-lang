Parser for Leap language.

Links:

* [Command line tool](https://github.com/rsk700/leap-cli) - formatting and verification.

# Leap language.

Leap is a light, simple language for describing data structures.

Supported data types:

- `str` - utf-8 string
- `int` - 64-bit integer number
- `float` - 64-bit floating point number
- `bool` - boolean type
- `list` - array of values
- `struct` - user defined type with fields
- `enum` - user defined type with multiple variants

## Naming

All user defined names use kebab case (all letters are lower case, separated with `-`), eg.: `user-auth`, `article-title`, `some-long-long-long-name`:

* name should start with a letter and can contain numbers
* words in the name are separated with a single `-`

## List

List defines array of values and accept single type argument for the type of elements:

- `list[int]` - list of integers
- `list[user]` - list of `user` structs
- `list[list[string]]` - list of lists of strings

## Struct

Struct is a user defined type, can have zero or more fields, and can have type arguments for generict values.

Example:

```
.struct user
    name: str
    age: int
    address: str
    tags: list[str]
    active: bool
```

here `user.name` is string, and `user.tags` is list of strings

Empty struct with no fields:

```
.struct none
```

Struct with type argument:

```
.struct some[t]
    value: t
```

here `t` is a type argument, and if it will be applied as `str`, `value` will become `str`

## Enum

Enum is a user defined type, which describes which variants it can be, only structs can be variants of enum, enum can have type arguments.

Example:

```
.enum response
    user
    none
```

here `response` can be either `user` or `none` struct

Variants can be named:

```
.enum account
    admin: user
    customer: user
```

here variant names allow to avoid name conflict, as both variants `admin` and `customer` use same type `user`.

Enum with type argument:

```
.enum option[t]
    some[t]
    none
```

here `t` is a type argument, and if it will applied as `int`, `some[t]` variant will become `some[int]`

## Type arguments

Types can have type arguments for generic values. If there is multiple type arguments, they separated with spaces:

```
.struct some-struct[a b c]
    value-a: a
    value-b: b
    value-c: c
    value-d: other-struct[a b list[c]]
    value-e: some[int]
```

here `a`, `b`, `c` is type arguments, which should be applied in order to use type, for example `some-struct[int int str]`, in this case `value-a` will have `int` type, and `value-d` will have `other-struct[int int list[str]]` type. `value-e` have type `some[int]`, which is `some[t]` with `t` applied as `int`.

## Comments

Comments start with `/--` and can be placed on separate line, or at the end of the line:

```
/-- some comment about page struct
.struct page[t]
    items: list[t] /-- other comment about items of page
    /-- comment about page additional info
    total-count: int
```

# Example

Lets model types which can be used for REST API of blog engine:

```
/-- general types
.struct none

.struct some[t]
    value: t

.enum option[t]
    none
    some[t]

.enum result[t e]
    ok: some[t]
    err: some[e]

/-- api types
.struct page[t]
    value: t
    total-count: option[int]

.struct user
    id: int
    name: str
    email: str

.struct article
    id: int
    author: user
    title: str
    text: str
    tags: list[str]
```

here for our api:

* for every request api returns `result[t str]`, `t` for correct response or `str` for error (string with error message)
* `GET /users/7` will return `result[user str]`, which allows to get info about user by id on success or error message
* `GET /articles` will return `result[page[article] str]`, which allows to get paged list of articles on success or error message
* `page.total-count` is optional, if `total-count` is unknown it will be equal to `none`, otherwise `some[int]`

# Example usage

Cargo.toml

```toml
[dependencies]
leap-lang = "0.2"
```

main.rs

```rust
use leap_lang::parser::parser::Parser;

fn main() {
    let types = Parser::parse("
        .enum enum1
        .struct struct1
        .struct struct2
            v1: int
    ").unwrap();
    for t in types {
        println!("name: {}", t.name());
    }
    // output:
    //
    // name: enum1
    // name: struct1
    // name: struct2
}
```