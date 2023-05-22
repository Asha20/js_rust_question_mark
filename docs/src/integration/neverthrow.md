# `neverthrow`

## Plugin configuration

Use the following plugin configuration:

```js
rustQuestionMark({
  valueCheck: x => x.isOk(),
  unwrap: x => x.value,
})
```

To also get proper TypeScript support, create an `ambient.d.ts` in your source directory. The name of the file doesn't matter and can be anything; what matters is the `.d.ts` extension. Paste the following inside the file:

```ts
declare module "neverthrow" {
  interface Ok<T, E> {
    get $(): T;
  }

  interface Err<T, E> {
    get $(): never;
  }
}

export {};
```