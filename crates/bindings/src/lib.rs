use early_return_core::{Config, Modifier};
use neon::prelude::*;

fn js_object_to_modifier<'a>(
    mut cx: FunctionContext<'a>,
    x: Handle<'a, JsObject>,
) -> NeonResult<(FunctionContext<'a>, Modifier<'a>)> {
    let kind: String = x.get::<JsString, _, _>(&mut cx, "kind")?.value(&mut cx);
    let value: String = x.get::<JsString, _, _>(&mut cx, "value")?.value(&mut cx);

    let modifier = match kind.as_str() {
        "function" => Modifier::function(value),
        "property" => Modifier::property(value),
        "method" => Modifier::method(value),
        _ => return cx.throw(x),
    };

    Ok((cx, modifier))
}

fn process(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);

    let config = cx.argument::<JsObject>(1)?;

    let value_check = config.get::<JsObject, _, _>(&mut cx, "valueCheck")?;
    let (mut cx, value_check) = js_object_to_modifier(cx, value_check)?;

    let unwrap = config.get::<JsObject, _, _>(&mut cx, "unwrap")?;
    let (mut cx, unwrap) = js_object_to_modifier(cx, unwrap)?;

    let mangle = config
        .get_opt::<JsBoolean, _, _>(&mut cx, "mangle")?
        .map_or(false, |x| x.value(&mut cx));

    let config = Config {
        value_check,
        unwrap,
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
