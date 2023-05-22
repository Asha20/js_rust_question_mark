# How it works

Let's look at the previous example again. When you write the following code:

```js
function add(a, b) {
  return a.$ + b.$;
}
```

And you initialize the plugin like this:

```js
rustQuestionMark({
  valueCheck: x => x.tag === "Ok" || x.tag === "Some",
  unwrap: x => x.value
})
```

The input will be transformed into the following:

```js
const EARLY_RETURN = Symbol();
const __unwrap = x => {
  if (x.tag === "Ok" || x.tag === "Some") return x.value;
  throw { [EARLY_RETURN]: x };
};

function add(a, b) {
  try {
    return __unwrap(a) + __unwrap(b);
  } catch (e) {
    if (EARLY_RETURN in e) return e[EARLY_RETURN];
    throw e;
  }
}
```

Every access of the `$` property is replaced with an `__unwrap` function call. This function will attempt to unwrap the provided value. However, if the value cannot be unwrapped, it will throw an object containing a `EARLY_RETURN` unique symbol.

All function bodies that contain the `.$` "operator" will become wrapped in a `try-catch` block which catches the `EARLY_RETURN` symbol and handles it appropriately. In the event that a regular error is thrown, it is simply re-thrown.

The plugin is flexible and it lets you choose the `Result/Option` implementation you'd like to use yourself. Since different implementations often have differing APIs, two things need to be specified when initializing the plugin:

- `valueCheck` -- How do I check if a `Result/Option` is an `Ok/Some`?
- `unwrap` -- How do I unwrap the value once I've confirmed it's an `Ok/Some`?

Optionally, you can also pass `mangle: true` to the configuration (disabled by default), which will suffix the `EARLY_RETURN` and `__unwrap` declarations with random characters, making it unlikely that a name collision occurs with existing variables of the same name.

## Why `.$` instead of a custom operator?

- It's legal ECMAScript, so it won't break IDEs
- It's short, making it easy to write and read
- It's an uncommon property name, so it's not very likely to clash with pre-existing code
- TypeScript support can be provided by augmenting existing interfaces with a getter

### What if my code uses `.$` on things that aren't Results/Options?

For speed & ease of implementation, this plugin operates on a simple **text search and replace** principle. As such, it **does not** actually check whether the object you're accessing `.$` on is a `Result/Option`. If you have instances in your code that you'd like to treat as simple property accesses and not trigger the plugin on them, change them from `.$` into `["$"]`:

```diff
- const $ = window.$;
+ const $ = window["$"];
```