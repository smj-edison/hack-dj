use std::error::Error as StdError;
use std::fs::File;

use rustyline::DefaultEditor;

use piccolo::{
    compiler::{ParseError, ParseErrorKind},
    io, meta_ops, Callback, CallbackReturn, Closure, Executor, Function, Lua, PrototypeError,
    StashedExecutor, StaticError,
};

fn run_code(lua: &mut Lua, executor: &StashedExecutor, code: &str) -> Result<(), StaticError> {
    lua.try_enter(|ctx| {
        let closure = match Closure::load(ctx, None, ("return ".to_string() + code).as_bytes()) {
            Ok(closure) => closure,
            Err(_) => Closure::load(ctx, None, code.as_bytes())?,
        };
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
    })?;

    lua.execute::<()>(executor)
}

fn run_repl(lua: &mut Lua) -> Result<(), Box<dyn StdError>> {
    let mut editor = DefaultEditor::new()?;
    let executor = lua.enter(|ctx| ctx.stash(Executor::new(ctx)));

    loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
            let read = editor.readline(prompt)?;
            let read_empty = read.trim().is_empty();
            if !read_empty {
                if !line.is_empty() {
                    // Separate input lines in the input to the parser
                    line.push('\n');
                }
                line.push_str(&read);
            }

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

fn main() -> Result<(), Box<dyn StdError>> {
    let mut lua = Lua::full();

    run_repl(&mut lua)?;

    Ok(())
}
