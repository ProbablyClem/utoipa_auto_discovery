<div style="text-align: center">
<h1>Utoipauto</h1>
  <p>
    <strong>Rust Macros to automate the addition of Paths/Schemas to Utoipa crate, simulating Reflection during the compilation phase</strong>
  </p>
</div>

# Crate presentation

Utoipa is a great crate for generating documentation (openapi/swagger) via source code.

But since Rust is a static programming language, we don't have the possibility of automatically discovering paths and
dto in runtime and adding them to the documentation,

For APIs with just a few endpoints, it's not that much trouble to add controller functions one by one, and DTOs one by
one.

But, if you have hundreds or even thousands of endpoints, the code becomes very verbose and difficult to maintain.

Ex :

```rust

#[derive(OpenApi)]
#[openapi(
    paths(
        // <================================ All functions  1 to N
        test_controller::service::func_get_1,
        test_controller::service::func_get_2,
        test_controller::service::func_get_3,
        test_controller::service::func_get_4,
       ....
       ....
       ....
        test_controller::service::func_get_N,

    ),
    components(
        // <====================== All DTO one by one
        schemas(TestDTO_1,  TestDTO_2, ........ , TestDTO_N)
    ),
    tags(
        (name = "todo", description = "Todo management endpoints.")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

```

The goal of this crate is to propose a macro that automates the detection of methods carrying Utoipa
macros (`#[utoipa::path(...]`), and adds them automatically. (it also detects sub-modules.)

It also detects struct that derive or implement `ToSchema` for the `components(schemas)` section, and the `ToResponse`
for the `components(responses)` section.

# Features

- [x] Automatic recursive path detection
- [x] Automatic import from module
- [x] Automatic import from src folder
- [x] Automatic model detection
- [x] Automatic response detection
- [x] Works with workspaces
- [x] Exclude a method from automatic scanning
- [x] Custom path detection

# How to use it

Simply add the crate `utoipauto` to the project

```
cargo add utoipauto
```

Import macro

```rust
use utoipauto::utoipauto;
```

Then add the `#[utoipauto]` macro just before the #[derive(OpenApi)] and `#[openapi]` macros.

## Important !!

Put `#[utoipauto]` before `#[derive(OpenApi)] `and `#[openapi]` macros.

```rust
#[utoipauto(paths = "MODULE_SRC_FILE_PATH, MODULE_SRC_FILE_PATH, ...")]
```

The paths receives a String which must respect this structure :

`"MODULE_SRC_FILE_PATH, MODULE_SRC_FILE_PATH, ..."`

You can add several paths by separating them with a coma `","`.

## `utoipauto` Macro Attributes

All attributes are optional!
The `utoipauto` macro supports several attributes to customize its behavior:

- **`paths`**: Specifies the paths to scan for functions and structs. If no path is specified, the macro will
  automatically scan the `src` folder.

```rust
#[utoipauto(paths = "./src/rest")]
```

- **convert_to_full_path**: When set to false, it allows importing from another crate without converting to a full path.
  Defaults to `true`.

```rust
#[utoipauto(paths = "./utoipauto/src from utoipauto", convert_to_full_path = false)]
```

- **function_attribute_name**: Specifies a custom attribute to look for instead of the default #[utoipa::path(...)].

```rust
#[utoipauto(function_attribute_name = "handler")] 
```

- **schema_attribute_name**: Specifies a custom derive attribute for model detection instead of the default ToSchema.

```rust
#[utoipauto(schema_attribute_name = "Schema")]
```

- **response_attribute_name**: Specifies a custom derive attribute for response detection instead of the default
  ToResponse.

```rust
#[utoipauto(response_attribute_name = "Response")]
```

These attributes allow you to tailor the macro's behavior to fit your project's structure and naming conventions

## Usage with workspaces

If you are using a workspace, it just works out of the box.

If you want to import from another crate you have to specify the crate, the path and set `convert_to_full_path` to
`false`.

```rust
#[utoipauto(paths = "./utoipauto/src from utoipauto", convert_to_full_path = false)]
```

### Import from src folder

If no path is specified, the macro will automatically scan the `src` folder and add all the methods carrying
the `#[utoipa::path(...)]` macro, and all structs deriving `ToSchema` and `ToResponse`.
Here's an example of how to add all the methods contained in the src code.

```rust
...

use utoipauto::utoipauto;

...
#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "todo", description = "Todo management endpoints.")
    ),
    modifiers(&SecurityAddon)
)]

pub struct ApiDoc;

...

```

### Import from module

Here's an example of how to add all the methods and structs contained in the rest module.

```rust

use utoipauto::utoipauto;

#[utoipauto(
    paths = "./src/rest"
)]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "todo", description = "Todo management endpoints.")
    ),
    modifiers(&SecurityAddon)
)]

pub struct ApiDoc;

```

Here's an example of how to add all methods and structs contained in the rest module if it is in a sub-folder:

```rust

use utoipauto::utoipauto;

#[utoipauto(
    paths = "(./src/lib/rest from crate::rest)"
)]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "todo", description = "Todo management endpoints.")
    ),
    modifiers(&SecurityAddon)
)]

pub struct ApiDoc;

```

### Import from filename

Here's an example of how to add all the methods contained in the test_controller and test2_controller modules.
you can also combine automatic and manual addition, as here we've added a method manually to the documentation "
other_controller::get_users", and a schema "TestDTO".

```rust

use utoipauto::utoipauto;

#[utoipauto(
    paths = "./src/rest/test_controller.rs,./src/rest/test2_controller.rs "
)]
#[derive(OpenApi)]
#[openapi(
    paths(

        crate::rest::other_controller::get_users,
    ),
    components(
        schemas(TestDTO)
    ),
    tags(
        (name = "todo", description = "Todo management endpoints.")
    ),
    modifiers(&SecurityAddon)
)]

pub struct ApiDoc;



```

## Exclude a method from automatic scanning

you can exclude a function from the Doc Path list by adding the following macro `#[utoipa_ignore]` .

ex:

```rust
    /// Get all pets from database
///
#[utoipa_ignore]  //<============== this Macro
#[utoipa::path(
    responses(
            (status = 200, description = "List all Pets", body = [ListPetsDTO])
    )
)]
#[get("/pets")]
async fn get_all_pets(req: HttpRequest, store: web::Data<AppState>) -> impl Responder {
    // your CODE
}

```

## Exclude a struct from automatic scanning

you can also exclude a struct from the models and reponses list by adding the following macro `#[utoipa_ignore]` .

ex:

```rust
    #[utoipa_ignore]  //<============== this Macro
#[derive(ToSchema)]
struct ModelToIgnore {
    // your CODE
}

```

### Custom path detection

By default, this macro will look for function with the `#[utoipa::path(...)]` attribute, but you can also specify a
custom attribute to look for.

```rust
#[handler]
pub fn custom_router() {
    // ...
}

#[utoipauto(function_attribute_name = "handler")] //Custom attribute
#[derive(OpenApi)]
# [openapi(tags()))]
pub struct ApiDoc;

```

You can also specify custom attributes for the model and response detection.

```rust
#[derive(Schema, Response)]
pub struct CustomModel {
    // ...
}

#[utoipauto(schema_attribute_name = "Schema", response_attribute_name = "Response")] //Custom derive
#[derive(OpenApi)]
#[openapi(tags())]
pub struct ApiDoc;

```

## Note

Sub-modules within a module containing methods tagged with utoipa::path are also automatically detected.

## Contributing

Contributions are welcomed, feel free to submit a PR or an issue.

## Inspiration

Inspired by [utoipa_auto_discovery](https://github.com/rxdiscovery/utoipa_auto_discovery)
