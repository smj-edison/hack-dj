use std::io::stdin;

use piccolo::{meta_ops, Callback, CallbackReturn, Closure, Executor, Function, Lua};

fn main() {
    let mut lua = Lua::full();

    let executor = lua.enter(|ctx| ctx.stash(Executor::new(ctx)));

    loop {
        let mut line = String::new();

        loop {
            let read = stdin().read_line(&mut line).unwrap();

            let res = lua.try_enter(|ctx| {
                let closure =
                    Closure::load(ctx, None, ("return ".to_string() + &line).as_bytes()).unwrap();

                let function = Function::compose(
                    &ctx,
                    [
                        closure.into(),
                        Callback::from_fn(&ctx, |ctx, _, stack| {
                            Ok(if stack.is_empty() {
                                CallbackReturn::Return
                            } else {
                                CallbackReturn::Call {
                                    function: meta_ops::call(ctx, ctx.get_global("print"))?,
                                    then: None,
                                }
                            })
                        })
                        .into(),
                    ],
                );

                ctx.fetch(executor).restart(ctx, function, ());

                Ok(())
            });

            match run_code(lua, &executor, &line) {
                Err(StaticError::Runtime(err))
                    if !read_empty
                        && matches!(
                            err.downcast::<PrototypeError>(),
                            Some(PrototypeError::Parser(ParseError {
                                kind: ParseErrorKind::EndOfStream { .. },
                                ..
                            }))
                        ) =>
                {
                    prompt = ">> ";
                }
                Err(e) => {
                    editor.add_history_entry(line)?;
                    eprintln!("{}", e);
                    break;
                }
                Ok(()) => {
                    editor.add_history_entry(line)?;
                    break;
                }
            }
        }
    }
}
