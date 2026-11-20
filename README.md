# Dang Mods for RoyCraft

Independent add-ons for RoyCraft's Rust 26.3 Nightly (0a875da).

- **RoyCraft Harvest** — hold the backtick key to harvest connected logs or ores. The binding can be changed in Controls. Multiplayer harvesting needs Harvest on the server.
- **RoyCraft Jade** — a client-side HUD inspired by Jade. Look at a block or creature to see its name and, when available, health. Numpad 1 toggles the panel.
- **RoyCraft Mouse Tweaks** — an independent RoyCraft preset for the Rust client's built-in drag and item transfer controls.
- **RoyCraft Inventory Sort** — an independent RoyCraft preset for the Rust client's built-in sort buttons and middle-mouse sorting.

Use **Mods → Dang** in RoyCraft. The native add-ons also have separate `.hmod` release files. Put each desired `.hmod` in the instance's `horizon-native-mods` folder, enable it and select **Build and play**. The two JSON presets are applied by the launcher; restart the game after changing them.

## Stable v0.2.97

- Stable has a separate **RoyCraft Jade** WebAssembly edition. It shows blocks and supports the Numpad 1 toggle. Use the stable .hmod in the instance's horizon-mods folder.
- Mouse Tweaks and Inventory Sort use built-in features available in both versions.
- Harvest currently needs the Nightly runtime. Stable v0.2.97 uses the older native mixin system.

The stable package is built from roycraft_jade_stable with Rust's wasm32-unknown-unknown target. Convert the resulting core WebAssembly module with wasm-tools component new and package it as mod.wasm beside horizon-mod.toml.
