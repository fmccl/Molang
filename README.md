# Molang
From the [Minecraft docs](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/molangreference/examples/molangconcepts/molangintroduction?view=minecraft-bedrock-stable), Molang is a simple expression-based language designed for fast, data-driven calculation of values at run-time, with a direct connection to in-game values and systems.

That said, it's just a normal expression language, which could be suitable for spreadsheets, animation software and games other than Minecraft.
# Basic Usage
```rs
assert_eq!(Value::Number(200.0), run(&compile("!1 ? 100 : 200").unwrap(), &HashMap::new(), &HashMap::new()).unwrap());
```
## Adding functions from outside
```rs

let mut functions: HashMap<&str, &dyn Fn(Vec<Value>) -> Result<Value, MolangError>> =
    HashMap::new();
    
functions.insert("max", &|args| {
    let mut biggest: Option<f32> = None;

        for arg in args {

            if let Value::Number(num) = arg {

            match biggest {
                None => biggest = Some(num),
                Some(big) if num > big => biggest = Some(num),
                _ => {}
            }

        } else {
            return Err(MolangError::FunctionError("Expected a number".into()));
        }
    }
        
    Ok(Value::Number(biggest.ok_or(MolangError::FunctionError(
        "No arguments passed to max".into(),
    ))?))

});

assert_eq!(
    Value::Number(500.0),
    run(&compile("max(1, 5, 2) * 100").unwrap(), &functions, &HashMap::new()).unwrap()
);

```
## Adding constants
```rs
let mut constants = HashMap::new();

constants.insert("pi", Value::Number(3.14));

assert_eq!(
    Value::Number(100.0 * 3.14),
    run(&compile("pi * 100").unwrap(), &HashMap::new(), &constants).unwrap()
);
```
