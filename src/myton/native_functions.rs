use super::environment::{Env, EnvVariable};
use super::functions::NativeFunction;
use super::traceback::Traceback;
use super::types::DynValue;

pub fn define_globals(env: &Env) {
    let mut env = env.borrow_mut();
    let native_functions: Vec<(&str, NativeFunction)> = vec![(
        "clock",
        NativeFunction {
            func: native_clock,
            nb_args: 0,
        },
    )];

    for (name, func) in native_functions {
        env.set(
            name.to_string(),
            DynValue::from_native_function(func, name.to_string()),
        );
    }

    env.set_env_var(EnvVariable::NewLines, DynValue::from(0));
}

pub fn native_clock(_: &Env, _: Vec<DynValue>) -> Result<DynValue, Traceback> {
    let now = std::time::SystemTime::now();
    if let Ok(since_the_epoch) = now.duration_since(std::time::UNIX_EPOCH) {
        let in_seconds =
            since_the_epoch.as_secs() as f64 + since_the_epoch.subsec_nanos() as f64 * 1e-9;
        Ok(DynValue::from(in_seconds))
    } else {
        Err(Traceback::from_message("clock: time went backwards??"))
    }
}
