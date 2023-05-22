# Custom implementation

Here's the implementation we're going to be working with. This example should serve as a guide to help you integrate your own implementation:

```ts
interface Ok<T> {
  isOk: true;
  value: T;
}

interface Err<E> {
  isOk: false,
  value: E;
}

type Result<T, E> = Ok<T> | Err<E>;

interface Some<T> {
  isSome: true;
  value: T;
}

interface None {
  isSome: false,
}

type Option<T> = Some<T> | None;
```

There are two steps to integrate your custom `Result/Option` types with the plugin:

1. Configuring the plugin
2. Adding the `$` getter for TypeScript support

## Configuring the plugin

The plugin configuration looks like this:

```ts
interface Config {
  valueCheck: (x: any) => any;
  unwrap: (x: any) => any;
  mangle?: boolean;
}
```

- `valueCheck` -- This function should return a truthy value if `x` is an `Ok/Some` value. Otherwise, it should return a falsy value.
- `unwrap` -- This function should return the unwrapped value from the given `Ok/Some`.

With our implementation, we could write the configuration like so:

```js
rustQuestionMark({
  valueCheck: x => x.isOk || x.isSome,
  unwrap: x => x.value,
})
```

## Adding the `$` getter

Simply change the type declarations like so:

```diff
interface Ok<T> {
  isOk: true;
  value: T;
+ get $(): T;
}

interface Err<E> {
  isOk: false,
  value: E;
+ get $(): never;
}

type Result<T, E> = Ok<T> | Err<E>;

interface Some<T> {
  isSome: true;
  value: T;
+ get $(): T;
}

interface None {
  isSome: false,
+ get $(): never;
}

type Option<T> = Some<T> | None;
```