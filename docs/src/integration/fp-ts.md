# `fp-ts`

## Plugin configuration

Use the following plugin configuration:

```js
rustQuestionMark({
  valueCheck: x => x._tag === "Right" || x._tag === "Some",
  unwrap: x => x._tag === "Right" ? x.right : x.value,
})
```

To also get proper TypeScript support, create an `ambient.d.ts` in your source directory. The name of the file doesn't matter and can be anything; what matters is the `.d.ts` extension. Paste the following inside the file:

```ts
declare module "fp-ts/Either" {
  interface Right<A> {
    get $(): A;
  }

  interface Left<E> {
    get $(): never;
  }
}

declare module "fp-ts/Option" {
  interface Some<A> {
    get $(): A;
  }

  interface None {
    get $(): never;
  }
}

export {};
```