<h1 align="center">zjk8s</h1>

<p align="center">
  Kubernetes explorer for zellij
  <br><br>
  <a href="https://github.com/dj95/zjk8s/actions/workflows/lint.yml">
    <img alt="clippy check" src="https://github.com/dj95/zjk8s/actions/workflows/lint.yml/badge.svg" />
  </a>
  <a href="https://github.com/dj95/zjk8s/releases">
    <img alt="latest version" src="https://img.shields.io/github/v/tag/dj95/zjk8s.svg?sort=semver" />
  </a>

  <br><br>
  The goal of this plugin is to provide an easy way to explore resources in a kubernetes cluster and retrieve its details.
</p>

<img src="./assets/demo.gif" alt="Demo GIF of zellij with zjk8s" />

## 🚀 Usage

zjk8s must either be downloaded to the file system or configured with a remote URL within a zellij layout, such that zellij is able to download the plugin. The following configuration must be added to the layout, in which you'd like to use zjk8s.

```javascript
pane {
    plugin location="https://github.com/dj95/zjk8s/releases/latest/download/zjk8s.wasm" {
        kubernetes_context "k8s"

        selected_item_bg               "#737592"
        selected_col_bg                "#2a2b3e"
        selected_col_selected_item_bg  "#737592"
    }
}
```

> [!IMPORTANT]
> In case you experience any crashes or issues, please in the first step try to clear the cache! (`$HOME/.cache/zellij/` for Linux, `$HOME/Library/Caches/org.Zellij-Contributors.Zellij/` on macOS)

When the layout and the plugin is started, it should display the namespaces, all resource types in the first namespace and all resources of the first resource type. For larger clusters, it might take a few seconds to load all resources. When everything is loaded, navigate with the arrow keys through the lists and columns. If you press `Enter` on a resource, it will display the details of the resource on the right.

## ❄️ Installation with nix flake

Add this repository to your inputs and then with the following overlay to your packages.
Then you are able to install and refer to it with `pkgs.zjk8s`. When templating the
config file, you can use `${pkgs.zjk8s}/bin/zjk8s.wasm` as the path.

```nix
  inputs = {
    # ...

    zjk8s = {
      url = "github:dj95/zjk8s";
    };
  };


  # define the outputs of this flake - especially the home configurations
  outputs = { self, nixpkgs, zjstatus, ... }@inputs:
  let
    inherit (inputs.nixpkgs.lib) attrValues;

    overlays = with inputs; [
      # ...
      (final: prev: {
        zjk8s = zjk8s.packages.${prev.system}.default;
      })
    ];
```

## 🚧 Development

Make sure you have rust and the `wasm32-wasi` target installed. If using nix, you could utilize the nix-shell
in this repo for obtaining `cargo` and `rustup`. Then you'll only need to add the target with
`rustup target add wasm32-wasi`.

With the toolchain, simply build `zjk8s` with `cargo build`. Then you are able to run the example configuration
with `zellij -l plugin-dev-workspace.kdl` from the root of the repository.

## 🤝 Contributing

If you are missing features or find some annoying bugs please feel free to submit an issue or a bugfix within a pull request :)

## 📝 License

© 2024 Daniel Jankowski

This project is licensed under the MIT license.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
