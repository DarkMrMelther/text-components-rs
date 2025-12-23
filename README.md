# Text Components RS

This is a library for easy implementation and usage of Minecraft's Text Components, designed for Java edition but extensible to match Bedrock's Components.

### How to use

You can make your first text component like this:

```rs
let component = TextComponent::plain("Hello World!");
```

Decorate it like this:

```rs
let component = component.color(Color::Red).bold(true);
```

Adding interctavility like this:

```rs
let component = component.insertion("Hello");

let component = component.hover_event(
    HoverEvent::show_text("Hello World!")
);

let component = component.click_event(
    ClickEvent::open_url("https://github.com/DarkMrMelther/text-components-rs")
);
```

Once the component is ready to be sent or displayed only rests building it:

```rs
component.build(resolutor, PrettyTextBuilder);
// Equivalent of doing:
component.to_pretty_string(resolutor);
```

If you want to use serde you will need to do this instead:

```rs
component.resolve(resolutor).serialize(serializer);
```

### Roadmap

- [x] Text Components
- [x] Build system
- [x] Resolution system
- [ ] Parsing system
- [ ] Translations build macro
- [x] Terminal integration
- [x] Serde integration
- [x] SimdNbt integration
- [ ] MiniMessages integration
- [ ] Extensibility integration

### Test

To test the capabilities of the library you can execute:

```bash
cargo run --example main
```

With all the features:

```bash
cargo run --example main --features serde,nbt
```
