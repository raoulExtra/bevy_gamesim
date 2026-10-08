# Engine clients

Each client renders the same Rust simulation through an engine-specific adapter.

```text
Bevy / Godot / Unity / Unreal
        input + presentation
              ↕
       shared Rust protocol
              ↕
       authoritative simulation
```

The clients must not contain a second implementation of bomb, blast, damage, score, or round rules. They may interpolate state and produce speculative visual effects, but authoritative results come from Rust.

- `bevy/` — native Bevy presentation client
- `godot/` — GDExtension or external-process adapter
- `unity/` — C# native-plugin or network adapter
- `unreal/` — C++ module/plugin or network adapter
