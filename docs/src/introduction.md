# Introduction

`rollup-plugin-rust-question-mark` is a Rollup plugin that lets you use Rust's `?` operator in JavaScript and TypeScript. There are many libraries on NPM that provide the `Result/Either` and `Option/Maybe` monads. However, JavaScript's lack of the `?` operator unfortunately makes these unwieldy to use.

Let's look at a simple example: a function that adds two numbers. Here's how we might write it in Rust:

```rust
fn add(a: Result<u32, String>, b: Result<u32, String>) -> Result<u32, String> {
  Ok(a? + b?)
}
```

And here's the TypeScript version:

```ts
function add(a: Result<number, string>, b: Result<number, string>): Result<number, string> {
  if (a.isErr()) return a;
  if (b.isErr()) return b;
  return Ok(a.unwrap() + b.unwrap());
}
```

In JS-land, this issue is often dealt with by embracing functional composition extensively. However, I think that Rust's imperative approach is often easier to read and understand. Using this plugin allows you to write the code like so:

```ts
function add(a: Result<number, string>, b: Result<number, string>): Result<number, string> {
  return Ok(a.$ + b.$);
}
```