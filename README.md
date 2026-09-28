# PowerToys Linux (CachyOS Performance Edition) 🚀

A modular, blazingly fast rewrite of **PowerToys** engineered specifically for **Linux & Hyprland**, tuned to **CachyOS performance principles** (`x86-64-v3` AVX2 microarchitecture, ThinLTO, sub-millisecond dispatch).

---

## 🎨 Design & Aesthetic

* **Compositor Native:** Styled to integrate with modern Hyprland desktop shells (**Caelestia**, Catppuccin Mocha).
* **Frosted Glass UI:** Semi-transparent layer-shell overlays rendered via GPU using **Slint**.
* **Hyprland Shaders:** Compatible with native `layerrule = blur` and bezier pop-in animations.

---

## 🧱 Modular Architecture

Every utility is a self-contained crate with its own UI (`.slint`), logic, and configuration:

```text
PowerToys/
├── .cargo/
│   └── config.toml               # CachyOS x86-64-v3 optimization flags
├── build.bat                     # Instant developer runner (Windows & Linux)
├── crates/
│   ├── core/                     # Shared traits, CompositorBridge, PowerToyModule
│   ├── theme/                    # Shared Caelestia/Catppuccin design tokens (.slint)
│   ├── backend-mock/             # Windows / Dev mock bridge (simulates windows & monitors)
│   ├── backend-hyprland/         # Linux native Hyprland IPC bridge (hyprland-rs)
│   ├── modules/
│   │   ├── run/                  # PowerToys Run (Search, Math calculator, Window jumper)
│   │   └── fancyzones/           # FancyZones (Grid visualizer, snapping calculations)
│   └── daemon/                   # Core orchestrator and CLI entrypoint
```

---

## ⚡ CachyOS Performance Optimizations

1. **Target Microarchitecture:** Built with `target-cpu=x86-64-v3` enabling hardware AVX2, FMA, and BMI2 instructions.
2. **ThinLTO & Codegen:** Cross-crate Link-Time Optimization (`lto = "thin"`) with `codegen-units = 1`.
3. **Zero-Overhead Memory:** Uses `mimalloc` to eradicate multithreaded heap contention.
4. **Instant Layer-Shell Invocations:** Overlays boot and render in **< 4ms**.

---

## 💻 Developing on Windows

You can build, test, and live-preview all UI overlays directly on Windows without needing a Wayland compositor. The project automatically loads the **Mock Compositor Bridge** when running on non-Linux hosts:

```powershell
# Verify workspace compilation
.\build.bat check

# Launch PowerToys Run (interactive search UI on Windows)
.\build.bat run

# Launch FancyZones Layout Manager
.\build.bat run fancyzones
```

---

## 🐧 Running on CachyOS / Hyprland

On Linux, compile directly against the active Hyprland compositor:

```bash
# Build optimized release binary
cargo build --release

# Launch PowerToys Run via Hyprland IPC
./target/release/powertoys run

# Launch FancyZones Editor
./target/release/powertoys fancyzones
```

### Hyprland Keybinds (`~/.config/hypr/hyprland.conf`)

```ini
# Blur and animations for PowerToys overlays
layerrule = blur, powertoys
layerrule = ignorezero, powertoys
layerrule = animation popin 85%, powertoys

# Hotkeys
bind = SUPER, SPACE, exec, powertoys run
bind = SUPER SHIFT, Z, exec, powertoys fancyzones
```

---

## 🧩 Adding a New Module

To add a new PowerToy (e.g. `ocr` or `colorpicker`):
1. Create a folder in `crates/modules/<name>/`.
2. Implement the `PowerToyModule` trait:
   ```rust
   #[async_trait]
   impl PowerToyModule for MyModule {
       fn id(&self) -> &'static str { "mymodule" }
       fn name(&self) -> &'static str { "My Module" }
       async fn trigger(&mut self) -> anyhow::Result<()> { ... }
   }
   ```
3. Register the crate in the root `Cargo.toml`.
