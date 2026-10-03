use rustyline::{DefaultEditor, Result as RustyResult, error::ReadlineError};

use calculator::Compile;

cfg_select! {
    feature = "jit" => {
        use calculator::Jit  as Engine;
    },
    feature = "interpreter" => {
        use calculator::Interpreter as Engine;
    },
    feature = "vm" => {
         use calculator::vm::bytecode::Interpreter  as Engine;
        use::calculator::VM;
    },
    _ => {
        use calculator::Interpreter as Engine;
    }
}

fn main() -> RustyResult<()> {
    let mut rl = DefaultEditor::new()?;

    println!("TaiwoLang REPL. Expressions are line evaluated");

    loop {
        let readline = rl.readline(">>> ");

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                cfg_select! {
                    any(feature = "jit", feature = "interpreter") => {
                        match Engine::from_source(line) {
                            Ok(result) => println!("{}", result),
                            Err(e) => eprintln!("{}", e)
                        }
                    },
                    feature = "vm" => {
                        let byte_code = Engine::from_source(line);
                        println!("vm byte code: {:?}", byte_code);

                        let mut vm = VM::new(byte_code);
                        vm.run();

                        println!("Result: {}", vm.pop_last());
                    }
                    _ => {
                        match Engine::from_source(line) {
                            Ok(result) => println!("{}", result),
                            Err(e) => eprintln!("{}", e)
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl + C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl + D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
