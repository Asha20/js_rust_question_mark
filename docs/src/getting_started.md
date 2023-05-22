# Getting Started

First, install the plugin:

```sh
$ npm install rollup-plugin-rust-question-mark
```

Then, add it to your Rollup config:

```js
// rollup.config.js
import rustQuestionMark from "rollup-plugin-question-mark";

const config = ...;

export default {
  input: "src/main.js",
  plugins: [
    rustQuestionMark(config),
  ],
  output: {
    file: "dist/bundle.js",
    format: "esm",
  },
};
```

Depending on which `Result/Option` library you're using, you're going to need to provide a different config object to the plugin. Choose the option that suits your project best:

- [I am using `fp-ts`](./integration/fp-ts.md)
- [I am using `neverthrow`](./integration/neverthrow.md)
- [I am using some other third-party library](./integration/other_third_party.md)
- [I am using my own custom `Result/Option` implementation](./integration/custom.md)