use std::borrow::Cow;

use early_return_core::Config;
use neon::prelude::*;

fn process(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);

    let config = cx.argument::<JsObject>(1)?;

    let value_check = config
        .get::<JsString, _, _>(&mut cx, "valueCheck")?
        .value(&mut cx);

    let unwrap = config
        .get::<JsString, _, _>(&mut cx, "unwrap")?
        .value(&mut cx);

    let mangle = config
        .get_opt::<JsBoolean, _, _>(&mut cx, "mangle")?
        .map_or(false, |x| x.value(&mut cx));

    let config = Config {
        value_check: Cow::Owned(value_check),
        unwrap: Cow::Owned(unwrap),
        mangle,
    };

    let result = early_return_core::process(&input, config);
    Ok(cx.string(result))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("process", process)?;
    Ok(())
}
