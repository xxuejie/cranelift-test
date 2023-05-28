use cranelift_codegen::{
    ir::{types::I64, AbiParam, InstBuilder, Signature},
    isa::{lookup_by_name, CallConv},
    settings,
    verifier::verify_function,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

fn main() {
    let flags = settings::Flags::new(settings::builder());
    let x64_isa = lookup_by_name("x86_64-unknown-linux-gnu").unwrap();
    let x64_isa = x64_isa.finish(flags.clone()).unwrap();

    let object_builder = ObjectBuilder::new(
        x64_isa,
        "test_object",
        cranelift_module::default_libcall_names(),
    )
    .unwrap();
    let mut object_module = ObjectModule::new(object_builder);

    let mut sig = Signature::new(CallConv::Fast);

    let arg_num = 8;

    for _ in 0..arg_num {
        sig.params.push(AbiParam::new(I64));
        sig.returns.push(AbiParam::new(I64));
    }

    let mut module_context = object_module.make_context();
    module_context.func.signature = sig;

    let mut fn_builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut module_context.func, &mut fn_builder_ctx);

    let block0 = builder.create_block();
    builder.append_block_params_for_function_params(block0);
    builder.switch_to_block(block0);
    builder.seal_block(block0);

    let mut params = builder.block_params(block0).to_vec();
    params.reverse();
    builder.ins().return_(&params);

    let res = verify_function(&module_context.func, &flags);
    println!("{}", module_context.func.display());
    if let Err(errors) = res {
        panic!("{}", errors);
    }

    let id = object_module
        .declare_function("foo", Linkage::Export, &module_context.func.signature)
        .unwrap();

    object_module
        .define_function(id, &mut module_context)
        .unwrap();

    object_module.clear_context(&mut module_context);

    let product = object_module.finish();
    let data = product.emit().unwrap();

    std::fs::write("test.o", data).unwrap();
}
