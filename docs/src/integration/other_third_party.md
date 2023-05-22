# Other third-party libraries

If you're using a third-party `Result/Option` library that isn't already covered here, you can open a new issue about it on Github so that its integration guide can be added eventually. If you'd like to integrate it by yourself, there are two steps:

1. Configuring the plugin
2. Augmenting the library's TypeScript types to add the `$` getter

> **Advice:** Take a look at the existing third-party integrations to get a better idea of how to set things up yourself.

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

## Augmenting the library's types

> **Important note:** If the `Ok/Err` and `Some/None` types are defined in the original library by using `type` instead of `interface`, augmenting them is unfortunately **not possible**. This issue can only be resolved within the original library.

Create an `ambient.d.ts` in your source directory. The name of the file doesn't matter and can be anything; what matters is the `.d.ts` extension. Inside this file, add a `declare module "library-name-here"` clause and augment the relevant interfaces in order to add the `$` property getter. Here are some pointers:

- For your `Ok/Some` interfaces, add the `get $(): T` getter
- For your `Err/None` interfaces, add the `get $(): never` getter
- Add an `export {}` to your file in order to make it a module (otherwise you might run into an issue where your declarations overwrite the library instead of augmenting it)
- Make sure your generic type variables have the same name as those in the library when augmenting. If the library uses `interface Some<A>`, augmenting with `interface Some<T>` will cause an error.