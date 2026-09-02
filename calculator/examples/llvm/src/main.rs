use inkwell::{context::Context, execution_engine::JitFunction};

const ADDITION_FUNC_NAME: &str = "add";

fn main() {
    let context = Context::create();

    let module = context.create_module("addition");

    let i32_type = context.i32_type();

    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let fn_val = module.add_function(ADDITION_FUNC_NAME, fn_type, None);
    let entry_basic_block = context.append_basic_block(fn_val, "entry");

    let builder = context.create_builder();
    builder.position_at_end(entry_basic_block);

    let first_param = fn_val.get_nth_param(0).unwrap().into_int_value();
    let second_param = fn_val.get_nth_param(1).unwrap().into_int_value();

    let return_build = builder
        .build_int_add(first_param, second_param, "result")
        .unwrap();
    let return_instruction = builder.build_return(Some(&return_build)).unwrap();

    dbg!("module: {:?}", module.clone());
    dbg!("builder: {:?}", builder);

    assert_eq!(return_instruction.get_num_operands(), 1);

    let execution_engine = module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .unwrap();

    unsafe {
        type Addition = unsafe extern "C" fn(i32, i32) -> i32;

        let add: JitFunction<Addition> = execution_engine.get_function(ADDITION_FUNC_NAME).unwrap();

        let x = 87;
        let y = 342;
        println!("The value is {}", add.call(x, y));

        assert_eq!(add.call(x, y), x + y);
    }
}
